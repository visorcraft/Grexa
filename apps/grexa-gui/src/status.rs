//! Status-text formatter.
//!
//! Mirrors `docs/grex-status-text-audit.md`: every status string flows
//! through Fluent so plurals are correct in every locale. The formatter
//! lives Rust-side so the QML view binds to a simple `String`. The QML
//! binding lands with the cxx-qt iteration; until then the
//! `format_status` entry point is reachable only from tests, hence the
//! crate-wide `dead_code` allow on its helpers.

#![allow(dead_code)]

use grexa_core::SearchSummary;
use grexa_i18n::{Bundle, Value};

use crate::tab::TabState;
use crate::tab::TabStatus;

pub fn format_status(bundle: &Bundle, tab: &TabState) -> String {
    match &tab.status {
        TabStatus::Idle => bundle
            .t("search-status-ready")
            .unwrap_or_else(|_| "Ready".into()),
        TabStatus::Searching => bundle
            .t("search-status-running")
            .unwrap_or_else(|_| "Searching…".into()),
        TabStatus::Replacing => bundle
            .t("replace-status-running")
            .unwrap_or_else(|_| "Replacing…".into()),
        TabStatus::Cancelled => bundle
            .t("search-status-cancelled")
            .unwrap_or_else(|_| "Cancelled".into()),
        TabStatus::Error(message) => bundle
            .format("search-status-error", &[("message", message.clone().into())])
            .unwrap_or_else(|_| format!("Error: {message}")),
        TabStatus::Completed => match &tab.summary {
            Some(summary) if tab.view.is_filtered => format_filtered_summary(bundle, summary, tab),
            Some(summary) => format_completed_summary(bundle, summary),
            None => bundle
                .t("search-status-ready")
                .unwrap_or_else(|_| "Ready".into()),
        },
    }
}

fn format_completed_summary(bundle: &Bundle, summary: &SearchSummary) -> String {
    let elapsed = format_elapsed(bundle, summary.elapsed_ms);
    bundle
        .format(
            "search-status-found",
            &[
                ("matches", count(summary.matches)),
                ("files", count(summary.files_matched)),
                ("elapsed", elapsed.into()),
            ],
        )
        .unwrap_or_else(|_| {
            format!("Found {} matches in {} files", summary.matches, summary.files_matched)
        })
}

fn format_filtered_summary(bundle: &Bundle, summary: &SearchSummary, tab: &TabState) -> String {
    bundle
        .format(
            "search-status-filtered",
            &[
                ("shown", count(tab.view.content.len())),
                ("total", count(summary.matches)),
                ("files", count(summary.files_matched)),
            ],
        )
        .unwrap_or_else(|_| {
            format!(
                "Showing {} of {} matches in {} files",
                tab.view.content.len(),
                summary.matches,
                summary.files_matched
            )
        })
}

fn format_elapsed(bundle: &Bundle, elapsed_ms: u128) -> String {
    let seconds_total = elapsed_ms as f64 / 1000.0;
    if seconds_total < 1.0 {
        return bundle
            .t("elapsed-subsecond")
            .unwrap_or_else(|_| "under a second".into());
    }
    if seconds_total < 60.0 {
        return bundle
            .format("elapsed-seconds", &[("seconds", count(seconds_total.round() as usize))])
            .unwrap_or_else(|_| format!("{:.0} seconds", seconds_total));
    }
    let minutes = (seconds_total / 60.0) as usize;
    let remaining_seconds = (seconds_total - (minutes as f64 * 60.0)).round() as usize;
    if remaining_seconds == 0 {
        return bundle
            .format("elapsed-minutes-only", &[("minutes", count(minutes))])
            .unwrap_or_else(|_| format!("{minutes} minutes"));
    }
    bundle
        .format(
            "elapsed-minutes-and-seconds",
            &[
                ("minutes", count(minutes)),
                ("seconds", count(remaining_seconds)),
            ],
        )
        .unwrap_or_else(|_| format!("{minutes} minutes and {remaining_seconds} seconds"))
}

fn count(value: usize) -> Value<'static> {
    Value::from(value as f64)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use grexa_core::{SearchOptions, SearchSummary};
    use grexa_i18n::{Bundle, Locale};

    use super::*;
    use crate::tab::{TabId, TabState};

    fn english() -> Bundle {
        Bundle::for_locale(Locale::English).unwrap()
    }

    fn tab_with_summary(summary: SearchSummary) -> TabState {
        let mut tab = TabState::new(TabId(1), SearchOptions::new("/tmp", "TODO"));
        tab.status = TabStatus::Completed;
        tab.summary = Some(summary);
        tab.view.is_filtered = false;
        tab
    }

    #[test]
    fn status_ready_for_idle_tab() {
        let bundle = english();
        let tab = TabState::new(TabId(1), SearchOptions::new("/tmp", "TODO"));
        assert_eq!(format_status(&bundle, &tab), "Ready");
    }

    #[test]
    fn status_running_for_searching_tab() {
        let bundle = english();
        let mut tab = TabState::new(TabId(1), SearchOptions::new("/tmp", "TODO"));
        tab.status = TabStatus::Searching;
        assert_eq!(format_status(&bundle, &tab), "Searching…");
    }

    #[test]
    fn status_completed_with_singular_plural_match() {
        let bundle = english();
        let tab = tab_with_summary(SearchSummary {
            results: vec![],
            file_results: vec![],
            files_scanned: 1,
            files_matched: 1,
            matches: 1,
            skipped_files: 0,
            elapsed_ms: 250,
            cancelled: false,
        });
        let out = format_status(&bundle, &tab);
        assert!(out.contains("Found 1 match"), "got {out:?}");
        assert!(out.contains("1 file"));
        assert!(out.contains("under a second"));
    }

    #[test]
    fn status_completed_with_plural_match() {
        let bundle = english();
        let tab = tab_with_summary(SearchSummary {
            results: vec![],
            file_results: vec![],
            files_scanned: 7,
            files_matched: 7,
            matches: 42,
            skipped_files: 0,
            elapsed_ms: 1200,
            cancelled: false,
        });
        let out = format_status(&bundle, &tab);
        assert!(out.contains("Found 42 matches"));
        assert!(out.contains("7 files"));
        assert!(out.contains("1 second"));
    }

    #[test]
    fn status_completed_three_minutes_and_five_seconds() {
        let bundle = english();
        let tab = tab_with_summary(SearchSummary {
            results: vec![],
            file_results: vec![],
            files_scanned: 1000,
            files_matched: 500,
            matches: 1000,
            skipped_files: 0,
            elapsed_ms: 185_000,
            cancelled: false,
        });
        let out = format_status(&bundle, &tab);
        assert!(out.contains("3 minutes"));
        assert!(out.contains("5 seconds"));
    }

    #[test]
    fn status_cancelled() {
        let bundle = english();
        let mut tab = TabState::new(TabId(1), SearchOptions::new("/tmp", "TODO"));
        tab.status = TabStatus::Cancelled;
        assert_eq!(format_status(&bundle, &tab), "Cancelled");
    }

    #[test]
    fn status_error_carries_message() {
        let bundle = english();
        let mut tab = TabState::new(TabId(1), SearchOptions::new("/tmp", "TODO"));
        tab.status = TabStatus::Error("permission denied".into());
        let out = format_status(&bundle, &tab);
        assert!(out.contains("Error"));
        assert!(out.contains("permission denied"));
    }

    #[test]
    fn status_filtered_shows_shown_of_total() {
        let bundle = english();
        let summary = SearchSummary {
            results: (0..50).map(make_result).collect(),
            file_results: vec![],
            files_scanned: 1,
            files_matched: 1,
            matches: 50,
            skipped_files: 0,
            elapsed_ms: 50,
            cancelled: false,
        };
        let mut tab = tab_with_summary(summary);
        tab.view.is_filtered = true;
        tab.view.content = (0..12).map(make_result).collect();
        let out = format_status(&bundle, &tab);
        assert!(out.contains("12"), "got {out:?}");
        assert!(out.contains("50"));
    }

    fn make_result(i: usize) -> grexa_core::SearchResult {
        grexa_core::SearchResult {
            file_name: format!("f{i}.txt"),
            line_number: i + 1,
            column_number: 1,
            line_content: String::new(),
            match_preview_before: String::new(),
            match_preview_match: String::new(),
            match_preview_after: String::new(),
            full_path: PathBuf::from(format!("/tmp/f{i}.txt")),
            relative_path: PathBuf::from(format!("f{i}.txt")),
            match_count: 1,
        }
    }
}
