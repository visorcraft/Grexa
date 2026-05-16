pub mod models;
pub mod search;
pub mod storage;

pub use models::{
    FileSearchResult, OutputFormat, SearchOptions, SearchResult, SearchResultSortField,
    SearchSummary, SizeLimitType, SizeUnit, StringComparisonMode, UnicodeNormalizationMode,
};
pub use search::{SearchError, search};
pub use storage::{
    AppPaths, DefaultSettings, JsonStoreError, RecentPathStore, RecentSearch, SearchHistoryStore,
    SearchProfile, SearchProfileStore, SettingsStore,
};
