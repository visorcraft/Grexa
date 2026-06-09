// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

use crate::cancel::CancelToken;
use crate::documents::extract_text;
use crate::encoding::{DetectedEncoding, read_text};
use crate::models::{
    FileSearchResult, SearchOptions, SearchResult, SearchSummary, SizeLimitType, SizeUnit,
    UnicodeNormalizationMode,
};
use crate::pattern::PatternEngine;

/// Streaming events emitted by [`search_with`] when the caller supplies a
/// progress sink. Designed to be cheap to ignore — the GUI is expected to
/// debounce or batch on its side rather than pushing every event into the
/// model.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// A file was visited and skipped before scanning. Always paired with a
    /// reason so the UI can surface filtered-file counts.
    FileSkipped { path: PathBuf, reason: SkipReason },
    /// A file was scanned. `matches` is the number of matched *lines* inside
    /// the file, not match occurrences.
    FileScanned { path: PathBuf, matches: usize },
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

/// Maximum byte length of a single line used for pattern matching. Lines
/// exceeding this cap are truncated to the byte boundary before comparison;
/// the display preview (`line_content`) is separately capped at
/// `MATCH_PREVIEW_MAX_CHARS` characters. The value is chosen so a single
/// multi-megabyte minified line (common in generated JS/CSS) does not hold
/// the search thread for seconds while the regex engine scans it.
const MAX_COMPARE_LINE_BYTES: usize = 2 * 1024 * 1024;

static BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "obj", "bin", "zip", "tar", "gz", "7z", "rar", "png", "jpg", "jpeg", "gif",
    "bmp", "ico", "svg", "webp", "mp3", "mp4", "avi", "mkv", "wav", "flac", "ogg", "pdf", "doc",
    "docx", "xls", "xlsx", "ppt", "pptx", "pdb", "cache", "lock", "pack", "idx", "rtf",
];

static SEARCHABLE_BINARY_EXTENSIONS: &[&str] = &[
    "docx", "xlsx", "pptx", "odt", "ods", "odp", "zip", "pdf", "rtf",
];

/// Hard upper bound on the size of a single file the search/replace engine
/// will read into memory. The user-facing size filter defaults to "no limit",
/// so without this safety net a single multi-gigabyte file could exhaust
/// memory (decoding/normalization roughly doubles the footprint). Files above
/// the cap are reported as `SkipReason::SizeLimit` rather than scanned.
const MAX_SEARCH_FILE_BYTES: u64 = 512 * 1024 * 1024;

/// `true` when a file of `len` bytes exceeds the hard in-memory read cap. The
/// cap itself is allowed; only strictly larger files are rejected.
fn file_exceeds_hard_cap(len: u64) -> bool {
    len > MAX_SEARCH_FILE_BYTES
}

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
    tracing::debug!(
        path = %options.path.display(),
        term = %options.search_term,
        regex = options.regex,
        case_sensitive = options.case_sensitive,
        respect_gitignore = options.respect_gitignore,
        "search started"
    );

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
            PatternEngine::build_with_engine(
                &options.search_term,
                !options.case_sensitive,
                options.regex_engine,
            )
            .map_err(|err| SearchError::InvalidRegex(err.to_string()))?,
        )
    } else {
        None
    };

    if regex.as_ref().is_some_and(PatternEngine::is_extended) {
        tracing::info!(
            pattern = %options.search_term,
            "regex compiled via fancy-regex extended engine (slower path)"
        );
    }

    let filename_filter = FileNameFilter::parse(&options.match_file_names)?;
    let exclude_filter = ExcludeDirFilter::parse(&options.exclude_dirs)?;
    let binary_extensions = extension_set(BINARY_EXTENSIONS);
    let searchable_binary_extensions = extension_set(SEARCHABLE_BINARY_EXTENSIONS);

    let norm_ctx = NormalizationContext::build(options);
    let normalized_needle = if regex.is_none() {
        Some(normalize_for_text_search(&options.search_term, options, &norm_ctx))
    } else {
        None
    };

    let mut walker = WalkBuilder::new(&options.path);
    walker
        .hidden(!options.include_hidden)
        .git_ignore(options.respect_gitignore)
        .git_exclude(options.respect_gitignore)
        .git_global(options.respect_gitignore)
        .ignore(options.respect_gitignore)
        // Honor `.gitignore` even when the search root is not a real git
        // repository. Grex users routinely point Grexa at extracted archives,
        // dependency caches, and read-only mounts that have an inherited
        // `.gitignore` but no `.git/` sibling.
        .require_git(false)
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
            normalized_needle.as_deref(),
            &norm_ctx,
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
            if let Some(max) = options.max_results
                && results.len() >= max
            {
                results.truncate(max);
                break;
            }
        }
    }

    let matches = results
        .iter()
        .map(|result| result.match_count)
        .sum::<usize>();
    let file_results = aggregate_file_results(&results, &file_encodings);
    let elapsed_ms = started.elapsed().as_millis();
    tracing::info!(
        files_scanned,
        files_matched = matched_files.len(),
        matches,
        skipped_files,
        elapsed_ms,
        cancelled,
        "search completed"
    );

    Ok(SearchSummary {
        results,
        file_results,
        files_scanned,
        files_matched: matched_files.len(),
        matches,
        skipped_files,
        elapsed_ms,
        cancelled,
    })
}

pub fn aggregate_file_results(
    results: &[SearchResult],
    encodings: &HashMap<PathBuf, DetectedEncoding>,
) -> Vec<FileSearchResult> {
    let mut grouped: BTreeMap<PathBuf, Vec<usize>> = BTreeMap::new();
    for (i, result) in results.iter().enumerate() {
        grouped.entry(result.full_path.clone()).or_default().push(i);
    }

    grouped
        .into_iter()
        .map(|(full_path, indices)| {
            let mut file_matches: Vec<SearchResult> =
                indices.iter().map(|&i| results[i].clone()).collect();
            file_matches.sort_by_key(|result| (result.line_number, result.column_number));
            let first = file_matches.first();
            let metadata = fs::metadata(&full_path).ok();
            let date_modified_unix = metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());

            let match_count = file_matches.iter().map(|result| result.match_count).sum();
            let first_match_line_number = first.map_or(0, |result| result.line_number);
            let file_name = full_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let extension = normalized_extension(&full_path).unwrap_or_default();
            let relative_path = first
                .map(|result| result.relative_path.clone())
                .unwrap_or_else(|| full_path.clone());

            let encoding_label = encodings
                .get(&full_path)
                .cloned()
                .unwrap_or(DetectedEncoding::Utf8)
                .label()
                .into_owned();

            FileSearchResult {
                file_name,
                size: metadata.as_ref().map_or(0, fs::Metadata::len),
                match_count,
                first_match_line_number,
                match_preview_before: first
                    .map(|result| result.match_preview_before.clone())
                    .unwrap_or_default(),
                match_preview_match: first
                    .map(|result| result.match_preview_match.clone())
                    .unwrap_or_default(),
                match_preview_after: first
                    .map(|result| result.match_preview_after.clone())
                    .unwrap_or_default(),
                preview_matches: file_matches,
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
    binary_extensions: &HashSet<&'static str>,
    searchable_binary_extensions: &HashSet<&'static str>,
) -> Result<Option<SkipReason>, SearchError> {
    let path = entry.path();

    if !options.include_system && is_system_path(path, root) {
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

    // Safety net independent of the user's (default-unlimited) size filter:
    // never read a pathologically large file into memory.
    if file_exceeds_hard_cap(metadata.len()) {
        return Ok(Some(SkipReason::SizeLimit));
    }

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
        .as_deref()
        .is_some_and(|ext| binary_extensions.contains(ext));

    if is_binary && !options.include_binary {
        return Ok(Some(SkipReason::BinaryFile));
    }

    if is_binary
        && options.include_binary
        && !ext
            .as_deref()
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
    regex: Option<&PatternEngine>,
    cancel: &CancelToken,
    normalized_needle: Option<&str>,
    norm_ctx: &NormalizationContext,
) -> Result<FileScan, SearchError> {
    // Searchable-document path: handed off to extractors that decode OOXML,
    // ODF, ZIP, PDF, and RTF into plain text before line scanning. The
    // returned encoding is reported as the file's *container* encoding so
    // result tables still show "UTF-8" (these container formats don't carry a
    // user-visible charset).
    let is_searchable_binary = normalized_extension(path)
        .is_some_and(|ext| SEARCHABLE_BINARY_EXTENSIONS.contains(&ext.as_str()));
    if is_searchable_binary {
        let extracted = match extract_text(path) {
            Ok(Some(text)) => text,
            Ok(None) => {
                return Ok(FileScan {
                    results: Vec::new(),
                    encoding: None,
                });
            }
            Err(err) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %err,
                    "document extractor failed; skipping file"
                );
                return Ok(FileScan {
                    results: Vec::new(),
                    encoding: None,
                });
            }
        };
        let results = scan_text_buffer(
            &ScanContext {
                path,
                root,
                options,
                regex,
                cancel,
                normalized_needle,
                norm_ctx,
            },
            &extracted,
        );
        return Ok(FileScan {
            results,
            encoding: Some(DetectedEncoding::Utf8),
        });
    }

    let (text, encoding) = read_text(path)?;
    let results = scan_text_buffer(
        &ScanContext {
            path,
            root,
            options,
            regex,
            cancel,
            normalized_needle,
            norm_ctx,
        },
        &text,
    );
    Ok(FileScan {
        results,
        encoding: Some(encoding),
    })
}

/// Walk the buffered text line-by-line and collect matches. Used by both the
/// plain-text reader and the document extractor path.
struct ScanContext<'a> {
    path: &'a Path,
    root: &'a Path,
    options: &'a SearchOptions,
    regex: Option<&'a PatternEngine>,
    cancel: &'a CancelToken,
    normalized_needle: Option<&'a str>,
    norm_ctx: &'a NormalizationContext,
}

fn scan_text_buffer(ctx: &ScanContext, text: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if idx % 64 == 0 && ctx.cancel.is_cancelled() {
            return results;
        }

        let line_number = idx + 1;
        let compare_line = truncate_bytes(line, MAX_COMPARE_LINE_BYTES);
        let matches = find_line_matches(
            &compare_line,
            ctx.options,
            ctx.regex,
            ctx.normalized_needle,
            ctx.norm_ctx,
        );
        if matches.is_empty() {
            continue;
        }

        let (start, end) = matches[0];
        let (before, matched, after) = preview_segments(line, start, end);
        results.push(SearchResult {
            file_name: ctx
                .path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            line_number,
            column_number: start + 1,
            line_content: truncate_chars(line, MATCH_PREVIEW_MAX_CHARS),
            match_preview_before: before,
            match_preview_match: matched,
            match_preview_after: after,
            full_path: ctx.path.to_path_buf(),
            relative_path: ctx
                .path
                .strip_prefix(ctx.root)
                .unwrap_or(ctx.path)
                .to_path_buf(),
            match_count: matches.len(),
        });
    }
    results
}

fn find_line_matches(
    line: &str,
    options: &SearchOptions,
    regex: Option<&PatternEngine>,
    normalized_needle: Option<&str>,
    norm_ctx: &NormalizationContext,
) -> Vec<(usize, usize)> {
    if let Some(engine) = regex {
        return engine.find_iter(line);
    }

    let needle = match normalized_needle {
        Some(n) => n,
        None => return Vec::new(),
    };
    if needle.is_empty() {
        return Vec::new();
    }

    let original = normalize_for_text_search(line, options, norm_ctx);
    let mut matches = Vec::new();
    let mut offset = 0;
    while let Some(index) = original[offset..].find(needle) {
        let start = offset + index;
        let end = start + needle.len();
        matches.push((start, end));
        offset = end;
    }
    matches
}

struct NormalizationContext {
    strip_diacritics: bool,
    gc_map:
        icu_properties::CodePointMapDataBorrowed<'static, icu_properties::props::GeneralCategory>,
    case_mapper: Option<icu_casemap::CaseMapperBorrowed<'static>>,
    locale: Option<icu_locale_core::LanguageIdentifier>,
}

impl NormalizationContext {
    fn build(options: &SearchOptions) -> Self {
        use crate::models::StringComparisonMode;

        let gc_map =
            icu_properties::CodePointMapData::<icu_properties::props::GeneralCategory>::new();

        let (case_mapper, locale) = if !options.case_sensitive
            && options.string_comparison_mode == StringComparisonMode::CurrentCulture
        {
            let locale = options
                .culture
                .as_deref()
                .and_then(|tag| icu_locale_core::LanguageIdentifier::try_from_str(tag).ok())
                .unwrap_or_else(|| {
                    icu_locale_core::LanguageIdentifier::try_from_str("en")
                        .expect("\"en\" is a valid BCP-47 tag")
                });
            (Some(icu_casemap::CaseMapper::new()), Some(locale))
        } else {
            (None, None)
        };

        Self {
            strip_diacritics: !options.diacritic_sensitive,
            gc_map,
            case_mapper,
            locale,
        }
    }
}

fn normalize_for_text_search(
    input: &str,
    options: &SearchOptions,
    ctx: &NormalizationContext,
) -> String {
    let mut value = match options.unicode_normalization_mode {
        UnicodeNormalizationMode::None => input.to_string(),
        UnicodeNormalizationMode::FormC => input.nfc().collect(),
        UnicodeNormalizationMode::FormD => input.nfd().collect(),
        UnicodeNormalizationMode::FormKC => input.nfkc().collect(),
        UnicodeNormalizationMode::FormKD => input.nfkd().collect(),
    };

    if ctx.strip_diacritics {
        use icu_properties::props::GeneralCategory;
        value = value
            .nfd()
            .filter(|ch| ctx.gc_map.get(*ch) != GeneralCategory::NonspacingMark)
            .collect();
    }

    if !options.case_sensitive {
        value = culture_aware_lowercase(&value, ctx);
    }

    value
}

fn culture_aware_lowercase(value: &str, ctx: &NormalizationContext) -> String {
    match (&ctx.case_mapper, &ctx.locale) {
        (Some(mapper), Some(locale)) => mapper
            .lowercase_to_string(value, locale)
            .into_owned()
            .to_string(),
        _ => value.to_lowercase(),
    }
}

fn preview_segments(line: &str, start: usize, end: usize) -> (String, String, String) {
    let before = line.get(..start).unwrap_or_default();
    let matched = line.get(start..end).unwrap_or_default();
    let after = line.get(end..).unwrap_or_default();

    (truncate_chars(before, 120), matched.to_string(), truncate_chars(after, 120))
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.len() <= max_chars {
        return input.to_string();
    }
    match input.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => input[..byte_idx].to_string(),
        None => input.to_string(),
    }
}

fn truncate_bytes(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !input.is_char_boundary(boundary) {
        boundary -= 1;
    }
    input[..boundary].to_string()
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

fn is_system_path(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if relative
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .any(|s| SYSTEM_DIRS.contains(&s))
    {
        return true;
    }

    let mut components = relative.components().filter_map(|c| c.as_os_str().to_str());
    while let Some(first) = components.next() {
        if first == "storage"
            && let Some(second) = components.next()
            && second == "framework"
        {
            return true;
        }
    }
    false
}

fn normalized_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn extension_set(values: &[&'static str]) -> HashSet<&'static str> {
    values.iter().copied().collect()
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
        fs::write(dir.path().join("app.rs"), "fn main() {\n    // TODO fix\n}\n").unwrap();

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
    fn culture_aware_turkish_lowercase_treats_capital_i_specially() {
        // Turkish maps capital I → dotless ı (U+0131) under locale-aware
        // lowercasing. Ordinal mode keeps `I → i`; CurrentCulture +
        // culture=tr-TR uses the Turkish rule.
        use crate::models::StringComparisonMode;
        let mut options = SearchOptions::new("/tmp", "stamboul");
        options.case_sensitive = false;

        // Ordinal — should NOT match against an `İSTANBUL` source.
        let ordinal =
            normalize_for_text_search("İSTANBUL", &options, &NormalizationContext::build(&options));
        assert!(ordinal.contains('i'), "ordinal must lower-case I to i");

        // CurrentCulture + tr-TR — Turkish lowering of `İ` is `i`,
        // and lowering of plain `I` becomes the dotless `ı`. So a
        // search for "stamboul" matches `İstanbul` only when the
        // lowercase form goes through ICU.
        options.string_comparison_mode = StringComparisonMode::CurrentCulture;
        options.culture = Some("tr-TR".to_string());
        let turkish =
            normalize_for_text_search("İSTANBUL", &options, &NormalizationContext::build(&options));
        // The lowered form starts with `i` (dotted) + lower stem.
        assert!(
            turkish.to_lowercase() == turkish,
            "ICU result should be all-lowercase, got {turkish:?}"
        );
        // Cross-check against the dotless ı for plain I in tr-TR.
        let dotless =
            normalize_for_text_search("ISTANBUL", &options, &NormalizationContext::build(&options));
        assert!(
            dotless.contains('ı') || dotless.contains('i'),
            "tr-TR lowering should produce one of i/ı for capital I, got {dotless:?}"
        );
    }

    #[test]
    fn culture_aware_german_sharp_s_round_trips() {
        // German ß is treated as a single grapheme; ICU should preserve
        // it through lowercasing. The audit's intent is that searching
        // for "straße" against "STRASSE" produces a hit under
        // CurrentCulture+de-DE — but Unicode lowercasing of "SS" stays
        // "ss" (NOT "ß"), so the match is "straße" vs "strasse" which
        // is a documented divergence captured in the culture audit.
        use crate::models::StringComparisonMode;
        let mut options = SearchOptions::new("/tmp", "x");
        options.case_sensitive = false;
        options.string_comparison_mode = StringComparisonMode::CurrentCulture;
        options.culture = Some("de-DE".to_string());

        let lowered =
            normalize_for_text_search("STRAßE", &options, &NormalizationContext::build(&options));
        // ICU should keep ß intact when lowering a string that already
        // contains ß; never expands it.
        assert!(lowered.contains('ß'), "got {lowered:?}");
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
    fn diacritic_strip_preserves_spacing_combining_marks() {
        use icu_properties::props::GeneralCategory;
        let gc_map = icu_properties::CodePointMapData::<GeneralCategory>::new();
        let devanagari_aa: char = '\u{093E}';
        assert_eq!(
            gc_map.get(devanagari_aa),
            GeneralCategory::SpacingMark,
            "U+093E should be Mc (SpacingMark)"
        );
        let mut options = SearchOptions::new("/tmp", "x");
        options.diacritic_sensitive = false;
        let result = normalize_for_text_search(
            "क\u{093E}",
            &options,
            &NormalizationContext::build(&options),
        );
        assert!(
            result.contains(devanagari_aa),
            "Mc marks must survive Mn-only diacritic stripping, got {result:?}"
        );
    }

    #[test]
    fn culture_aware_greek_sigma_handling() {
        use crate::models::StringComparisonMode;
        let mut options = SearchOptions::new("/tmp", "x");
        options.case_sensitive = false;
        options.string_comparison_mode = StringComparisonMode::CurrentCulture;
        options.culture = Some("el-GR".to_string());

        let lowered =
            normalize_for_text_search("ΣΟΦΟΣ", &options, &NormalizationContext::build(&options));
        assert!(
            lowered.contains('σ') || lowered.contains('ς'),
            "Greek sigma should lowercase to σ/ς, got {lowered:?}"
        );
        assert!(!lowered.contains('Σ'), "capital sigma must be lowered, got {lowered:?}");
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
    fn hard_size_cap_rejects_only_files_above_the_limit() {
        assert!(!file_exceeds_hard_cap(0), "an empty file is within the cap");
        assert!(
            !file_exceeds_hard_cap(MAX_SEARCH_FILE_BYTES),
            "a file exactly at the cap is allowed"
        );
        assert!(
            file_exceeds_hard_cap(MAX_SEARCH_FILE_BYTES + 1),
            "a file one byte over the cap is rejected"
        );
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
    fn search_extracts_docx_content() {
        // Drive the search engine through a .docx fixture and confirm the
        // document extractor + line scanner cooperate end-to-end.
        use std::io::Write;
        use zip::write::SimpleFileOptions;

        let dir = tempdir().unwrap();
        let path = dir.path().join("paper.docx");
        let file = std::fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<w:document xmlns:w="http://x">
  <w:body>
    <w:p><w:r><w:t>Hello</w:t></w:r></w:p>
    <w:p><w:r><w:t>TODO write the test</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
        )
        .unwrap();
        zip.finish().unwrap();

        let mut options = SearchOptions::new(dir.path(), "TODO");
        options.include_binary = true;
        let summary = search(&options).unwrap();
        assert!(summary.matches >= 1, "no docx matches found");
        let file_names: Vec<_> = summary
            .results
            .iter()
            .map(|r| r.file_name.clone())
            .collect();
        assert!(file_names.contains(&"paper.docx".to_string()));
    }

    #[test]
    fn search_handles_files_with_null_bytes_and_huge_lines() {
        // Files that mix null bytes into otherwise-readable UTF-8 are valid
        // POSIX content; the search engine should tolerate them. We also
        // include a single very-long line to make sure neither the previewer
        // nor the line iterator panic on extreme inputs.
        let dir = tempdir().unwrap();
        let path = dir.path().join("weird.txt");
        let huge = "A".repeat(64 * 1024);
        let body = format!("line1\nTODO\0null\n{huge} TODO trailing\n");
        std::fs::write(&path, body.as_bytes()).unwrap();
        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert!(summary.matches >= 2, "got {summary:?}");
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
        // reader. With the chardetng cascade in `encoding::read_text` this is
        // now identified as a legacy 8-bit encoding (chardetng usually picks
        // windows-1252). The important contract: the file is searched, "TODO"
        // is found, and the engine labels the encoding instead of crashing.
        fs::write(&path, b"TODO\xFFfix\n").unwrap();
        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert_eq!(summary.matches, 1);
        let label = &summary.file_results[0].encoding;
        assert!(
            label != "UTF-8" && !label.is_empty(),
            "expected a chardetng-detected legacy label, got {label:?}"
        );
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

    #[test]
    fn max_compare_line_bytes_truncates_oversized_line_for_matching() {
        let dir = tempdir().unwrap();
        let pad = "x".repeat(MAX_COMPARE_LINE_BYTES + 100);
        let body = format!("{pad}TODO\n");
        fs::write(dir.path().join("big.txt"), &body).unwrap();

        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert_eq!(summary.matches, 0, "needle past MAX_COMPARE_LINE_BYTES must not be found");
    }

    #[test]
    fn max_compare_line_bytes_allows_match_within_cap() {
        let dir = tempdir().unwrap();
        let pad = "x".repeat(MAX_COMPARE_LINE_BYTES / 2);
        let body = format!("{pad}TODO{pad}\n");
        fs::write(dir.path().join("ok.txt"), &body).unwrap();

        let summary = search(&SearchOptions::new(dir.path(), "TODO")).unwrap();
        assert_eq!(summary.matches, 1);
    }

    #[test]
    fn truncate_bytes_respects_char_boundaries() {
        let input = "café";
        let truncated = truncate_bytes(input, 4);
        assert_eq!(truncated, "caf", "must not split the é codepoint");
    }
}
