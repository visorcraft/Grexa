// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `SearchController` — the QObject that drives the active Search tab.
//!
//! Plays three roles:
//!
//! 1. **Search orchestrator.** Owns the form-state qproperties
//!    (`status_text`, `match_count`, `busy`, `recent_path_count`) and
//!    the `start_search` / `cancel` / `recent_paths_json` invokables.
//! 2. **Result list model.** Inherits `QAbstractListModel` so QML's
//!    `ListView { model: searchController }` binds directly. Each row
//!    is one `ResultRow` (path / line / preview).
//! 3. **Async worker host.** Implements [`cxx_qt::Threading`] so the
//!    blocking `search_with` call runs on a `std::thread::spawn`'d
//!    worker, with progress events hopped back to the GUI thread via
//!    `qt_thread().queue(...)` in 64-match batches.

use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use grexa_core::{
    CancelToken, ProgressEvent, SearchOptions, SearchResult, context_preview, search_with,
};

use super::workspace_handle::with_workspace;

/// How many matches the worker collects before queueing a batch hop to
/// the GUI thread. Tuned for 16ms render budget at typical match
/// preview widths.
const BATCH_SIZE: usize = 64;

/// One row in the result list model.
#[derive(Debug, Clone)]
pub struct ResultRow {
    pub full_path: PathBuf,
    pub relative_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub preview_before: String,
    pub preview_match: String,
    pub preview_after: String,
}

impl From<&SearchResult> for ResultRow {
    fn from(r: &SearchResult) -> Self {
        Self {
            full_path: r.full_path.clone(),
            relative_path: r.relative_path.clone(),
            line: r.line_number as u32,
            column: r.column_number as u32,
            preview_before: r.match_preview_before.clone(),
            preview_match: r.match_preview_match.clone(),
            preview_after: r.match_preview_after.clone(),
        }
    }
}

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QString, status_text)]
        #[qproperty(i32, match_count)]
        #[qproperty(i32, files_matched)]
        #[qproperty(i32, files_scanned)]
        #[qproperty(bool, busy)]
        #[qproperty(i32, recent_path_count)]
        type SearchController = super::SearchControllerRust;

        /// Start an asynchronous search. The current results are cleared
        /// and rows stream in as the worker thread reports matches.
        /// Returns immediately; observe `busy` for completion.
        #[qinvokable]
        fn start_search(
            self: Pin<&mut SearchController>,
            path: &QString,
            term: &QString,
            regex: bool,
            case_sensitive: bool,
            whole_word: bool,
        );

        /// Cancel the in-flight search. Idempotent; safe to call when
        /// no search is running.
        #[qinvokable]
        fn cancel(self: Pin<&mut SearchController>);

        /// Clear the result list and reset `match_count` to 0.
        #[qinvokable]
        fn clear_results(self: Pin<&mut SearchController>);

        /// Return the recent-paths list as a JSON array.
        #[qinvokable]
        fn recent_paths_json(self: &SearchController) -> QString;

        /// Read a single property of a row from QML — used by context
        /// menus / dialogs that need a value outside the delegate.
        #[qinvokable]
        fn row_full_path(self: &SearchController, row: i32) -> QString;

        /// Render a context preview around a match. Returns formatted
        /// text with line numbers and a `>` marker on the match line,
        /// or a human-readable error if the file can't be read. Used
        /// by the Context Preview dialog.
        #[qinvokable]
        fn preview_at(self: &SearchController, path: &QString, line: i32) -> QString;

        /// Fired when the recent-paths list grows or shrinks.
        #[qsignal]
        fn history_changed(self: Pin<&mut SearchController>);

        /// Fired exactly once when an async search ends. `cancelled`
        /// indicates whether the search was stopped before completing.
        #[qsignal]
        fn search_completed(self: Pin<&mut SearchController>, cancelled: bool);
    }

    // QAbstractListModel overrides.
    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &SearchController, parent: &QModelIndex) -> i32;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &SearchController, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &SearchController) -> QHash_i32_QByteArray;
    }

    // Inherited from QAbstractListModel for begin/end notification.
    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut SearchController>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut SearchController>);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut SearchController>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut SearchController>);
    }

    impl cxx_qt::Threading for SearchController {}
}

/// Role IDs exposed to QML via `roleNames()`. QML accesses them as
/// model.rolePath, model.roleLine, etc. on a delegate.
mod role {
    pub const PATH: i32 = 0x0100; // Qt::UserRole + 0
    pub const RELATIVE_PATH: i32 = 0x0101;
    pub const LINE: i32 = 0x0102;
    pub const COLUMN: i32 = 0x0103;
    pub const PREVIEW_BEFORE: i32 = 0x0104;
    pub const PREVIEW_MATCH: i32 = 0x0105;
    pub const PREVIEW_AFTER: i32 = 0x0106;
}

/// Rust-side state for `SearchController`. Owned by the cxx-qt-generated
/// C++ class via `super::SearchControllerRust`.
#[derive(Default)]
pub struct SearchControllerRust {
    status_text: QString,
    match_count: i32,
    files_matched: i32,
    files_scanned: i32,
    busy: bool,
    recent_path_count: i32,
    rows: Vec<ResultRow>,
    cancel_token: Option<CancelToken>,
}

impl SearchControllerRust {
    /// Append a batch of rows to the model. Returns the (first_idx, last_idx)
    /// pair the caller should pass to `beginInsertRows` / `endInsertRows`.
    pub fn append_batch(&mut self, batch: Vec<ResultRow>) -> Option<(i32, i32)> {
        if batch.is_empty() {
            return None;
        }
        let first = self.rows.len() as i32;
        let last = first + batch.len() as i32 - 1;
        self.rows.extend(batch);
        Some((first, last))
    }

    /// Row count for `QAbstractListModel::rowCount`.
    pub fn row_count(&self) -> i32 {
        self.rows.len() as i32
    }

    /// Read a row's data for a given role. Returns `None` for out-of-range
    /// indices or unknown roles.
    pub fn row_data(&self, row: usize, role: i32) -> Option<String> {
        let r = self.rows.get(row)?;
        Some(match role {
            role::PATH => r.full_path.to_string_lossy().into_owned(),
            role::RELATIVE_PATH => r.relative_path.to_string_lossy().into_owned(),
            role::LINE => r.line.to_string(),
            role::COLUMN => r.column.to_string(),
            role::PREVIEW_BEFORE => r.preview_before.clone(),
            role::PREVIEW_MATCH => r.preview_match.clone(),
            role::PREVIEW_AFTER => r.preview_after.clone(),
            _ => return None,
        })
    }

    /// Recent paths as a JSON array string.
    pub fn recent_paths_json_string(&self) -> String {
        let strings: Vec<String> = with_workspace(|w| {
            w.recent_paths
                .load()
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        });
        serde_json::to_string(&strings).unwrap_or_else(|_| "[]".into())
    }
}

impl ffi::SearchController {
    fn start_search(
        mut self: Pin<&mut Self>,
        path: &QString,
        term: &QString,
        regex: bool,
        case_sensitive: bool,
        whole_word: bool,
    ) {
        // Cancel any in-flight search so we don't end up with two workers
        // pushing into the same model.
        if let Some(token) = self.as_ref().rust().cancel_token.clone() {
            token.cancel();
        }

        // Clear the model up-front so QML sees the reset before rows start
        // streaming in.
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().rows.clear();
        unsafe { self.as_mut().end_reset_model() };

        let path_str = path.to_string();
        let term_str = term.to_string();
        let cancel = CancelToken::new();
        self.as_mut().rust_mut().cancel_token = Some(cancel.clone());

        self.as_mut().set_match_count(0);
        self.as_mut().set_files_matched(0);
        self.as_mut().set_files_scanned(0);
        self.as_mut().set_busy(true);
        self.as_mut().set_status_text(QString::from("Searching…"));

        let mut options = SearchOptions::new(PathBuf::from(&path_str), &term_str);
        options.regex = regex;
        options.case_sensitive = case_sensitive;
        // SearchOptions doesn't expose a whole-word toggle yet; tracked
        // for Phase 4 follow-up. The QML side passes the flag so the
        // wiring is in place when the core lands it.
        let _ = whole_word;

        let thread = self.qt_thread();

        std::thread::spawn(move || {
            let mut batch: Vec<ResultRow> = Vec::with_capacity(BATCH_SIZE);
            let mut files_scanned: u32 = 0;
            let mut files_matched: u32 = 0;

            let outcome = {
                let emit_batch = |events: &mut Vec<ResultRow>, scanned: u32, matched: u32| {
                    if events.is_empty() {
                        return;
                    }
                    let rows = std::mem::take(events);
                    let _ = thread.queue(move |pin| {
                        let scanned_i32 = scanned as i32;
                        let matched_i32 = matched as i32;
                        push_rows(pin, rows, scanned_i32, matched_i32);
                    });
                };

                let mut sink = |event: ProgressEvent| match event {
                    ProgressEvent::Match(r) => {
                        batch.push(ResultRow::from(&r));
                        if batch.len() >= BATCH_SIZE {
                            emit_batch(&mut batch, files_scanned, files_matched);
                        }
                    }
                    ProgressEvent::FileScanned { matches, .. } => {
                        files_scanned += 1;
                        if matches > 0 {
                            files_matched += 1;
                        }
                    }
                    ProgressEvent::FileSkipped { .. } => {}
                };

                let result = search_with(&options, &cancel, Some(&mut sink));
                // Flush remaining batch even if search errored mid-way.
                emit_batch(&mut batch, files_scanned, files_matched);
                result
            };

            let cancelled = cancel.is_cancelled();
            let _ = thread.queue(move |pin| {
                finish_search(pin, outcome, cancelled, &path_str);
            });
        });
    }

    fn cancel(mut self: Pin<&mut Self>) {
        if let Some(token) = self.as_ref().rust().cancel_token.clone() {
            token.cancel();
        }
        // The worker emits search_completed when it sees the cancellation —
        // don't flip `busy` here, let the worker do it on its way out.
        self.as_mut().set_status_text(QString::from("Cancelling…"));
    }

    fn clear_results(mut self: Pin<&mut Self>) {
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().rows.clear();
        unsafe { self.as_mut().end_reset_model() };
        self.as_mut().set_match_count(0);
        self.as_mut().set_files_matched(0);
        self.as_mut().set_files_scanned(0);
    }

    fn recent_paths_json(&self) -> QString {
        QString::from(&self.rust().recent_paths_json_string())
    }

    fn row_full_path(&self, row: i32) -> QString {
        if row < 0 {
            return QString::default();
        }
        match self.rust().rows.get(row as usize) {
            Some(r) => QString::from(r.full_path.to_string_lossy().as_ref()),
            None => QString::default(),
        }
    }

    fn preview_at(&self, path: &QString, line: i32) -> QString {
        if line < 1 {
            return QString::from("(invalid line number)");
        }
        let path = std::path::PathBuf::from(path.to_string());
        match context_preview(&path, line as usize, 5, 5) {
            Ok(result) => {
                let mut buf = String::new();
                for ln in &result.lines {
                    use std::fmt::Write;
                    let marker = if ln.is_match { '>' } else { ' ' };
                    let _ = writeln!(&mut buf, "{marker} {:>5}  {}", ln.line_number, ln.content);
                }
                QString::from(&buf)
            }
            Err(err) => QString::from(&format!("(preview failed: {err})")),
        }
    }

    fn row_count(&self, parent: &QModelIndex) -> i32 {
        // Flat list: only the root index has rows.
        if parent.row() >= 0 {
            return 0;
        }
        self.rust().row_count()
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let row = index.row();
        if row < 0 {
            return QVariant::default();
        }
        match self.rust().row_data(row as usize, role) {
            Some(value) => QVariant::from(&QString::from(&value)),
            None => QVariant::default(),
        }
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut hash = QHash::<QHashPair_i32_QByteArray>::default();
        hash.insert(role::PATH, QByteArray::from("path"));
        hash.insert(role::RELATIVE_PATH, QByteArray::from("relativePath"));
        hash.insert(role::LINE, QByteArray::from("line"));
        hash.insert(role::COLUMN, QByteArray::from("column"));
        hash.insert(role::PREVIEW_BEFORE, QByteArray::from("previewBefore"));
        hash.insert(role::PREVIEW_MATCH, QByteArray::from("previewMatch"));
        hash.insert(role::PREVIEW_AFTER, QByteArray::from("previewAfter"));
        hash
    }
}

fn push_rows(
    mut pin: Pin<&mut ffi::SearchController>,
    rows: Vec<ResultRow>,
    files_scanned: i32,
    files_matched: i32,
) {
    if rows.is_empty() {
        return;
    }
    let parent = QModelIndex::default();
    // Two-step append: peek the range first so we can wrap the actual
    // mutation in `begin_insert_rows` / `end_insert_rows`.
    let preview_first = pin.as_ref().rust().row_count();
    let preview_last = preview_first + rows.len() as i32 - 1;
    unsafe {
        pin.as_mut()
            .begin_insert_rows(&parent, preview_first, preview_last)
    };
    let appended = pin.as_mut().rust_mut().append_batch(rows);
    unsafe { pin.as_mut().end_insert_rows() };

    let added = appended.map(|(f, l)| l - f + 1).unwrap_or(0);
    if added == 0 {
        return;
    }
    let new_count = pin.as_ref().rust().match_count + added;
    pin.as_mut().set_match_count(new_count);
    pin.as_mut().set_files_scanned(files_scanned);
    pin.as_mut().set_files_matched(files_matched);
}

fn finish_search(
    mut pin: Pin<&mut ffi::SearchController>,
    outcome: Result<grexa_core::SearchSummary, grexa_core::SearchError>,
    cancelled: bool,
    path_str: &str,
) {
    pin.as_mut().set_busy(false);
    match outcome {
        Ok(summary) => {
            let path = PathBuf::from(path_str);
            with_workspace(|w| {
                let _ = w.recent_paths.add(path);
            });
            let recent_count =
                with_workspace(|w| w.recent_paths.load().unwrap_or_default().len() as i32);
            let previous = pin.as_ref().rust().recent_path_count;
            if recent_count != previous {
                pin.as_mut().set_recent_path_count(recent_count);
                pin.as_mut().history_changed();
            }
            let status = if cancelled {
                format!(
                    "Cancelled — {} matches in {} files",
                    pin.as_ref().rust().match_count,
                    pin.as_ref().rust().files_matched
                )
            } else {
                format!(
                    "Found {} matches in {} files in {} ms",
                    summary.matches, summary.files_matched, summary.elapsed_ms
                )
            };
            pin.as_mut().set_status_text(QString::from(&status));
        }
        Err(err) => {
            pin.as_mut()
                .set_status_text(QString::from(&format!("Error: {err}")));
        }
    }
    pin.as_mut().rust_mut().cancel_token = None;
    pin.as_mut().search_completed(cancelled);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::Workspace;
    use std::cell::RefCell;
    use std::fs;
    use std::rc::Rc;
    use tempfile::tempdir;

    #[test]
    fn append_batch_returns_inclusive_range() {
        let mut state = SearchControllerRust::default();
        let batch = vec![ResultRow {
            full_path: PathBuf::from("/x"),
            relative_path: PathBuf::from("x"),
            line: 1,
            column: 1,
            preview_before: String::new(),
            preview_match: "TODO".into(),
            preview_after: String::new(),
        }];
        let range = state.append_batch(batch).unwrap();
        assert_eq!(range, (0, 0));
        assert_eq!(state.row_count(), 1);
    }

    #[test]
    fn row_data_returns_none_for_unknown_role() {
        let mut state = SearchControllerRust::default();
        state.append_batch(vec![ResultRow {
            full_path: PathBuf::from("/a"),
            relative_path: PathBuf::from("a"),
            line: 7,
            column: 3,
            preview_before: "before ".into(),
            preview_match: "MATCH".into(),
            preview_after: " after".into(),
        }]);
        assert_eq!(state.row_data(0, role::PATH).as_deref(), Some("/a"));
        assert_eq!(state.row_data(0, role::LINE).as_deref(), Some("7"));
        assert_eq!(state.row_data(0, role::PREVIEW_MATCH).as_deref(), Some("MATCH"));
        assert_eq!(state.row_data(0, 0xdead), None);
        assert_eq!(state.row_data(99, role::PATH), None);
    }

    #[test]
    fn recent_paths_json_is_array_when_workspace_empty() {
        let dir = tempdir().unwrap();
        let ws = Rc::new(RefCell::new(Workspace::under(&dir.path().join("xdg"))));
        super::super::install_workspace(ws);

        let state = SearchControllerRust::default();
        assert_eq!(state.recent_paths_json_string(), "[]");
    }

    #[test]
    fn append_batch_accumulates_offsets() {
        let mut state = SearchControllerRust::default();
        let row = ResultRow {
            full_path: PathBuf::from("/x"),
            relative_path: PathBuf::from("x"),
            line: 1,
            column: 1,
            preview_before: String::new(),
            preview_match: "m".into(),
            preview_after: String::new(),
        };
        assert_eq!(state.append_batch(vec![row.clone(); 3]), Some((0, 2)));
        assert_eq!(state.append_batch(vec![row.clone(); 2]), Some((3, 4)));
        assert_eq!(state.row_count(), 5);
    }

    #[test]
    fn search_engine_streaming_produces_rows() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.txt");
        fs::write(&file_a, "TODO 1\nTODO 2\nplain\nTODO 3\n").unwrap();

        let mut rows: Vec<ResultRow> = Vec::new();
        let mut sink = |event: ProgressEvent| {
            if let ProgressEvent::Match(r) = event {
                rows.push(ResultRow::from(&r));
            }
        };

        let options = SearchOptions::new(dir.path().to_path_buf(), "TODO");
        let cancel = CancelToken::new();
        let summary = search_with(&options, &cancel, Some(&mut sink)).unwrap();

        assert_eq!(summary.matches, 3);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r.preview_match == "TODO"));
    }
}
