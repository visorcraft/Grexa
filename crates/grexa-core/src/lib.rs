// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

pub mod baloo;
pub mod cancel;
pub mod constants;
pub mod desktop;
pub mod documents;
pub mod encoding;
pub mod models;
pub mod pattern;
pub mod preview;
pub mod replace;
pub mod search;
pub mod sort;
pub mod storage;

pub use baloo::{
    BalooAdapter, BalooError, BaloosearchCliAdapter, NullBalooAdapter, StubBalooAdapter,
};
pub use cancel::CancelToken;
pub use constants::{MAX_SEARCH_FILE_BYTES, file_exceeds_hard_cap};
pub use desktop::{
    EditorPreset, TrashError, UserPathKind, classify_user_path, file_manager_show_items_uris,
    move_to_trash, open_in_editor_command, reveal_with_xdg_open,
};
pub use documents::{ExtractError, extract_text};
pub use encoding::{
    DetectedEncoding, detect_from_bytes, detect_from_path, heuristic_label, read_text,
};
pub use models::{
    FileSearchResult, OutputFormat, RegexEngine, SearchOptions, SearchResult,
    SearchResultSortField, SearchSummary, SizeLimitType, SizeUnit, StringComparisonMode,
    UnicodeNormalizationMode,
};
pub use pattern::{PatternEngine, PatternError};
pub use preview::{ContextLine, ContextPreviewResult, PreviewError, context_preview};
pub use replace::{
    FileReplaceFailure, FileReplaceReport, ReplaceError, ReplaceJournalEntry, ReplaceOptions,
    ReplaceSummary, load_residual_journal, replace_file, replace_with, set_journal_path_override,
};
pub use search::{ProgressEvent, ProgressSink, SearchError, SkipReason, search, search_with};
pub use sort::{SortDirection, apply_default_sort, sort_content, sort_files};
pub use storage::{
    AppPaths, DefaultSettings, ImportError, JsonStoreError, RecentPathStore, RecentSearch,
    SearchHistoryStore, SearchProfile, SearchProfileStore, SettingsStore, ThemePreference,
};
