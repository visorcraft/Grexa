// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeLimitType {
    NoLimit,
    LessThan,
    EqualTo,
    GreaterThan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeUnit {
    KB,
    MB,
    GB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringComparisonMode {
    Ordinal,
    CurrentCulture,
    InvariantCulture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeNormalizationMode {
    None,
    FormC,
    FormD,
    FormKC,
    FormKD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchResultSortField {
    None,
    Name,
    Line,
    Column,
    Path,
    Extension,
    Encoding,
    Matches,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchOptions {
    pub path: PathBuf,
    pub search_term: String,
    pub regex: bool,
    pub case_sensitive: bool,
    pub respect_gitignore: bool,
    pub include_hidden: bool,
    pub include_binary: bool,
    pub include_system: bool,
    pub include_subfolders: bool,
    pub include_symlinks: bool,
    pub match_file_names: String,
    pub exclude_dirs: String,
    pub size_limit_type: SizeLimitType,
    pub size_limit_kb: Option<u64>,
    pub size_unit: SizeUnit,
    pub string_comparison_mode: StringComparisonMode,
    pub unicode_normalization_mode: UnicodeNormalizationMode,
    pub diacritic_sensitive: bool,
    pub culture: Option<String>,
    pub use_file_index: bool,
}

impl SearchOptions {
    pub fn new(path: impl Into<PathBuf>, search_term: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            search_term: search_term.into(),
            regex: false,
            case_sensitive: false,
            respect_gitignore: false,
            include_hidden: false,
            include_binary: false,
            include_system: false,
            include_subfolders: true,
            include_symlinks: false,
            match_file_names: String::new(),
            exclude_dirs: String::new(),
            size_limit_type: SizeLimitType::NoLimit,
            size_limit_kb: None,
            size_unit: SizeUnit::KB,
            string_comparison_mode: StringComparisonMode::Ordinal,
            unicode_normalization_mode: UnicodeNormalizationMode::None,
            diacritic_sensitive: true,
            culture: None,
            use_file_index: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_name: String,
    pub line_number: usize,
    pub column_number: usize,
    pub line_content: String,
    pub match_preview_before: String,
    pub match_preview_match: String,
    pub match_preview_after: String,
    pub full_path: PathBuf,
    pub relative_path: PathBuf,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub file_name: String,
    pub size: u64,
    pub match_count: usize,
    pub first_match_line_number: usize,
    pub match_preview_before: String,
    pub match_preview_match: String,
    pub match_preview_after: String,
    pub preview_matches: Vec<SearchResult>,
    pub full_path: PathBuf,
    pub relative_path: PathBuf,
    pub extension: String,
    pub encoding: String,
    pub date_modified_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSummary {
    pub results: Vec<SearchResult>,
    pub file_results: Vec<FileSearchResult>,
    pub files_scanned: usize,
    pub files_matched: usize,
    pub matches: usize,
    pub skipped_files: usize,
    pub elapsed_ms: u128,
    #[serde(default)]
    pub cancelled: bool,
}

impl SearchSummary {
    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(self.elapsed_ms as u64)
    }
}
