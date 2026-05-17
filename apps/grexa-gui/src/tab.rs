//! Per-tab state.
//!
//! Grex's `TabViewModel` (audited in `docs/grex-tab-viewmodel-audit.md`)
//! holds one search's UI state: query, options, results, sort, filtered
//! view, replacement input, AI mode, status text. Grexa replicates the
//! same shape in pure Rust so the QML side is a thin observer.

use std::path::PathBuf;

use grexa_core::{
    CancelToken, FileSearchResult, ProgressEvent, SearchOptions, SearchResult,
    SearchResultSortField, SearchSummary, SortDirection, apply_default_sort, sort_content,
    sort_files,
};

/// Internal tab id. The controller hands these out monotonically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

/// What the result list is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultMode {
    Content,
    Files,
}

/// Per-tab lifecycle status. Variants are GUI-bound; the type lives in
/// this binary's tree so `dead_code` is silenced for unused variants.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabStatus {
    Idle,
    Searching,
    Replacing,
    Cancelled,
    Completed,
    Error(String),
}

/// Sorted/filtered view of the canonical result list. Stored alongside the
/// raw `SearchSummary` so toggling sort or the search-within filter never
/// loses the original data. Fields are public so the QML view model can
/// read them once the cxx-qt bindings land; `dead_code` is allowed
/// until that wiring exists.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ResultView {
    pub content: Vec<SearchResult>,
    pub files: Vec<FileSearchResult>,
    /// True when the lists above are filtered down from the raw summary.
    pub is_filtered: bool,
    /// Total raw rows before filtering — used by the status bar's
    /// "showing X of Y" message.
    pub raw_total_content: usize,
    pub raw_total_files: usize,
}

/// Single Search-tab state object. Equivalent to Grex `TabViewModel`.
/// Many fields and methods are bound by the QML side that hasn't landed
/// yet; the `dead_code` allow lets the host crate compile clean while
/// the controller surface stays stable for the cxx-qt iteration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TabState {
    pub id: TabId,
    pub title: String,
    pub options: SearchOptions,
    pub replacement: String,
    pub result_mode: ResultMode,
    pub status: TabStatus,
    pub summary: Option<SearchSummary>,
    pub view: ResultView,
    /// Search-within-results filter — applied on top of `summary`.
    pub within_filter: String,
    pub within_regex: bool,
    /// Current sort field and direction.
    pub sort_field: SearchResultSortField,
    pub sort_direction: SortDirection,
    /// True when AI Search panel is active for this tab. AI mode hides
    /// the result tables, matching Grex.
    pub ai_mode: bool,
    /// Cancellation handle for the in-flight search.
    pub cancel: CancelToken,
}

#[allow(dead_code)]
impl TabState {
    pub fn new(id: TabId, options: SearchOptions) -> Self {
        let title = derive_title(&options);
        Self {
            id,
            title,
            options,
            replacement: String::new(),
            result_mode: ResultMode::Content,
            status: TabStatus::Idle,
            summary: None,
            view: ResultView::default(),
            within_filter: String::new(),
            within_regex: false,
            sort_field: SearchResultSortField::Name,
            sort_direction: SortDirection::Ascending,
            ai_mode: false,
            cancel: CancelToken::new(),
        }
    }

    /// Apply the result of a completed search to this tab's view.
    /// Rebuilds the sorted/filtered display.
    pub fn install_summary(&mut self, summary: SearchSummary) {
        self.summary = Some(summary);
        self.status = if self.summary.as_ref().map(|s| s.cancelled).unwrap_or(false) {
            TabStatus::Cancelled
        } else {
            TabStatus::Completed
        };
        self.rebuild_view();
    }

    /// Re-derive `view` from `summary` + `within_filter`.
    pub fn rebuild_view(&mut self) {
        let Some(summary) = &self.summary else {
            self.view = ResultView::default();
            return;
        };
        let raw_total_content = summary.results.len();
        let raw_total_files = summary.file_results.len();
        let (mut content, mut files) = if self.within_filter.trim().is_empty() {
            (summary.results.clone(), summary.file_results.clone())
        } else {
            let needle_lower = self.within_filter.to_lowercase();
            let predicate = |line: &str| {
                if self.within_regex {
                    regex::RegexBuilder::new(&self.within_filter)
                        .case_insensitive(true)
                        .build()
                        .map(|re| re.is_match(line))
                        .unwrap_or(false)
                } else {
                    line.to_lowercase().contains(&needle_lower)
                }
            };
            let content: Vec<_> = summary
                .results
                .iter()
                .filter(|r| predicate(&r.line_content))
                .cloned()
                .collect();
            let files: Vec<_> = summary
                .file_results
                .iter()
                .filter(|f| f.preview_matches.iter().any(|r| predicate(&r.line_content)))
                .cloned()
                .collect();
            (content, files)
        };

        // Apply the current sort.
        if self.sort_field == SearchResultSortField::None {
            apply_default_sort(&mut content, &mut files);
        } else {
            sort_content(&mut content, self.sort_field, self.sort_direction);
            sort_files(&mut files, self.sort_field, self.sort_direction);
        }

        self.view = ResultView {
            content,
            files,
            is_filtered: !self.within_filter.trim().is_empty(),
            raw_total_content,
            raw_total_files,
        };
    }

    /// Toggle direction when sorting by the active field, otherwise switch
    /// field and reset to ascending. Mirrors Grex's `SortResults` toggle
    /// behavior captured in `docs/grex-tab-viewmodel-audit.md`.
    pub fn apply_sort(&mut self, field: SearchResultSortField) {
        if field == self.sort_field {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_direction = SortDirection::Ascending;
        }
        self.rebuild_view();
    }

    /// Set the search-within-results filter and recompute.
    pub fn set_within_filter(&mut self, value: impl Into<String>, regex: bool) {
        self.within_filter = value.into();
        self.within_regex = regex;
        self.rebuild_view();
    }

    /// Update path and re-derive the auto-title.
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.options.path = path.into();
        self.title = derive_title(&self.options);
    }

    /// Switch into AI mode and clear the result-table-side state, matching
    /// Grex's behavior of hiding result grids while the AI chat is active.
    pub fn enable_ai_mode(&mut self) {
        self.ai_mode = true;
        self.within_filter.clear();
        self.rebuild_view();
    }

    pub fn disable_ai_mode(&mut self) {
        self.ai_mode = false;
    }

    /// Apply a streaming `ProgressEvent` to the tab. Today this only
    /// updates the `Searching` status counter; the full table-model
    /// streaming integration is the GUI's job.
    pub fn observe_progress(&mut self, _event: &ProgressEvent) {
        if matches!(self.status, TabStatus::Idle | TabStatus::Completed) {
            self.status = TabStatus::Searching;
        }
    }
}

/// Auto-title rule from `docs/grex-tab-viewmodel-audit.md`:
///
/// - blank path → "New tab"
/// - ≤ 30 chars → use the trailing path component
/// - longer → `<first>/.../<last>`; preserve `\\server` on UNC inputs
fn derive_title(options: &SearchOptions) -> String {
    let path = options.path.to_string_lossy();
    if path.trim().is_empty() {
        return "New tab".to_string();
    }
    if path.len() <= 30 {
        return options
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.into_owned());
    }
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() <= 2 {
        return path.into_owned();
    }
    let first = parts.first().copied().unwrap_or_default();
    let last = parts.last().copied().unwrap_or_default();
    format!("{first}/.../{last}")
}

#[cfg(test)]
mod tests {
    use grexa_core::SearchOptions;

    use super::*;

    fn tab(path: &str, term: &str) -> TabState {
        TabState::new(TabId(1), SearchOptions::new(path, term))
    }

    #[test]
    fn auto_title_uses_last_component_for_short_paths() {
        let state = tab("/home/me/code", "TODO");
        assert_eq!(state.title, "code");
    }

    #[test]
    fn auto_title_collapses_long_paths() {
        let state = tab("/very/deep/nested/path/with/many/segments/that/keeps/going/code", "x");
        assert!(state.title.contains("/.../"));
    }

    #[test]
    fn auto_title_blank_path_is_new_tab() {
        let state = tab("", "x");
        assert_eq!(state.title, "New tab");
    }

    #[test]
    fn install_summary_marks_completed() {
        let mut state = tab("/tmp", "TODO");
        let summary = SearchSummary {
            results: vec![],
            file_results: vec![],
            files_scanned: 0,
            files_matched: 0,
            matches: 0,
            skipped_files: 0,
            elapsed_ms: 12,
            cancelled: false,
        };
        state.install_summary(summary);
        assert_eq!(state.status, TabStatus::Completed);
    }

    #[test]
    fn install_summary_with_cancel_marks_cancelled() {
        let mut state = tab("/tmp", "TODO");
        let summary = SearchSummary {
            results: vec![],
            file_results: vec![],
            files_scanned: 0,
            files_matched: 0,
            matches: 0,
            skipped_files: 0,
            elapsed_ms: 5,
            cancelled: true,
        };
        state.install_summary(summary);
        assert_eq!(state.status, TabStatus::Cancelled);
    }

    #[test]
    fn within_filter_narrows_content_view() {
        let mut state = tab("/tmp", "TODO");
        state.summary = Some(SearchSummary {
            results: vec![
                make_result("a.txt", "TODO write tests"),
                make_result("b.txt", "TODO ship release"),
                make_result("c.txt", "unrelated"),
            ],
            file_results: vec![],
            files_scanned: 3,
            files_matched: 3,
            matches: 3,
            skipped_files: 0,
            elapsed_ms: 1,
            cancelled: false,
        });
        state.set_within_filter("write", false);
        assert_eq!(state.view.content.len(), 1);
        assert!(state.view.is_filtered);
        assert_eq!(state.view.raw_total_content, 3);
    }

    #[test]
    fn apply_sort_toggles_direction_on_repeat() {
        let mut state = tab("/tmp", "TODO");
        state.sort_field = SearchResultSortField::Name;
        state.sort_direction = SortDirection::Ascending;
        state.apply_sort(SearchResultSortField::Name);
        assert_eq!(state.sort_direction, SortDirection::Descending);
        state.apply_sort(SearchResultSortField::Name);
        assert_eq!(state.sort_direction, SortDirection::Ascending);
    }

    #[test]
    fn apply_sort_resets_direction_when_field_changes() {
        let mut state = tab("/tmp", "TODO");
        state.sort_field = SearchResultSortField::Name;
        state.sort_direction = SortDirection::Descending;
        state.apply_sort(SearchResultSortField::Line);
        assert_eq!(state.sort_field, SearchResultSortField::Line);
        assert_eq!(state.sort_direction, SortDirection::Ascending);
    }

    #[test]
    fn enable_ai_mode_clears_filter() {
        let mut state = tab("/tmp", "TODO");
        state.within_filter = "anything".to_string();
        state.enable_ai_mode();
        assert!(state.ai_mode);
        assert!(state.within_filter.is_empty());
    }

    fn make_result(name: &str, line: &str) -> SearchResult {
        SearchResult {
            file_name: name.to_string(),
            line_number: 1,
            column_number: 1,
            line_content: line.to_string(),
            match_preview_before: String::new(),
            match_preview_match: String::new(),
            match_preview_after: String::new(),
            full_path: PathBuf::from(name),
            relative_path: PathBuf::from(name),
            match_count: 1,
        }
    }
}
