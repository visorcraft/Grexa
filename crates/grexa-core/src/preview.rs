// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::encoding::{DetectedEncoding, read_text};

/// Bounds matching Grex's settings clamp: 1 to 20 lines on each side.
pub const MIN_CONTEXT_LINES: u8 = 1;
pub const MAX_CONTEXT_LINES: u8 = 20;
pub const DEFAULT_CONTEXT_LINES: u8 = 5;

/// One line of context around a match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextLine {
    pub line_number: usize,
    pub content: String,
    pub is_match: bool,
}

/// Result of [`context_preview`]. Line numbers are 1-based, inclusive of
/// both `lines_before` and `lines_after`. `match_line_index` is the
/// 0-based offset of the requested line inside `lines`, or `None` when the
/// file is shorter than `match_line_number`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPreviewResult {
    pub file_name: String,
    pub full_path: PathBuf,
    pub match_line_number: usize,
    pub lines: Vec<ContextLine>,
    pub match_line_index: Option<usize>,
    pub encoding: DetectedEncoding,
}

#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("file path must not be empty")]
    EmptyPath,
    #[error("line number must be >= 1")]
    InvalidLineNumber,
    #[error("file not found: {0}")]
    NotFound(PathBuf),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Render `lines_before` lines before `line_number` and `lines_after` lines
/// after it. Both counts are clamped to `1..=20` to match Grex settings.
pub fn context_preview(
    path: &Path,
    line_number: usize,
    lines_before: u8,
    lines_after: u8,
) -> Result<ContextPreviewResult, PreviewError> {
    if path.as_os_str().is_empty() {
        return Err(PreviewError::EmptyPath);
    }
    if line_number < 1 {
        return Err(PreviewError::InvalidLineNumber);
    }

    let lines_before = lines_before.clamp(MIN_CONTEXT_LINES, MAX_CONTEXT_LINES) as usize;
    let lines_after = lines_after.clamp(MIN_CONTEXT_LINES, MAX_CONTEXT_LINES) as usize;

    let (text, encoding) = read_text(path).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => PreviewError::NotFound(path.to_path_buf()),
        io::ErrorKind::PermissionDenied => PreviewError::PermissionDenied(path.to_path_buf()),
        _ => PreviewError::Io(err),
    })?;

    let start_line = line_number.saturating_sub(lines_before).max(1);
    let end_line = line_number.saturating_add(lines_after);

    let mut lines = Vec::new();
    let mut match_line_index = None;

    for (idx, content) in text.lines().enumerate() {
        let current_line = idx + 1;
        if current_line < start_line {
            continue;
        }
        if current_line > end_line {
            break;
        }

        let is_match = current_line == line_number;
        if is_match {
            match_line_index = Some(lines.len());
        }
        lines.push(ContextLine {
            line_number: current_line,
            content: content.to_string(),
            is_match,
        });
    }

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(ContextPreviewResult {
        file_name,
        full_path: path.to_path_buf(),
        match_line_number: line_number,
        lines,
        match_line_index,
        encoding,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::*;

    fn write(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
    }

    #[test]
    fn returns_requested_range_inclusive() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "L1\nL2\nL3\nL4\nL5\nL6\nL7\n");

        let result = context_preview(&path, 4, 2, 2).unwrap();
        let numbers: Vec<_> = result.lines.iter().map(|l| l.line_number).collect();
        assert_eq!(numbers, vec![2, 3, 4, 5, 6]);
        assert_eq!(result.match_line_index, Some(2));
        assert!(result.lines[2].is_match);
        assert!(!result.lines[0].is_match);
    }

    #[test]
    fn clamps_at_file_start() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "L1\nL2\nL3\n");

        let result = context_preview(&path, 1, 5, 1).unwrap();
        let numbers: Vec<_> = result.lines.iter().map(|l| l.line_number).collect();
        assert_eq!(numbers, vec![1, 2]);
        assert_eq!(result.match_line_index, Some(0));
    }

    #[test]
    fn clamps_at_file_end() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "L1\nL2\nL3\n");

        let result = context_preview(&path, 3, 1, 5).unwrap();
        let numbers: Vec<_> = result.lines.iter().map(|l| l.line_number).collect();
        assert_eq!(numbers, vec![2, 3]);
        assert_eq!(result.match_line_index, Some(1));
    }

    #[test]
    fn line_beyond_eof_yields_no_match_index() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "L1\nL2\n");

        let result = context_preview(&path, 10, 2, 2).unwrap();
        assert!(result.match_line_index.is_none());
    }

    #[test]
    fn empty_file_returns_empty_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "");

        let result = context_preview(&path, 1, 5, 5).unwrap();
        assert!(result.lines.is_empty());
        assert_eq!(result.match_line_index, None);
    }

    #[test]
    fn missing_file_returns_not_found() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nope.txt");
        let err = context_preview(&path, 1, 5, 5).unwrap_err();
        assert!(matches!(err, PreviewError::NotFound(_)));
    }

    #[test]
    fn line_number_zero_is_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        write(&path, "L1\n");
        let err = context_preview(&path, 0, 5, 5).unwrap_err();
        assert!(matches!(err, PreviewError::InvalidLineNumber));
    }

    #[test]
    fn clamps_request_counts_to_settings_range() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let body: String = (1..=100).map(|n| format!("L{n}\n")).collect();
        write(&path, &body);

        // request 0 lines: should treat as MIN_CONTEXT_LINES (1)
        let result = context_preview(&path, 50, 0, 0).unwrap();
        let numbers: Vec<_> = result.lines.iter().map(|l| l.line_number).collect();
        assert_eq!(numbers, vec![49, 50, 51]);

        // request 100: should clamp to MAX_CONTEXT_LINES (20)
        let result = context_preview(&path, 50, 100, 100).unwrap();
        let numbers: Vec<_> = result.lines.iter().map(|l| l.line_number).collect();
        assert_eq!(numbers.first(), Some(&30));
        assert_eq!(numbers.last(), Some(&70));
    }

    #[test]
    fn handles_utf16_le_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let mut bytes = vec![0xFF, 0xFE];
        for ch in "first\nsecond\nthird\n".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        fs::write(&path, bytes).unwrap();

        let result = context_preview(&path, 2, 1, 1).unwrap();
        assert_eq!(result.encoding, DetectedEncoding::Utf16Le);
        let contents: Vec<_> = result.lines.iter().map(|l| l.content.clone()).collect();
        assert_eq!(contents, vec!["first", "second", "third"]);
    }
}
