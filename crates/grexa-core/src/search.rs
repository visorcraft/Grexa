use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder};
use regex::{Regex, RegexBuilder};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::cancel::CancelToken;
use crate::encoding::{DetectedEncoding, read_text};
use crate::models::{
    FileSearchResult, SearchOptions, SearchResult, SearchSummary, SizeLimitType, SizeUnit,
    UnicodeNormalizationMode,
};

/// Streaming events emitted by [`search_with`] when the caller supplies a
/// progress sink. Designed to be cheap to ignore — the GUI is expected to
/// debounce or batch on its side rather than pushing every event into the
/// model.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// A file was visited and skipped before scanning. Always paired with a
    /// reason so the UI can surface filtered-file counts.
    FileSkipped {
        path: PathBuf,
        reason: SkipReason,
    },
    /// A file was scanned. `matches` is the number of matched *lines* inside
    /// the file, not match occurrences.
    FileScanned {
        path: PathBuf,
        matches: usize,
    },
    /// A new match was produced. Sent eagerly so the GUI can stream rows into
    /// the table without waiting for the final summary.
    Match(SearchResult),
}

/// Why a file was skipped during traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// Path matched the system-path auto-exclusions or pseudo-filesystem
    /// guards.
    SystemPath,
    /// Path matched the user-supplied exclude-dir filter.
    ExcludedDirectory,
    /// File name did not match the user-supplied include glob set.
    FileNameMismatch,
    /// File size did not fit the size-limit filter.
    SizeLimit,
    /// File extension is a non-searchable binary.
    BinaryFile,
    /// File could not be stat'd or read.
    IoError,
}

/// Trampoline type for callers that don't want to spell out the closure
/// signature.
pub type ProgressSink<'a> = &'a mut dyn FnMut(ProgressEvent);

const MATCH_PREVIEW_MAX_CHARS: usize = 400;

static BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "obj", "bin", "zip", "tar", "gz", "7z", "rar", "png", "jpg", "jpeg", "gif",
    "bmp", "ico", "svg", "webp", "mp3", "mp4", "avi", "mkv", "wav", "flac", "ogg", "pdf", "doc",
    "docx", "xls", "xlsx", "ppt", "pptx", "pdb", "cache", "lock", "pack", "idx", "rtf",
];

static SEARCHABLE_BINARY_EXTENSIONS: &[&str] = &[
    "docx", "xlsx", "pptx", "odt", "ods", "odp", "zip", "pdf", "rtf",
];

static SYSTEM_DIRS: &[&str] = &[
    ".git",
    "vendor",
    "node_modules",
    "bin",
    "obj",
    "sys",
    "proc",
    "dev",
];

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("search path does not exist: {0}")]
    PathNotFound(PathBuf),
    #[error("search path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("invalid regex pattern: {0}")]
    InvalidRegex(String),
    #[error("invalid match file glob '{pattern}': {message}")]
    InvalidGlob { pattern: String, message: String },
    #[error("invalid exclude directory regex: {0}")]
    InvalidExcludeRegex(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience entry point that runs without cancellation or progress
/// emission. Equivalent to `search_with(options, &CancelToken::new(), None)`.
pub fn search(options: &SearchOptions) -> Result<SearchSummary, SearchError> {
    search_with(options, &CancelToken::new(), None)
}

/// Run a search with optional cooperative cancellation and progress
/// emission. The returned summary is well-formed even when cancellation
/// fires partway through: `cancelled = true` and the partial results that
/// were already produced are kept.
pub fn search_with(
    options: &SearchOptions,
    cancel: &CancelToken,
    mut progress: Option<ProgressSink<'_>>,
) -> Result<SearchSummary, SearchError> {
    let started = Instant::now();

    if !options.path.exists() {
        return Err(SearchError::PathNotFound(options.path.clone()));
    }

    if !options.path.is_dir() {
        return Err(SearchError::NotDirectory(options.path.clone()));
    }

    if options.search_term.trim().is_empty() {
        return Ok(SearchSummary {
            results: Vec::new(),
            file_results: Vec::new(),
            files_scanned: 0,
            files_matched: 0,
            matches: 0,
            skipped_files: 0,
            elapsed_ms: started.elapsed().as_millis(),
            cancelled: false,
        });
    }

    let regex = if options.regex {
        Some(
            RegexBuilder::new(&options.search_term)
                .case_insensitive(!options.case_sensitive)
                .build()
                .map_err(|err| SearchError::InvalidRegex(err.to_string()))?,
        )
    } else {
        None
    };

    let filename_filter = FileNameFilter::parse(&options.match_file_names)?;
    let exclude_filter = ExcludeDirFilter::parse(&options.exclude_dirs)?;
    let binary_extensions = extension_set(BINARY_EXTENSIONS);
    let searchable_binary_extensions = extension_set(SEARCHABLE_BINARY_EXTENSIONS);

    let mut walker = WalkBuilder::new(&options.path);
    walker
        .hidden(!options.include_hidden)
        .git_ignore(options.respect_gitignore)
        .git_exclude(options.respect_gitignore)
        .git_global(options.respect_gitignore)
        .ignore(options.respect_gitignore)
        .follow_links(options.include_symlinks)
        .same_file_system(false);

    if !options.include_subfolders {
        walker.max_depth(Some(1));
    }

    let mut results = Vec::new();
    let mut files_scanned = 0;
    let mut skipped_files = 0;
    let mut matched_files = HashSet::new();
    let mut file_encodings: HashMap<PathBuf, DetectedEncoding> = HashMap::new();
    let mut cancelled = false;

    for entry in walker.build() {
        if cancel.is_cancelled() {
            cancelled = true;
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                skipped_files += 1;
                if let Some(sink) = progress.as_deref_mut() {
                    sink(ProgressEvent::FileSkipped {
                        path: PathBuf::new(),
                        reason: SkipReason::IoError,
                    });
                }
                continue;
            }
        };

        let Some(file_type) = entry.file_type() else {
            skipped_files += 1;
            if let Some(sink) = progress.as_deref_mut() {
                sink(ProgressEvent::FileSkipped {
                    path: entry.path().to_path_buf(),
                    reason: SkipReason::IoError,
                });
            }
            continue;
        };

        if file_type.is_dir() {
            continue;
        }

        if !file_type.is_file() {
            skipped_files += 1;
            if let Some(sink) = progress.as_deref_mut() {
                sink(ProgressEvent::FileSkipped {
                    path: entry.path().to_path_buf(),
                    reason: SkipReason::IoError,
                });
            }
            continue;
        }

        if let Some(reason) = classify_skip(
            &entry,
            &options.path,
            options,
            &filename_filter,
            &exclude_filter,
            &binary_extensions,
            &searchable_binary_extensions,
        )? {
            skipped_files += 1;
            if let Some(sink) = progress.as_deref_mut() {
                sink(ProgressEvent::FileSkipped {
                    path: entry.path().to_path_buf(),
                    reason,
                });
            }
            continue;
        }

        files_scanned += 1;
        let scan = search_file(
            entry.path(),
            &options.path,
            options,
            regex.as_ref(),
            cancel,
        )?;
        if let Some(encoding) = scan.encoding {
            file_encodings.insert(entry.path().to_path_buf(), encoding);
        }
        let file_results = scan.results;
        if cancel.is_cancelled() {
            cancelled = true;
            if !file_results.is_empty() {
                matched_files.insert(entry.path().to_path_buf());
                if let Some(sink) = progress.as_deref_mut() {
                    for result in &file_results {
                        sink(ProgressEvent::Match(result.clone()));
                    }
                }
                results.extend(file_results);
            }
            break;
        }

        if !file_results.is_empty() {
            matched_files.insert(entry.path().to_path_buf());
            if let Some(sink) = progress.as_deref_mut() {
                sink(ProgressEvent::FileScanned {
                    path: entry.path().to_path_buf(),
                    matches: file_results.len(),
                });
                for result in &file_results {
                    sink(ProgressEvent::Match(result.clone()));
                }
            }
            results.extend(file_results);
        }
    }

    let matches = results.iter().map(|result| result.match_count).sum();
    let file_results = aggregate_file_results(&results, &file_encodings);

    Ok(SearchSummary {
        results,
        file_results,
        files_scanned,
        files_matched: matched_files.len(),
        matches,
        skipped_files,
        elapsed_ms: started.elapsed().as_millis(),
        cancelled,
    })
}


pub fn aggregate_file_results(
    results: &[SearchResult],
    encodings: &HashMap<PathBuf, DetectedEncoding>,
) -> Vec<FileSearchResult> {
    let mut grouped: BTreeMap<PathBuf, Vec<SearchResult>> = BTreeMap::new();
    for result in results {
        grouped
            .entry(result.full_path.clone())
            .or_default()
            .push(result.clone());
    }

    grouped
        .into_iter()
        .map(|(full_path, mut matches)| {
            matches.sort_by_key(|result| (result.line_number, result.column_number));
            let first = matches.first().cloned();
            let metadata = fs::metadata(&full_path).ok();
            let date_modified_unix = metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());

            let match_count = matches.iter().map(|result| result.match_count).sum();
            let first_match_line_number = first.as_ref().map_or(0, |result| result.line_number);
            let file_name = full_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let extension = normalized_extension(&full_path).unwrap_or_default();
            let relative_path = first
                .as_ref()
                .map(|result| result.relative_path.clone())
                .unwrap_or_else(|| full_path.clone());

            let encoding_label = encodings
                .get(&full_path)
                .copied()
                .unwrap_or(DetectedEncoding::Utf8)
                .label()
                .to_string();

            FileSearchResult {
                file_name,
                size: metadata.as_ref().map_or(0, fs::Metadata::len),
                match_count,
                first_match_line_number,
                match_preview_before: first
                    .as_ref()
                    .map(|result| result.match_preview_before.clone())
                    .unwrap_or_default(),
                match_preview_match: first
                    .as_ref()
                    .map(|result| result.match_preview_match.clone())
                    .unwrap_or_default(),
                match_preview_after: first
                    .as_ref()
                    .map(|result| result.match_preview_after.clone())
                    .unwrap_or_default(),
                preview_matches: matches,
                full_path,
                relative_path,
                extension,
                encoding: encoding_label,
                date_modified_unix,
            }
        })
        .collect()
}

fn classify_skip(
    entry: &DirEntry,
    root: &Path,
    options: &SearchOptions,
    filename_filter: &FileNameFilter,
    exclude_filter: &ExcludeDirFilter,
    binary_extensions: &HashSet<String>,
    searchable_binary_extensions: &HashSet<String>,
) -> Result<Option<SkipReason>, SearchError> {
    let path = entry.path();

    if !options.include_system && is_system_path(path) {
        return Ok(Some(SkipReason::SystemPath));
    }

    if exclude_filter.matches(path, root) {
        return Ok(Some(SkipReason::ExcludedDirectory));
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if !filename_filter.matches(file_name) {
        return Ok(Some(SkipReason::FileNameMismatch));
    }

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(Some(SkipReason::IoError)),
    };

    if !size_matches(
        metadata.len(),
        options.size_limit_type,
        options.size_limit_kb,
        options.size_unit,
    ) {
        return Ok(Some(SkipReason::SizeLimit));
    }

    let ext = normalized_extension(path);
    let is_binary = ext
        .as_ref()
        .is_some_and(|ext| binary_extensions.contains(ext));

    if is_binary && !options.include_binary {
        return Ok(Some(SkipReason::BinaryFile));
    }

    if is_binary
        && options.include_binary
        && !ext
            .as_ref()
            .is_some_and(|ext| searchable_binary_extensions.contains(ext))
    {
        return Ok(Some(SkipReason::BinaryFile));
    }

    Ok(None)
}

struct FileScan {
    results: Vec<SearchResult>,
    encoding: Option<DetectedEncoding>,
}

fn search_file(
    path: &Path,
    root: &Path,
    options: &SearchOptions,
    regex: Option<&Regex>,
    cancel: &CancelToken,
) -> Result<FileScan, SearchError> {
    // Initial slice: plain text files. Searchable binary/document extraction comes in a later phase.
    if normalized_extension(path)
        .is_some_and(|ext| SEARCHABLE_BINARY_EXTENSIONS.contains(&ext.as_str()))
    {
        return Ok(FileScan {
            results: Vec::new(),
            encoding: None,
        });
    }

    let (text, encoding) = read_text(path)?;
    let mut results = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        // Cancellation check every 64 lines keeps the latency low for both
        // tiny and huge files without paying for an atomic load per line.
        if idx % 64 == 0 && cancel.is_cancelled() {
            return Ok(FileScan {
                results,
                encoding: Some(encoding),
            });
        }

        let line_number = idx + 1;
        let matches = find_line_matches(line, options, regex);

        if matches.is_empty() {
            continue;
        }

        let (start, end) = matches[0];
        let (before, matched, after) = preview_segments(line, start, end);

        results.push(SearchResult {
            file_name: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            line_number,
            column_number: start + 1,
            line_content: truncate_chars(line, MATCH_PREVIEW_MAX_CHARS),
            match_preview_before: before,
            match_preview_match: matched,
            match_preview_after: after,
            full_path: path.to_path_buf(),
            relative_path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
            match_count: matches.len(),
        });
    }

    Ok(FileScan {
        results,
        encoding: Some(encoding),
    })
}

fn find_line_matches(
    line: &str,
    options: &SearchOptions,
    regex: Option<&Regex>,
) -> Vec<(usize, usize)> {
    if let Some(regex) = regex {
        return regex
            .find_iter(line)
            .map(|mat| (mat.start(), mat.end()))
            .collect();
    }

    let original = normalize_for_text_search(line, options);
    let needle = normalize_for_text_search(&options.search_term, options);
    if needle.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut offset = 0;
    while let Some(index) = original[offset..].find(&needle) {
        let start = offset + index;
        let end = start + needle.len();
        matches.push((start, end));
        offset = end;
    }
    matches
}

fn normalize_for_text_search(input: &str, options: &SearchOptions) -> String {
    let mut value = match options.unicode_normalization_mode {
        UnicodeNormalizationMode::None => input.to_string(),
        UnicodeNormalizationMode::FormC => input.nfc().collect(),
        UnicodeNormalizationMode::FormD => input.nfd().collect(),
        UnicodeNormalizationMode::FormKC => input.nfkc().collect(),
        UnicodeNormalizationMode::FormKD => input.nfkd().collect(),
    };

    if !options.diacritic_sensitive {
        value = value.nfd().filter(|ch| !is_combining_mark(*ch)).collect();
    }

    if !options.case_sensitive {
        value = value.to_lowercase();
    }

    value
}

fn preview_segments(line: &str, start: usize, end: usize) -> (String, String, String) {
    let before = line.get(..start).unwrap_or_default();
    let matched = line.get(start..end).unwrap_or_default();
    let after = line.get(end..).unwrap_or_default();

    (
        truncate_chars(before, 120),
        matched.to_string(),
        truncate_chars(after, 120),
    )
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    input.chars().take(max_chars).collect()
}

fn size_matches(
    size_bytes: u64,
    limit_type: SizeLimitType,
    limit_kb: Option<u64>,
    unit: SizeUnit,
) -> bool {
    let Some(limit_kb) = limit_kb else {
        return true;
    };

    if limit_type == SizeLimitType::NoLimit {
        return true;
    }

    let size_kb = size_bytes.div_ceil(1024);
    let tolerance = match unit {
        SizeUnit::KB => 10,
        SizeUnit::MB => 1024,
        SizeUnit::GB => 25 * 1024,
    };

    match limit_type {
        SizeLimitType::NoLimit => true,
        SizeLimitType::LessThan => size_kb <= limit_kb.saturating_add(tolerance),
        SizeLimitType::EqualTo => {
            let min = limit_kb.saturating_sub(tolerance);
            let max = limit_kb.saturating_add(tolerance);
            (min..=max).contains(&size_kb)
        }
        SizeLimitType::GreaterThan => size_kb >= limit_kb.saturating_sub(tolerance),
    }
}

fn is_system_path(path: &Path) -> bool {
    let components: Vec<String> = path
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(str::to_string))
        .collect();

    if components
        .iter()
        .any(|component| SYSTEM_DIRS.contains(&component.as_str()))
    {
        return true;
    }

    components
        .windows(2)
        .any(|window| window[0] == "storage" && window[1] == "framework")
}

fn normalized_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn extension_set(values: &[&str]) -> HashSet<String> {
    values.iter().map(|value| value.to_string()).collect()
}

struct FileNameFilter {
    includes: Option<GlobSet>,
    excludes: Option<GlobSet>,
}

impl FileNameFilter {
    fn parse(patterns: &str) -> Result<Self, SearchError> {
        let mut includes = GlobSetBuilder::new();
        let mut excludes = GlobSetBuilder::new();
        let mut has_include = false;
        let mut has_exclude = false;

        for raw in patterns
            .split(['|', ';'])
            .map(str::trim)
            .filter(|item| !item.is_empty())
        {
            let (target, excluded) = raw
                .strip_prefix('-')
                .map(|pattern| (pattern, true))
                .unwrap_or((raw, false));

            let glob = Glob::new(target).map_err(|err| SearchError::InvalidGlob {
                pattern: target.to_string(),
                message: err.to_string(),
            })?;

            if excluded {
                excludes.add(glob);
                has_exclude = true;
            } else {
                includes.add(glob);
                has_include = true;
            }
        }

        Ok(Self {
            includes: has_include
                .then(|| includes.build())
                .transpose()
                .map_err(|err| SearchError::InvalidGlob {
                    pattern: patterns.to_string(),
                    message: err.to_string(),
                })?,
            excludes: has_exclude
                .then(|| excludes.build())
                .transpose()
                .map_err(|err| SearchError::InvalidGlob {
                    pattern: patterns.to_string(),
                    message: err.to_string(),
                })?,
        })
    }

    fn matches(&self, file_name: &str) -> bool {
        if self
            .excludes
            .as_ref()
            .is_some_and(|excludes| excludes.is_match(file_name))
        {
            return false;
        }

        self.includes
            .as_ref()
            .is_none_or(|includes| includes.is_match(file_name))
    }
}

enum ExcludeDirFilter {
    Empty,
    Names(HashSet<String>),
    Regex(Regex),
}

impl ExcludeDirFilter {
    fn parse(value: &str) -> Result<Self, SearchError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(Self::Empty);
        }

        if (trimmed.contains('^') || trimmed.contains('$') || trimmed.contains('|'))
            && !trimmed.contains(',')
            && !trimmed.contains(';')
        {
            return Regex::new(trimmed)
                .map(Self::Regex)
                .map_err(|err| SearchError::InvalidExcludeRegex(err.to_string()));
        }

        let names = trimmed
            .split([',', ';'])
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(|item| item.to_ascii_lowercase())
            .collect();

        Ok(Self::Names(names))
    }

    fn matches(&self, path: &Path, root: &Path) -> bool {
        match self {
            Self::Empty => false,
            Self::Names(names) => path
                .strip_prefix(root)
                .unwrap_or(path)
                .components()
                .filter_map(|component| component.as_os_str().to_str())
                .any(|component| names.contains(&component.to_ascii_lowercase())),
            Self::Regex(regex) => path
                .components()
                .filter_map(|component| component.as_os_str().to_str())
                .any(|component| regex.is_match(component)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn finds_plain_text_matches() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("app.rs"),
            "fn main() {\n    // TODO fix\n}\n",
        )
        .unwrap();

        let options = SearchOptions::new(dir.path(), "TODO");
        let summary = search(&options).unwrap();

        assert_eq!(summary.matches, 1);
        assert_eq!(summary.files_matched, 1);
        assert_eq!(summary.file_results.len(), 1);
        assert_eq!(summary.results[0].line_number, 2);
        assert_eq!(summary.results[0].column_number, 8);
    }

    #[test]
    fn applies_match_file_exclusions() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("keep.rs"), "TODO\n").unwrap();
        fs::write(dir.path().join("skip.log"), "TODO\n").unwrap();

        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.match_file_names = "*.rs|-skip*".to_string();

        let summary = search(&options).unwrap();

        assert_eq!(summary.files_matched, 1);
        assert_eq!(summary.results[0].file_name, "keep.rs");
    }

    #[test]
    fn supports_regex_search() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("data.txt"), "abc-123\nabc-xyz\n").unwrap();

        let mut options = SearchOptions::new(dir.path(), r"abc-\d+");
        options.regex = true;

        let summary = search(&options).unwrap();

        assert_eq!(summary.matches, 1);
        assert_eq!(summary.results[0].line_number, 1);
    }

    #[test]
    fn supports_diacritic_insensitive_text_search() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("words.txt"), "café\n").unwrap();

        let mut options = SearchOptions::new(dir.path(), "cafe");
        options.diacritic_sensitive = false;

        let summary = search(&options).unwrap();

        assert_eq!(summary.matches, 1);
    }

    #[test]
    fn excludes_system_paths_by_default() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        fs::write(dir.path().join("node_modules").join("dep.js"), "TODO\n").unwrap();
        fs::write(dir.path().join("app.js"), "TODO\n").unwrap();

        let options = SearchOptions::new(dir.path(), "TODO");
        let summary = search(&options).unwrap();

        assert_eq!(summary.files_matched, 1);
        assert_eq!(summary.results[0].file_name, "app.js");
    }

    #[test]
    fn cancellation_returns_partial_summary() {
        let dir = tempdir().unwrap();
        for i in 0..50 {
            fs::write(dir.path().join(format!("f{i}.txt")), "TODO\n").unwrap();
        }

        let cancel = CancelToken::new();
        cancel.cancel();
        let summary = search_with(&SearchOptions::new(dir.path(), "TODO"), &cancel, None).unwrap();
        assert!(summary.cancelled);
        // Walker may have iterated before checking; partial counts are fine,
        // the contract is that we return cleanly with the flag set.
        assert!(summary.files_scanned <= 50);
    }

    #[test]
    fn progress_sink_receives_match_events() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "TODO\nTODO again\n").unwrap();
        fs::write(dir.path().join("b.log"), "TODO\n").unwrap();

        let mut events = Vec::new();
        let mut sink = |event: ProgressEvent| events.push(event);
        let summary = search_with(
            &SearchOptions::new(dir.path(), "TODO"),
            &CancelToken::new(),
            Some(&mut sink),
        )
        .unwrap();
        assert!(!summary.cancelled);
        let matches: Vec<_> = events
            .iter()
            .filter(|event| matches!(event, ProgressEvent::Match(_)))
            .collect();
        assert!(!matches.is_empty());
        let scanned: Vec<_> = events
            .iter()
            .filter(|event| matches!(event, ProgressEvent::FileScanned { .. }))
            .collect();
        assert!(!scanned.is_empty());
    }

    #[test]
    fn progress_sink_records_skip_reasons() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("keep.rs"), "TODO\n").unwrap();
        fs::write(dir.path().join("ignore.log"), "TODO\n").unwrap();

        let mut events = Vec::new();
        let mut sink = |event: ProgressEvent| events.push(event);
        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.match_file_names = "*.rs".to_string();

        search_with(&options, &CancelToken::new(), Some(&mut sink)).unwrap();
        assert!(events.iter().any(|event| matches!(
            event,
            ProgressEvent::FileSkipped {
                reason: SkipReason::FileNameMismatch,
                ..
            }
        )));
    }

    #[test]
    fn search_handles_utf16_le_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        let mut bytes = vec![0xFF, 0xFE];
        for ch in "hello\nTODO café\n".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        fs::write(&path, &bytes).unwrap();

        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert_eq!(summary.matches, 1);
        assert_eq!(summary.file_results[0].encoding, "UTF-16 LE");
    }

    #[test]
    fn search_tolerates_invalid_utf8_bytes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.txt");
        // "TODO" followed by raw 0xFF, then "fix" — used to crash the line
        // reader; now lossily decodes.
        fs::write(&path, b"TODO\xFFfix\n").unwrap();
        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert_eq!(summary.matches, 1);
        assert_eq!(summary.file_results[0].encoding, "UTF-8");
    }

    #[test]
    fn aggregates_matches_by_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("app.txt"), "TODO one\nTODO two\n").unwrap();

        let options = SearchOptions::new(dir.path(), "TODO");
        let summary = search(&options).unwrap();

        assert_eq!(summary.results.len(), 2);
        assert_eq!(summary.file_results.len(), 1);
        assert_eq!(summary.file_results[0].match_count, 2);
        assert_eq!(summary.file_results[0].first_match_line_number, 1);
        assert_eq!(summary.file_results[0].preview_matches.len(), 2);
    }
}
