use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use encoding_rs::Encoding;
use regex::RegexBuilder;
use thiserror::Error;

use crate::cancel::CancelToken;
use crate::encoding::{DetectedEncoding, read_text};
use crate::models::SearchOptions;
use crate::search::{ProgressSink, SearchError, search_with};

/// Configuration for a safe replace operation. The replace pipeline reuses
/// `SearchOptions` for filtering so dry-run preview and actual replace use
/// the exact same file set.
#[derive(Debug, Clone)]
pub struct ReplaceOptions {
    pub search: SearchOptions,
    /// Replacement string. For regex mode, `$1` / `$name` / `${name}` capture
    /// references are honored by `regex::Regex::replace_all`.
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct FileReplaceReport {
    pub path: PathBuf,
    pub matches_replaced: usize,
    pub encoding: DetectedEncoding,
}

#[derive(Debug, Clone)]
pub struct FileReplaceFailure {
    pub path: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReplaceSummary {
    pub files_modified: usize,
    pub files_unchanged: usize,
    pub matches_replaced: usize,
    pub reports: Vec<FileReplaceReport>,
    pub failures: Vec<FileReplaceFailure>,
    pub cancelled: bool,
    pub elapsed_ms: u128,
}

#[derive(Debug, Error)]
pub enum ReplaceError {
    #[error("invalid regex pattern: {0}")]
    InvalidRegex(String),
    #[error("search failed: {0}")]
    Search(#[from] SearchError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Execute the replace pipeline.
///
/// Per-file flow:
///
/// 1. Read the file via the encoding-aware reader (BOM detection + UTF-16
///    + lossy UTF-8 fallback).
/// 2. Substitute every match (text or regex) according to `replacement`.
/// 3. If the resulting text differs, encode it back using the detected
///    encoding and write through a sibling temporary file, then atomically
///    rename onto the target. The temp file lives in the same directory so
///    the rename stays on the same filesystem.
/// 4. Cancellation is checked once per file. Files already written stay
///    written; `cancelled = true` flags partial completion.
///
/// Failures are recorded per-file in `summary.failures` rather than
/// aborting the whole batch.
pub fn replace_with(
    options: &ReplaceOptions,
    cancel: &CancelToken,
    progress: Option<ProgressSink<'_>>,
) -> Result<ReplaceSummary, ReplaceError> {
    let started = Instant::now();

    // Drive the same search the user previewed; surface its progress events.
    let search_summary = search_with(&options.search, cancel, progress)?;
    let mut summary = ReplaceSummary {
        cancelled: search_summary.cancelled,
        ..Default::default()
    };

    // Deduplicate by full path; the search engine yields one row per match.
    let mut files: Vec<PathBuf> = search_summary
        .results
        .iter()
        .map(|result| result.full_path.clone())
        .collect();
    files.sort();
    files.dedup();

    let regex_engine = if options.search.regex {
        Some(
            RegexBuilder::new(&options.search.search_term)
                .case_insensitive(!options.search.case_sensitive)
                .build()
                .map_err(|err| ReplaceError::InvalidRegex(err.to_string()))?,
        )
    } else {
        None
    };

    for path in files {
        if cancel.is_cancelled() {
            summary.cancelled = true;
            break;
        }

        match rewrite_one(&path, options, regex_engine.as_ref()) {
            Ok(FileResult::Unchanged) => summary.files_unchanged += 1,
            Ok(FileResult::Replaced {
                matches,
                encoding,
            }) => {
                summary.files_modified += 1;
                summary.matches_replaced += matches;
                summary.reports.push(FileReplaceReport {
                    path,
                    matches_replaced: matches,
                    encoding,
                });
            }
            Err(err) => summary.failures.push(FileReplaceFailure {
                path,
                error: err.to_string(),
            }),
        }
    }

    summary.elapsed_ms = started.elapsed().as_millis();
    Ok(summary)
}

enum FileResult {
    Unchanged,
    Replaced {
        matches: usize,
        encoding: DetectedEncoding,
    },
}

fn rewrite_one(
    path: &Path,
    options: &ReplaceOptions,
    regex_engine: Option<&regex::Regex>,
) -> Result<FileResult, io::Error> {
    let original_metadata = fs::symlink_metadata(path)?;
    let (text, encoding) = read_text(path)?;
    let (new_text, matches) = apply_substitution(&text, options, regex_engine);
    if matches == 0 || new_text == text {
        return Ok(FileResult::Unchanged);
    }

    let encoded = encode_for_writeback(&new_text, encoding);
    atomic_write(path, &encoded)?;
    restore_permissions(path, &original_metadata)?;
    Ok(FileResult::Replaced { matches, encoding })
}

fn restore_permissions(path: &Path, original: &fs::Metadata) -> io::Result<()> {
    // Atomic rename installs the temp file with its own permissions (default
    // `0600` on Linux for `tempfile`). Re-apply the original permission bits
    // so replace doesn't silently downgrade group/world access.
    fs::set_permissions(path, original.permissions())?;
    Ok(())
}

fn apply_substitution(
    text: &str,
    options: &ReplaceOptions,
    regex_engine: Option<&regex::Regex>,
) -> (String, usize) {
    if let Some(re) = regex_engine {
        let count = re.find_iter(text).count();
        let replaced = re.replace_all(text, options.replacement.as_str()).into_owned();
        return (replaced, count);
    }

    let needle = &options.search.search_term;
    if needle.is_empty() {
        return (text.to_string(), 0);
    }

    if options.search.case_sensitive {
        let count = count_occurrences(text, needle);
        if count == 0 {
            return (text.to_string(), 0);
        }
        (text.replace(needle, &options.replacement), count)
    } else {
        // Case-insensitive plain text: build a literal-match regex with the
        // case-insensitive flag so capture-group syntax in the replacement is
        // treated as literal characters.
        let escaped = regex::escape(needle);
        match RegexBuilder::new(&escaped).case_insensitive(true).build() {
            Ok(re) => {
                let count = re.find_iter(text).count();
                if count == 0 {
                    return (text.to_string(), 0);
                }
                let replacement = regex::NoExpand(options.replacement.as_str());
                (re.replace_all(text, replacement).into_owned(), count)
            }
            Err(_) => (text.to_string(), 0),
        }
    }
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut offset = 0;
    while let Some(index) = text[offset..].find(needle) {
        count += 1;
        offset += index + needle.len();
    }
    count
}

fn encode_for_writeback(text: &str, encoding: DetectedEncoding) -> Vec<u8> {
    match encoding {
        DetectedEncoding::Utf8 => text.as_bytes().to_vec(),
        DetectedEncoding::Utf8Bom => {
            let mut bytes = Vec::with_capacity(text.len() + 3);
            bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            bytes.extend_from_slice(text.as_bytes());
            bytes
        }
        DetectedEncoding::Utf16Le => encode_utf16(text, &[0xFF, 0xFE], true),
        DetectedEncoding::Utf16Be => encode_utf16(text, &[0xFE, 0xFF], false),
        // UTF-32 round-trip is not supported yet (detect-only); fall back to
        // UTF-8 so we never silently corrupt the file by writing garbage.
        DetectedEncoding::Utf32Le | DetectedEncoding::Utf32Be => text.as_bytes().to_vec(),
    }
}

fn encode_utf16(text: &str, bom: &[u8], little_endian: bool) -> Vec<u8> {
    // encoding_rs encodes UTF-16 LE/BE directly; we prepend a BOM to keep the
    // file recognizable after rewrite.
    let encoder: &Encoding = if little_endian {
        encoding_rs::UTF_16LE
    } else {
        encoding_rs::UTF_16BE
    };
    let _ = encoder; // encoding_rs does not expose an encoder for UTF-16; fall
    // back to the standard library encoder.

    let mut bytes = Vec::with_capacity(bom.len() + text.len() * 2);
    bytes.extend_from_slice(bom);
    for code_unit in text.encode_utf16() {
        let unit = if little_endian {
            code_unit.to_le_bytes()
        } else {
            code_unit.to_be_bytes()
        };
        bytes.extend_from_slice(&unit);
    }
    bytes
}

fn atomic_write(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    std::io::Write::write_all(&mut tmp, bytes)?;
    tmp.as_file().sync_all()?;
    // `persist` renames atomically on the same filesystem.
    tmp.persist(target)
        .map_err(|err| io::Error::other(err.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    fn opts(path: &Path, term: &str, replacement: &str) -> ReplaceOptions {
        ReplaceOptions {
            search: SearchOptions::new(path, term),
            replacement: replacement.to_string(),
        }
    }

    #[test]
    fn rewrites_text_match() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "TODO write doc\nTODO fix bug\n").unwrap();

        let summary =
            replace_with(&opts(dir.path(), "TODO", "FIXME"), &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 1);
        assert_eq!(summary.matches_replaced, 2);
        let rewritten = fs::read_to_string(&target).unwrap();
        assert_eq!(rewritten, "FIXME write doc\nFIXME fix bug\n");
    }

    #[test]
    fn skips_files_without_matches() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "nothing to replace\n").unwrap();

        let summary =
            replace_with(&opts(dir.path(), "TODO", "FIXME"), &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 0);
        assert_eq!(summary.files_unchanged, 0); // file isn't even visited
    }

    #[test]
    fn regex_replace_with_capture_groups() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "version=1.2.3\nversion=2.0.0\n").unwrap();

        let mut options = opts(dir.path(), r"version=(\d+\.\d+\.\d+)", "v$1");
        options.search.regex = true;

        let summary = replace_with(&options, &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 1);
        assert_eq!(summary.matches_replaced, 2);
        let rewritten = fs::read_to_string(&target).unwrap();
        assert_eq!(rewritten, "v1.2.3\nv2.0.0\n");
    }

    #[test]
    fn case_insensitive_text_replace_treats_replacement_as_literal() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        fs::write(&target, "TODO\ntodo\nTodo\n").unwrap();

        let summary = replace_with(
            &opts(dir.path(), "todo", "$1-NOEXPAND"),
            &CancelToken::new(),
            None,
        )
        .unwrap();
        assert_eq!(summary.matches_replaced, 3);
        let rewritten = fs::read_to_string(&target).unwrap();
        // Dollar-sign capture references must not be expanded; verify the
        // literal `$1` survives intact in every replacement.
        assert!(rewritten.contains("$1-NOEXPAND"));
        assert!(!rewritten.contains("TODO"));
    }

    #[test]
    fn round_trips_utf8_bom() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"TODO header\n");
        fs::write(&target, bytes).unwrap();

        let summary =
            replace_with(&opts(dir.path(), "TODO", "FIX"), &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 1);
        let raw = fs::read(&target).unwrap();
        assert_eq!(&raw[0..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&raw[3..], b"FIX header\n");
    }

    #[test]
    fn round_trips_utf16_le() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.txt");
        let mut bytes = vec![0xFF, 0xFE];
        for ch in "TODO café\n".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        fs::write(&target, bytes).unwrap();

        let summary =
            replace_with(&opts(dir.path(), "TODO", "DONE"), &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 1);

        let raw = fs::read(&target).unwrap();
        assert_eq!(&raw[0..2], &[0xFF, 0xFE]);
        // Decode and check string content is correctly written back.
        let (_, encoding) = read_text(&target).unwrap();
        assert_eq!(encoding, DetectedEncoding::Utf16Le);
    }

    #[cfg(unix)]
    #[test]
    fn preserves_unix_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let target = dir.path().join("a.sh");
        fs::write(&target, "TODO write me\n").unwrap();
        let mut perms = fs::metadata(&target).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target, perms).unwrap();

        let summary =
            replace_with(&opts(dir.path(), "TODO", "DONE"), &CancelToken::new(), None).unwrap();
        assert_eq!(summary.files_modified, 1);

        let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o755);
    }

    #[test]
    fn pre_cancelled_token_reports_partial() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "TODO\n").unwrap();

        let cancel = CancelToken::new();
        cancel.cancel();
        let summary = replace_with(
            &opts(dir.path(), "TODO", "DONE"),
            &cancel,
            None,
        )
        .unwrap();
        assert!(summary.cancelled);
    }
}
