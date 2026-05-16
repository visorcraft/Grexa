pub mod cancel;
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

pub use cancel::CancelToken;
pub use encoding::{DetectedEncoding, detect_from_bytes, detect_from_path, heuristic_label, read_text};
pub use pattern::{PatternEngine, PatternError};
pub use desktop::{
    EditorPreset, file_manager_show_items_uris, open_in_editor_command, reveal_with_xdg_open,
};
pub use documents::{ExtractError, extract_text};
pub use preview::{ContextLine, ContextPreviewResult, PreviewError, context_preview};
pub use replace::{
    FileReplaceFailure, FileReplaceReport, ReplaceError, ReplaceOptions, ReplaceSummary,
    replace_with,
};
pub use models::{
    FileSearchResult, OutputFormat, SearchOptions, SearchResult, SearchResultSortField,
    SearchSummary, SizeLimitType, SizeUnit, StringComparisonMode, UnicodeNormalizationMode,
};
pub use search::{ProgressEvent, ProgressSink, SearchError, SkipReason, search, search_with};
pub use sort::{SortDirection, apply_default_sort, sort_content, sort_files};
pub use storage::{
    AppPaths, DefaultSettings, ImportError, JsonStoreError, RecentPathStore, RecentSearch,
    SearchHistoryStore, SearchProfile, SearchProfileStore, SettingsStore, ThemePreference,
};
