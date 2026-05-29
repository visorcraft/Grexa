// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Baloo candidate-seeding adapter.
//!
//! Phase 13 ships Baloo as an **optional candidate source**, never the source
//! of truth — every candidate file is re-verified by Grexa's own search
//! engine before it can appear in a result list. This module provides the
//! trait surface, a CLI-backed implementation that shells out to
//! `baloosearch`, and a `NullBalooAdapter` for environments without KDE.
//!
//! The spike outcome and keep/defer/drop recommendation are documented in
//! `docs/baloo-spike.md`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BalooError {
    #[error("baloosearch CLI is not installed")]
    NotInstalled,
    #[error("baloosearch failed: {0}")]
    Cli(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Adapter trait. The runtime decides whether to consult Baloo based on
/// `is_available()` plus `is_path_indexed(root)`. When both return `true`,
/// the GUI / CLI calls `candidates_for(query, root)` to obtain a candidate
/// list. The list is always re-verified by [`crate::search::search_with`]
/// before any row reaches the user.
pub trait BalooAdapter: Send + Sync {
    fn is_available(&self) -> bool;
    fn is_path_indexed(&self, root: &Path) -> bool;
    fn candidates_for(&self, query: &str, root: &Path) -> Result<Vec<PathBuf>, BalooError>;
}

/// No-op adapter — used when KDE / Baloo isn't present, or when the user
/// has opted out of indexing.
pub struct NullBalooAdapter;

impl BalooAdapter for NullBalooAdapter {
    fn is_available(&self) -> bool {
        false
    }
    fn is_path_indexed(&self, _root: &Path) -> bool {
        false
    }
    fn candidates_for(&self, _query: &str, _root: &Path) -> Result<Vec<PathBuf>, BalooError> {
        Ok(Vec::new())
    }
}

/// CLI-backed adapter that shells out to `baloosearch` / `balooctl6`.
/// Detects availability by probing the binaries on `$PATH`.
pub struct BaloosearchCliAdapter;

impl BaloosearchCliAdapter {
    pub fn new() -> Self {
        Self
    }

    fn find_cli() -> Option<PathBuf> {
        for name in &["baloosearch6", "baloosearch", "baloo-search"] {
            if let Ok(path_env) = std::env::var("PATH") {
                for dir in std::env::split_paths(&path_env) {
                    let candidate = dir.join(name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }
}

impl Default for BaloosearchCliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BalooAdapter for BaloosearchCliAdapter {
    fn is_available(&self) -> bool {
        Self::find_cli().is_some()
    }

    fn is_path_indexed(&self, root: &Path) -> bool {
        // `balooctl6 indexSize` exits 0 when the indexer is configured and
        // the path is included. The CLI's exit codes aren't documented, so
        // be conservative: report `false` unless `find_cli` exists *and*
        // the requested path lives under `$HOME` (Baloo's default scope).
        if Self::find_cli().is_none() {
            return false;
        }
        std::env::var_os("HOME")
            .map(|home| {
                let home = PathBuf::from(home);
                root.starts_with(home)
            })
            .unwrap_or(false)
    }

    fn candidates_for(&self, query: &str, root: &Path) -> Result<Vec<PathBuf>, BalooError> {
        let cli = Self::find_cli().ok_or(BalooError::NotInstalled)?;
        let output = Command::new(cli)
            .args(["-d"])
            .arg(root)
            .arg(query)
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()?;
        if !output.status.success() {
            return Err(BalooError::Cli(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
}

/// Test adapter — returns canned candidate lists.
#[derive(Default)]
pub struct StubBalooAdapter {
    pub available: bool,
    pub indexed_roots: Vec<PathBuf>,
    pub candidates: Vec<PathBuf>,
}

impl StubBalooAdapter {
    pub fn with_candidates(candidates: Vec<PathBuf>) -> Self {
        Self {
            available: true,
            indexed_roots: vec![PathBuf::from("/")],
            candidates,
        }
    }
}

impl BalooAdapter for StubBalooAdapter {
    fn is_available(&self) -> bool {
        self.available
    }
    fn is_path_indexed(&self, root: &Path) -> bool {
        self.indexed_roots.iter().any(|r| root.starts_with(r))
    }
    fn candidates_for(&self, _query: &str, _root: &Path) -> Result<Vec<PathBuf>, BalooError> {
        Ok(self.candidates.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_adapter_reports_unavailable() {
        let adapter = NullBalooAdapter;
        assert!(!adapter.is_available());
        assert!(!adapter.is_path_indexed(Path::new("/home/me")));
        assert!(
            adapter
                .candidates_for("anything", Path::new("/"))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn stub_adapter_returns_canned_candidates() {
        let stub = StubBalooAdapter::with_candidates(vec![
            PathBuf::from("/home/me/a.txt"),
            PathBuf::from("/home/me/b.txt"),
        ]);
        assert!(stub.is_available());
        assert!(stub.is_path_indexed(Path::new("/home/me/projects")));
        let hits = stub.candidates_for("query", Path::new("/")).unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn baloosearch_cli_adapter_reports_unavailable_when_binary_missing() {
        // CI typically doesn't ship baloosearch. We can't override $PATH
        // without affecting other tests, so this test only pins the
        // behavior when `find_cli` returns `None`.
        let adapter = BaloosearchCliAdapter::new();
        if BaloosearchCliAdapter::find_cli().is_none() {
            assert!(!adapter.is_available());
            assert!(!adapter.is_path_indexed(Path::new("/home/me")));
            let err = adapter.candidates_for("x", Path::new("/")).unwrap_err();
            assert!(matches!(err, BalooError::NotInstalled));
        }
    }
}
