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

/// Resolve a leading `~/` against `$HOME`. The empty-state chips,
/// recent-paths combo, and Workspace history all advertise tilde
/// paths — without this the search engine errors with
/// `PathNotFound("~/code")`.
fn expand_tilde(path: &str) -> String {
    let home = std::env::var_os("HOME");
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = home.as_ref()
    {
        let mut p = std::path::PathBuf::from(home);
        p.push(rest);
        return p.to_string_lossy().into_owned();
    }
    if path == "~"
        && let Some(home) = home
    {
        return std::path::PathBuf::from(home)
            .to_string_lossy()
            .into_owned();
    }
    path.to_string()
}

/// How many matches the worker collects before queueing a batch hop to
/// the GUI thread. Tuned for 16ms render budget at typical match
/// preview widths.
const BATCH_SIZE: usize = 64;

/// Snapshot of one tab's result buffer plus the qproperty-shaped
/// state the tab depends on. Created on tab-switch-away and
/// restored on tab-switch-back via the QML tab bar.
///
/// We keep `rows` here — `visible` is recomputed from `rows` +
/// `result_mode` + `within_filter` on restore, so the snapshot
/// is the canonical raw match list and the projection is
/// re-derived. That avoids stale `visible` indices if the user
/// flipped the within-filter while a different tab was active.
#[derive(Debug, Clone, Default)]
struct TabSnapshot {
    rows: Vec<ResultRow>,
    last_path: String,
    last_term: String,
    last_regex: bool,
    last_case_sensitive: bool,
    status_text: String,
    match_count: i32,
    files_matched: i32,
    files_scanned: i32,
    has_searched: bool,
    result_mode: i32,
    within_filter: String,
    within_regex: bool,
    target_kind: i32,
    selected_container_id: String,
    // Tab-local pipeline state. `busy` and `replacing` track whether the
    // tab has an in-flight search or replace; `last_replace_summary` is
    // the JSON-encoded result banner shown after a replace completes.
    // Container listings (`containers_json`) and `recent_path_count` are
    // intentionally session-global, not snapshotted.
    busy: bool,
    replacing: bool,
    last_replace_summary: String,
}

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

    #[auto_cxx_name]
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
        // 0 = Local, 1 = Docker, 2 = Podman (rootless), 3 = Podman (rootful).
        // QML target-selector dropdown writes here; `start_search` reads
        // it to dispatch between grexa-core and grexa-containers.
        #[qproperty(i32, target_kind)]
        // When `target_kind != Local`, this is the container ID the
        // user picked from the runtime's list. Empty otherwise.
        #[qproperty(QString, selected_container_id)]
        // 0 = Content (one row per match), 1 = Files (one row per file).
        // The model deduplicates rows when `result_mode == 1`.
        #[qproperty(i32, result_mode)]
        // Search-within-results filter. Empty disables the filter.
        // When `within_regex` is true, treated as a regex pattern;
        // otherwise plain substring match.
        #[qproperty(QString, within_filter)]
        #[qproperty(bool, within_regex)]
        // Replace pipeline state. `replacing` is the analogue of
        // `busy` for replace operations.
        #[qproperty(bool, replacing)]
        #[qproperty(QString, last_replace_summary)]
        // True once the user has clicked Search at least this session —
        // lets the empty state distinguish "haven't searched yet"
        // (false) from "searched, no matches" (true && match_count==0).
        #[qproperty(bool, has_searched)]
        // Cached container-runtime discovery result. Populated
        // asynchronously by `refresh_containers`; QML watches its
        // changed signal to repopulate the target dropdown without
        // blocking the GUI thread on `docker ps` / `podman ps`.
        #[qproperty(QString, containers_json)]
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

        /// Add `path` to the recent-paths store. Idempotent; the store
        /// dedupes. Used by the folder-picker dialog to remember
        /// browsed locations without requiring a successful search.
        #[qinvokable]
        fn add_recent_path(self: Pin<&mut SearchController>, path: &QString);

        /// Remove `path` from the recent-paths store. Used by the
        /// combobox's per-entry × affordance.
        #[qinvokable]
        fn remove_recent_path(self: Pin<&mut SearchController>, path: &QString);

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

        // ---- Result row context menu actions --------------------

        /// Open the file in the configured editor. Falls back to
        /// `xdg-open` when no preset is configured. Non-blocking — the
        /// editor process is detached from grexa.
        #[qinvokable]
        fn open_in_editor(self: &SearchController, path: &QString, line: i32);

        /// Highlight the file in the user's file manager via the
        /// FileManager1 D-Bus interface, falling back to `xdg-open`
        /// on the parent directory.
        #[qinvokable]
        fn reveal_in_file_manager(self: &SearchController, path: &QString);

        /// Copy arbitrary text to the system clipboard. Implementation
        /// shells to `wl-copy` (Wayland) or `xclip` (X11) — both are
        /// commonly available on KDE/GNOME hosts and ship in Flatpak
        /// base runtimes.
        #[qinvokable]
        fn copy_to_clipboard(self: &SearchController, text: &QString);

        // ---- Container search dispatch --------------------------

        /// Refresh the cached container-runtime discovery. Runs the
        /// `docker ps` / `podman ps` probes on a worker thread and
        /// updates `containers_json` when complete. QML target
        /// selector listens for `containers_jsonChanged` to repopulate
        /// without blocking the GUI thread.
        #[qinvokable]
        fn refresh_containers(self: Pin<&mut SearchController>);

        // ---- Replace pipeline -----------------------------------

        /// Run the replace flow on the current path + term + filters
        /// (mirrors the last search). Streams the replace summary back
        /// via `replace_completed` and sets `result_mode = Files` on
        /// success (matching Grex's behavior). Refuses for container
        /// targets and when no search has run.
        #[qinvokable]
        fn start_replace(self: Pin<&mut SearchController>, replacement: &QString);

        // ---- View refinement ------------------------------------

        /// Re-apply `within_filter` + `result_mode` against the
        /// current row set and notify QML via a model reset. Cheap
        /// because filtering operates on already-decoded rows.
        #[qinvokable]
        fn refresh_view(self: Pin<&mut SearchController>);

        /// Export every visible row to `dest_path` in the given
        /// format. `format`: 0=CSV, 1=JSON, 2=Markdown table.
        /// Writes synchronously; small files only — multi-million
        /// row exports should stream via the CLI.
        #[qinvokable]
        fn export_results(self: &SearchController, dest_path: &QString, format: i32) -> QString;

        /// Sort the underlying row set in place by the given column.
        /// `column`: 0=Path, 1=Line, 2=Match preview text.
        /// `ascending`: false flips to descending.
        ///
        /// Mutates `rows` and rebuilds `visible`. Surfaces via a
        /// model reset.
        #[qinvokable]
        fn sort_results(self: Pin<&mut SearchController>, column: i32, ascending: bool);

        /// Return the residual replace journal (if any) as a JSON
        /// object describing the most recent interrupted replace
        /// run. Empty string when no journal is present. The GUI
        /// shell calls this at startup when
        /// `replace_show_journal_on_startup` is true.
        #[qinvokable]
        fn residual_journal_json(self: &SearchController) -> QString;

        /// Clear the on-disk residual replace journal. Called from
        /// the recovery dialog's "Dismiss" button.
        #[qinvokable]
        fn clear_residual_journal(self: &SearchController);

        // ---- History --------------------------------------------

        /// Return the persisted search history as a JSON array.
        /// Each entry has `search_term`, `search_path`,
        /// `match_file_names`, `exclude_dirs`, `regex_search`,
        /// `files_search`, `search_case_sensitive`,
        /// `respect_gitignore`, `include_subfolders`,
        /// `include_hidden_items`, `include_binary_files`,
        /// `timestamp_unix`, `result_count`. Pulls from
        /// `SearchHistoryStore`.
        #[qinvokable]
        fn history_json(self: &SearchController) -> QString;

        /// Drop a history entry by its full row JSON. The store
        /// dedupes via `key`, so this passes the exact entry back.
        #[qinvokable]
        fn remove_history_entry(self: &SearchController, entry_json: &QString);

        // ---- Profiles -------------------------------------------

        /// Return every saved search profile as a JSON array.
        #[qinvokable]
        fn profiles_json(self: &SearchController) -> QString;

        /// Save the current search parameters (`path`, `term`,
        /// `regex`, `case_sensitive`, `result_mode`) as a named
        /// profile. Upserts on collision with the same name.
        #[qinvokable]
        fn save_profile(
            self: &SearchController,
            name: &QString,
            path: &QString,
            term: &QString,
            regex: bool,
            case_sensitive: bool,
            files_mode: bool,
        ) -> bool;

        /// Delete a saved profile by name.
        #[qinvokable]
        fn delete_profile(self: &SearchController, name: &QString) -> bool;

        // ---- Per-tab result-row isolation -----------------------

        /// Snapshot the current row buffer + qproperty state under
        /// `tab_id`. Called from QML before switching to a
        /// different tab. Idempotent — overwrites any existing
        /// snapshot for the same id.
        #[qinvokable]
        fn save_tab_snapshot(self: Pin<&mut SearchController>, tab_id: i32);

        /// Restore a previously-saved snapshot. Resets the model,
        /// reinstalls the rows, and re-emits the qproperty
        /// setters so QML sees the right counter / status. When
        /// `tab_id` has no snapshot, falls back to clearing the
        /// model (the "fresh tab" case).
        #[qinvokable]
        fn restore_tab_snapshot(self: Pin<&mut SearchController>, tab_id: i32);

        /// Drop a tab's snapshot. Called when the QML tab bar
        /// closes a tab so memory doesn't leak.
        #[qinvokable]
        fn drop_tab_snapshot(self: Pin<&mut SearchController>, tab_id: i32);

        /// Fired when the recent-paths list grows or shrinks.
        #[qsignal]
        fn history_changed(self: Pin<&mut SearchController>);

        /// Fired exactly once when an async search ends. `cancelled`
        /// indicates whether the search was stopped before completing.
        #[qsignal]
        fn search_completed(self: Pin<&mut SearchController>, cancelled: bool);

        /// Fired exactly once when a replace operation ends.
        /// `success` is false when the engine returned an error;
        /// otherwise `last_replace_summary` holds the JSON-encoded
        /// `ReplaceSummary`.
        #[qsignal]
        fn replace_completed(self: Pin<&mut SearchController>, success: bool);
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
    target_kind: i32,
    selected_container_id: QString,
    result_mode: i32,
    within_filter: QString,
    within_regex: bool,
    replacing: bool,
    last_replace_summary: QString,
    has_searched: bool,
    containers_json: QString,
    /// All rows the search emitted, before any view-level filtering
    /// or files-mode deduplication.
    rows: Vec<ResultRow>,
    /// Indices into `rows` that survive the current `within_filter`
    /// and `result_mode` view rules. The QAbstractListModel layer
    /// projects through this — `row_count` returns `visible.len()`,
    /// `row_data(i)` reads `rows[visible[i]]`.
    visible: Vec<usize>,
    /// The last successful search's path + term + flags. Replace and
    /// "refresh view" both replay against these.
    last_path: String,
    last_term: String,
    last_regex: bool,
    last_case_sensitive: bool,
    /// Per-tab result-row snapshots keyed by the QML-side monotonic
    /// tab id. Switching to a different tab calls
    /// `save_tab_snapshot(prev)` then `restore_tab_snapshot(next)`
    /// so each tab keeps its full result buffer. Memory cost scales
    /// with `tabs × rows-per-tab`, which is the price of real
    /// per-tab isolation; closing a tab drops its snapshot.
    tab_snapshots: std::collections::HashMap<i32, TabSnapshot>,
    cancel_token: Option<CancelToken>,
    /// Monotonic counter incremented on every `start_search`. Late
    /// `thread.queue` hops from a prior worker compare their captured
    /// generation against this and drop themselves when stale.
    active_generation: u64,
}

impl SearchControllerRust {
    fn cancel_active_search(&mut self) {
        if let Some(token) = self.cancel_token.take() {
            token.cancel();
        }
        self.active_generation = self.active_generation.wrapping_add(1);
    }

    /// Append a batch of rows to the model. Returns the (first_idx, last_idx)
    /// pair the caller should pass to `beginInsertRows` / `endInsertRows`.
    /// Indices are *visible* indices — when files-mode dedup or
    /// within-filter is active, the returned pair already accounts
    /// for which of the appended rows are actually visible.
    ///
    /// Test helper. Live `push_rows` uses the two-phase
    /// `filter_batch_for_view` + `append_with_visible` split so
    /// `beginInsertRows` can be bracketed around the correct
    /// visible-row count.
    #[cfg(test)]
    pub fn append_batch(&mut self, batch: Vec<ResultRow>) -> Option<(i32, i32)> {
        if batch.is_empty() {
            return None;
        }
        let kept = self.filter_batch_for_view(&batch);
        if kept.is_empty() {
            self.rows.extend(batch);
            return None;
        }
        let visible_before = self.visible.len();
        let first = visible_before as i32;
        let last = first + kept.len() as i32 - 1;
        self.append_with_visible(batch, kept);
        Some((first, last))
    }

    /// Pre-compute which rows in `batch` would pass the current
    /// view rules (`within_filter` + files-mode dedup) and return
    /// their indices INTO `batch` (NOT into `self.rows`). The caller
    /// uses the length to bracket `beginInsertRows`/`endInsertRows`,
    /// then commits via `append_with_visible(batch, kept)`.
    ///
    /// Splitting filter from append lets the model contract hold —
    /// `rowCount` always grows by exactly `kept.len()`.
    pub fn filter_batch_for_view(&self, batch: &[ResultRow]) -> Vec<usize> {
        let mut kept: Vec<usize> = Vec::with_capacity(batch.len());
        // Files-mode dedup must consider rows we're about to keep
        // from THIS batch as well as rows already in `self.visible`.
        // Track seen full_paths across both sources.
        if self.result_mode == 1 {
            let mut seen: std::collections::HashSet<std::path::PathBuf> =
                std::collections::HashSet::with_capacity(self.visible.len() + batch.len());
            for &idx in &self.visible {
                if let Some(r) = self.rows.get(idx) {
                    seen.insert(r.full_path.clone());
                }
            }
            for (i, row) in batch.iter().enumerate() {
                if !self.row_passes_within(row) {
                    continue;
                }
                if seen.insert(row.full_path.clone()) {
                    kept.push(i);
                }
            }
        } else {
            for (i, row) in batch.iter().enumerate() {
                if self.row_passes_within(row) {
                    kept.push(i);
                }
            }
        }
        kept
    }

    /// Commit a filtered batch. `kept` must come from
    /// `filter_batch_for_view(&batch)`; the indices are translated
    /// into `self.rows`-space as we extend.
    pub fn append_with_visible(&mut self, batch: Vec<ResultRow>, kept: Vec<usize>) {
        let new_start = self.rows.len();
        self.visible.extend(kept.iter().map(|&i| new_start + i));
        self.rows.extend(batch);
    }

    /// Row count for `QAbstractListModel::rowCount`. Reflects the
    /// view (within-filter + files-mode dedup), not raw `rows.len()`.
    pub fn row_count(&self) -> i32 {
        self.visible.len() as i32
    }

    /// Read a row's data for a given role. Returns `None` for out-of-range
    /// indices or unknown roles.
    pub fn row_data(&self, row: usize, role: i32) -> Option<String> {
        let idx = *self.visible.get(row)?;
        let r = self.rows.get(idx)?;
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

    /// Within-filter test only — does NOT consider files-mode dedup.
    /// Files-mode dedup is structurally handled where seen-set
    /// bookkeeping is available (`filter_batch_for_view` and
    /// `rebuild_visible`). Splitting the two keeps the dedup logic
    /// from accidentally consulting a partially-emptied
    /// `self.rows` (the original bug found in code review).
    fn row_passes_within(&self, row: &ResultRow) -> bool {
        let within = self.within_filter.to_string();
        let trimmed = within.trim();
        if trimmed.is_empty() {
            return true;
        }
        let line = format!("{}{}{}", row.preview_before, row.preview_match, row.preview_after);
        if self.within_regex {
            match regex::Regex::new(trimmed) {
                Ok(re) => re.is_match(&line),
                Err(_) => false,
            }
        } else {
            let needle = trimmed.to_lowercase();
            line.to_lowercase().contains(&needle)
        }
    }

    /// Recompute `visible` from scratch. O(rows.len()). Called when
    /// `within_filter`, `within_regex`, or `result_mode` changes.
    ///
    /// Iterates `self.rows` by index so we never have to take
    /// ownership of the vector mid-loop — the previous version
    /// did `mem::take(&mut self.rows)` and then asked the
    /// view-pass helper to consult `self.rows` for files-mode
    /// dedup, which always returned None because the vector was
    /// empty. The seen-set is now local to this function so dedup
    /// works regardless of `self.rows`'s state at entry.
    pub fn rebuild_visible(&mut self) {
        self.visible.clear();
        let files_mode = self.result_mode == 1;
        let mut seen: std::collections::HashSet<std::path::PathBuf> =
            std::collections::HashSet::new();
        for i in 0..self.rows.len() {
            if !self.row_passes_within(&self.rows[i]) {
                continue;
            }
            if files_mode {
                let full = self.rows[i].full_path.clone();
                if !seen.insert(full) {
                    continue;
                }
            }
            self.visible.push(i);
        }
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

    fn clear_rows_and_last_search(&mut self) {
        self.rows.clear();
        self.visible.clear();
        self.last_path.clear();
        self.last_term.clear();
        self.last_regex = false;
        self.last_case_sensitive = false;
    }

    #[cfg(test)]
    fn clear_search_state(&mut self) {
        self.clear_rows_and_last_search();
        self.match_count = 0;
        self.files_matched = 0;
        self.files_scanned = 0;
        self.has_searched = false;
        self.status_text = QString::default();
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
        // Cancel any in-flight search so we don't end up with two
        // workers pushing into the same model.
        if let Some(token) = self.as_ref().rust().cancel_token.clone() {
            token.cancel();
        }

        // Bump the generation. Closures queued by the prior worker
        // capture the old value and become no-ops; everything queued
        // from this point on captures the new one.
        let generation = self.as_ref().rust().active_generation.wrapping_add(1);
        self.as_mut().rust_mut().active_generation = generation;

        // Clear the model up-front so QML sees the reset before rows
        // start streaming in.
        unsafe { self.as_mut().begin_reset_model() };
        {
            let mut s = self.as_mut().rust_mut();
            s.rows.clear();
            s.visible.clear();
        }
        unsafe { self.as_mut().end_reset_model() };

        let path_str = expand_tilde(&path.to_string());
        let term_str = term.to_string();
        let cancel = CancelToken::new();
        {
            let mut s = self.as_mut().rust_mut();
            s.cancel_token = Some(cancel.clone());
            // Remember the search shape so `start_replace` can replay
            // it against the same scope without the QML having to
            // re-supply every field.
            s.last_path = path_str.clone();
            s.last_term = term_str.clone();
            s.last_regex = regex;
            s.last_case_sensitive = case_sensitive;
        }

        self.as_mut().set_match_count(0);
        self.as_mut().set_files_matched(0);
        self.as_mut().set_files_scanned(0);
        self.as_mut().set_busy(true);
        self.as_mut().set_has_searched(true);
        self.as_mut().set_status_text(QString::from("Searching…"));

        // Container target — dispatch to grexa-containers and skip the
        // local search-engine path entirely.
        let target_kind = self.as_ref().rust().target_kind;
        if target_kind != 0 {
            let container_id = self.as_ref().rust().selected_container_id.to_string();
            if container_id.trim().is_empty() {
                self.as_mut().rust_mut().cancel_token = None;
                self.as_mut().set_busy(false);
                self.as_mut().set_status_text(QString::from(
                    "Pick a container in the target selector before searching.",
                ));
                self.as_mut().search_completed(false);
                return;
            }
            let thread = self.qt_thread();
            let path_for_container = path_str.clone();
            let term_for_container = term_str.clone();
            std::thread::spawn(move || {
                let summary = run_container_search(
                    target_kind,
                    &container_id,
                    &path_for_container,
                    &term_for_container,
                    regex,
                    case_sensitive,
                );
                let _ = thread.queue(move |pin| {
                    finish_container_search(pin, generation, summary);
                });
            });
            return;
        }

        // Forward the persisted Settings into the SearchOptions so
        // toggles like `Respect .gitignore`, `Include hidden`, the
        // default match-files glob, etc. actually shape this search.
        // The fast/slow boolean flags from the SearchBar override
        // settings for *this* invocation only.
        let mut options = SearchOptions::new(PathBuf::from(&path_str), &term_str);
        options.regex = regex;
        options.case_sensitive = case_sensitive;
        let settings = with_workspace(|w| w.settings.load().unwrap_or_default());
        options.respect_gitignore = settings.respect_gitignore;
        options.include_hidden = settings.include_hidden_items;
        options.include_binary = settings.include_binary_files;
        options.include_system = settings.include_system_files;
        options.include_subfolders = settings.include_subfolders;
        options.include_symlinks = settings.include_symbolic_links;
        options.match_file_names = settings.default_match_files.clone();
        options.exclude_dirs = settings.default_exclude_dirs.clone();
        // Whole-word filtering isn't yet exposed in `SearchOptions`;
        // tracked as a real follow-up against grexa-core, not a
        // silent drop. When that lands, set `options.whole_word`.
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
                        push_rows(pin, generation, rows, scanned_i32, matched_i32);
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
                finish_search(pin, generation, outcome, cancelled, &path_str);
            });
        });
    }

    fn cancel(mut self: Pin<&mut Self>) {
        if !self.as_ref().rust().busy {
            return;
        }
        let matches = self.as_ref().rust().match_count.max(0) as usize;
        let files = self.as_ref().rust().files_matched.max(0) as usize;
        self.as_mut().rust_mut().cancel_active_search();
        self.as_mut().set_busy(false);
        self.as_mut().set_status_text(QString::from(&format!(
            "Cancelled — {} in {}",
            plural_count("count-matches", matches),
            plural_count("count-files", files),
        )));
        self.as_mut().search_completed(true);
    }

    fn clear_results(mut self: Pin<&mut Self>) {
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().clear_rows_and_last_search();
        unsafe { self.as_mut().end_reset_model() };
        self.as_mut().set_match_count(0);
        self.as_mut().set_files_matched(0);
        self.as_mut().set_files_scanned(0);
        self.as_mut().set_has_searched(false);
        self.as_mut().set_status_text(QString::default());
    }

    fn recent_paths_json(&self) -> QString {
        QString::from(&self.rust().recent_paths_json_string())
    }

    fn add_recent_path(mut self: Pin<&mut Self>, path: &QString) {
        let path_str = expand_tilde(&path.to_string());
        let trimmed = path_str.trim();
        if trimmed.is_empty() {
            return;
        }
        let added = with_workspace(|w| w.recent_paths.add(PathBuf::from(trimmed)).is_ok());
        if !added {
            return;
        }
        let count = with_workspace(|w| w.recent_paths.load().unwrap_or_default().len() as i32);
        self.as_mut().set_recent_path_count(count);
        self.as_mut().history_changed();
    }

    fn remove_recent_path(mut self: Pin<&mut Self>, path: &QString) {
        let path_str = path.to_string();
        let trimmed = path_str.trim();
        if trimmed.is_empty() {
            return;
        }
        let removed = with_workspace(|w| w.recent_paths.remove(&PathBuf::from(trimmed)).is_ok());
        if !removed {
            return;
        }
        let count = with_workspace(|w| w.recent_paths.load().unwrap_or_default().len() as i32);
        self.as_mut().set_recent_path_count(count);
        self.as_mut().history_changed();
    }

    fn row_full_path(&self, row: i32) -> QString {
        if row < 0 {
            return QString::default();
        }
        let idx = match self.rust().visible.get(row as usize) {
            Some(&i) => i,
            None => return QString::default(),
        };
        match self.rust().rows.get(idx) {
            Some(r) => QString::from(r.full_path.to_string_lossy().as_ref()),
            None => QString::default(),
        }
    }

    fn preview_at(&self, path: &QString, line: i32) -> QString {
        if line < 1 {
            return QString::from("(invalid line number)");
        }
        let path = std::path::PathBuf::from(path.to_string());
        // Read the user-configured ±N from the persisted Settings,
        // clamping to grexa-core's 0..=50 range. Falls back to 5/5
        // (the historical default in the audit) when settings can't
        // be read for some reason.
        let (before, after) = with_workspace(|w| {
            let s = w.settings.load().unwrap_or_default();
            (s.context_preview_lines_before.min(50), s.context_preview_lines_after.min(50))
        });
        match context_preview(&path, line as usize, before, after) {
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

    fn open_in_editor(&self, path: &QString, line: i32) {
        let path_str = path.to_string();
        if path_str.trim().is_empty() {
            return;
        }
        let (preset, custom_template) = with_workspace(|w| {
            let s = w.settings.load().unwrap_or_default();
            (editor_preset_from_settings(&s), s.editor_custom_command)
        });
        let line_opt = if line >= 1 { Some(line as usize) } else { None };
        // A non-empty custom command always wins. This matches how
        // VS Code / IntelliJ / etc. treat the "External Tools" pattern
        // — the preset gives users one click for common editors, the
        // template lets them override with the exact argv they want.
        let argv = if !custom_template.trim().is_empty() {
            expand_editor_template(custom_template.trim(), &path_str, line_opt)
        } else {
            grexa_core::open_in_editor_command(preset, std::path::Path::new(&path_str), line_opt)
        };
        spawn_detached(argv);
    }

    fn reveal_in_file_manager(&self, path: &QString) {
        let path_str = path.to_string();
        if path_str.trim().is_empty() {
            return;
        }
        let p = std::path::PathBuf::from(&path_str);
        // Try FileManager1 D-Bus first — KDE Dolphin / Nautilus / nemo
        // implement it. Fall back to xdg-open on the parent directory.
        if try_filemanager1_reveal(&p).is_ok() {
            return;
        }
        let argv = grexa_core::reveal_with_xdg_open(&p);
        spawn_detached(argv);
    }

    fn copy_to_clipboard(&self, text: &QString) {
        let s = text.to_string();
        if s.is_empty() {
            return;
        }
        copy_to_system_clipboard(&s);
    }

    fn refresh_containers(self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let json = build_containers_json();
            let _ = thread.queue(move |mut pin| {
                pin.as_mut().set_containers_json(QString::from(&json));
            });
        });
    }

    fn start_replace(mut self: Pin<&mut Self>, replacement: &QString) {
        if self.as_ref().rust().target_kind != 0 {
            self.as_mut()
                .set_status_text(QString::from("Replace is disabled for container targets."));
            self.as_mut().replace_completed(false);
            return;
        }
        let path = self.as_ref().rust().last_path.clone();
        let term = self.as_ref().rust().last_term.clone();
        if path.is_empty() || term.is_empty() {
            self.as_mut()
                .set_status_text(QString::from("Run a search first, then replace."));
            self.as_mut().replace_completed(false);
            return;
        }
        let replacement_str = replacement.to_string();
        self.as_mut().set_replacing(true);

        // Build the same SearchOptions the last search used.
        let regex = self.as_ref().rust().last_regex;
        let case_sensitive = self.as_ref().rust().last_case_sensitive;
        let mut options = grexa_core::SearchOptions::new(PathBuf::from(&path), &term);
        options.regex = regex;
        options.case_sensitive = case_sensitive;
        let settings = with_workspace(|w| w.settings.load().unwrap_or_default());
        options.respect_gitignore = settings.respect_gitignore;
        options.include_hidden = settings.include_hidden_items;
        options.include_binary = settings.include_binary_files;
        options.include_system = settings.include_system_files;
        options.include_subfolders = settings.include_subfolders;
        options.include_symlinks = settings.include_symbolic_links;
        options.match_file_names = settings.default_match_files.clone();
        options.exclude_dirs = settings.default_exclude_dirs.clone();

        let cancel = CancelToken::new();
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let opts = grexa_core::ReplaceOptions {
                search: options,
                replacement: replacement_str,
            };
            let outcome = grexa_core::replace_with(&opts, &cancel, None);
            let _ = thread.queue(move |pin| {
                finish_replace(pin, outcome);
            });
        });
    }

    fn residual_journal_json(&self) -> QString {
        match grexa_core::load_residual_journal() {
            Ok(Some(entry)) => {
                // Serialize a stable subset of the fields. The full
                // struct already derives Serialize via grexa-core.
                let json = serde_json::to_string(&entry).unwrap_or_else(|_| "{}".into());
                QString::from(&json)
            }
            _ => QString::default(),
        }
    }

    fn clear_residual_journal(&self) {
        // Removing the file via the journal-path helper. The
        // public API doesn't expose direct removal; the next
        // successful replace flow ends in a `cleanup` call on
        // its own journal. Until then, delete the file directly.
        let path = grexa_core::AppPaths::from_env()
            .state_dir
            .join("replace-journal.json");
        let _ = std::fs::remove_file(&path);
    }

    fn history_json(&self) -> QString {
        let entries = with_workspace(|w| w.history.load().unwrap_or_default());
        match serde_json::to_string(&entries) {
            Ok(s) => QString::from(&s),
            Err(_) => QString::from("[]"),
        }
    }

    fn remove_history_entry(&self, entry_json: &QString) {
        let s = entry_json.to_string();
        let entry: grexa_core::RecentSearch = match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(_) => return,
        };
        let _ = with_workspace(|w| w.history.remove(&entry));
    }

    fn profiles_json(&self) -> QString {
        let profiles = with_workspace(|w| w.profiles.load().unwrap_or_default());
        match serde_json::to_string(&profiles) {
            Ok(s) => QString::from(&s),
            Err(_) => QString::from("[]"),
        }
    }

    fn save_profile(
        &self,
        name: &QString,
        path: &QString,
        term: &QString,
        regex: bool,
        case_sensitive: bool,
        files_mode: bool,
    ) -> bool {
        let name_str = name.to_string();
        if name_str.trim().is_empty() {
            return false;
        }
        let mut options =
            grexa_core::SearchOptions::new(PathBuf::from(path.to_string()), term.to_string());
        options.regex = regex;
        options.case_sensitive = case_sensitive;
        let profile = grexa_core::SearchProfile::new(name_str, options, files_mode);
        with_workspace(|w| w.profiles.upsert(profile).is_ok())
    }

    fn delete_profile(&self, name: &QString) -> bool {
        let name_str = name.to_string();
        with_workspace(|w| w.profiles.remove(&name_str).is_ok())
    }

    fn save_tab_snapshot(mut self: Pin<&mut Self>, tab_id: i32) {
        // Build each field via separate accessors so the `rust()`
        // temporary doesn't outlive the struct-literal expression.
        let snapshot = TabSnapshot {
            rows: self.as_ref().rust().rows.clone(),
            last_path: self.as_ref().rust().last_path.clone(),
            last_term: self.as_ref().rust().last_term.clone(),
            last_regex: self.as_ref().rust().last_regex,
            last_case_sensitive: self.as_ref().rust().last_case_sensitive,
            status_text: self.as_ref().rust().status_text.to_string(),
            match_count: self.as_ref().rust().match_count,
            files_matched: self.as_ref().rust().files_matched,
            files_scanned: self.as_ref().rust().files_scanned,
            has_searched: self.as_ref().rust().has_searched,
            result_mode: self.as_ref().rust().result_mode,
            within_filter: self.as_ref().rust().within_filter.to_string(),
            within_regex: self.as_ref().rust().within_regex,
            target_kind: self.as_ref().rust().target_kind,
            selected_container_id: self.as_ref().rust().selected_container_id.to_string(),
            busy: self.as_ref().rust().busy,
            replacing: self.as_ref().rust().replacing,
            last_replace_summary: self.as_ref().rust().last_replace_summary.to_string(),
        };
        self.as_mut()
            .rust_mut()
            .tab_snapshots
            .insert(tab_id, snapshot);
    }

    fn restore_tab_snapshot(mut self: Pin<&mut Self>, tab_id: i32) {
        // Clone the snapshot rather than removing it so the contract
        // is idempotent — a double-restore (or a restore-before-save
        // on session bootstrap) doesn't wipe the buffer. The snapshot
        // is dropped explicitly on tab close via `drop_tab_snapshot`.
        let snap_opt = self.as_ref().rust().tab_snapshots.get(&tab_id).cloned();

        let (
            rows,
            last_path,
            last_term,
            last_regex,
            last_case_sensitive,
            status_text,
            match_count,
            files_matched,
            files_scanned,
            has_searched,
            result_mode,
            within_filter,
            within_regex,
            target_kind,
            selected_container_id,
            busy,
            replacing,
            last_replace_summary,
        ) = match snap_opt {
            Some(snap) => (
                snap.rows,
                snap.last_path,
                snap.last_term,
                snap.last_regex,
                snap.last_case_sensitive,
                QString::from(&snap.status_text),
                snap.match_count,
                snap.files_matched,
                snap.files_scanned,
                snap.has_searched,
                snap.result_mode,
                QString::from(&snap.within_filter),
                snap.within_regex,
                snap.target_kind,
                QString::from(&snap.selected_container_id),
                snap.busy,
                snap.replacing,
                QString::from(&snap.last_replace_summary),
            ),
            None => (
                Vec::new(),
                String::new(),
                String::new(),
                false,
                false,
                QString::default(),
                0,
                0,
                0,
                false,
                0,
                QString::default(),
                false,
                0,
                QString::default(),
                false,
                false,
                QString::default(),
            ),
        };

        // Set projection-driving qproperties through the generated
        // setters before rebuilding `visible`. Writing these backing
        // fields directly would pre-stage the values and make the
        // setters below silent no-ops, leaving QML bound controls
        // stale after a tab switch.
        self.as_mut().set_result_mode(result_mode);
        self.as_mut().set_within_filter(within_filter);
        self.as_mut().set_within_regex(within_regex);
        self.as_mut().set_target_kind(target_kind);
        self.as_mut()
            .set_selected_container_id(selected_container_id);

        unsafe { self.as_mut().begin_reset_model() };
        {
            let mut s = self.as_mut().rust_mut();
            s.rows = rows;
            s.last_path = last_path;
            s.last_term = last_term;
            s.last_regex = last_regex;
            s.last_case_sensitive = last_case_sensitive;
            // Re-project: the visible vec is derived from rows +
            // result_mode + within_filter on every restore, never
            // persisted to the snapshot. This keeps the projection
            // consistent if the user changed view rules while a
            // different tab was active.
            s.rebuild_visible();
        }
        unsafe { self.as_mut().end_reset_model() };

        self.as_mut().set_status_text(status_text);
        self.as_mut().set_match_count(match_count);
        self.as_mut().set_files_matched(files_matched);
        self.as_mut().set_files_scanned(files_scanned);
        self.as_mut().set_has_searched(has_searched);
        self.as_mut().set_busy(busy);
        self.as_mut().set_replacing(replacing);
        self.as_mut().set_last_replace_summary(last_replace_summary);
    }

    fn drop_tab_snapshot(mut self: Pin<&mut Self>, tab_id: i32) {
        self.as_mut().rust_mut().tab_snapshots.remove(&tab_id);
    }

    fn export_results(&self, dest_path: &QString, format: i32) -> QString {
        let dest = std::path::PathBuf::from(dest_path.to_string());
        if dest_path.to_string().trim().is_empty() {
            return QString::from("error: destination path is empty");
        }
        let rust = self.rust();
        // Project through `visible` so the user gets exactly what's
        // on screen — within-filter + files-mode dedup are honored.
        let rows: Vec<&ResultRow> = rust
            .visible
            .iter()
            .filter_map(|&i| rust.rows.get(i))
            .collect();

        let body = match format {
            1 => export_as_json(&rows),
            2 => export_as_markdown(&rows),
            _ => export_as_csv(&rows),
        };
        match std::fs::write(&dest, body) {
            Ok(()) => QString::from(&format!("Wrote {} rows to {}", rows.len(), dest.display())),
            Err(err) => QString::from(&format!("Export failed: {err}")),
        }
    }

    fn refresh_view(mut self: Pin<&mut Self>) {
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().rebuild_visible();
        unsafe { self.as_mut().end_reset_model() };
        // match_count reflects raw match count, files_matched
        // reflects file count — those don't change on view refresh.
    }

    fn sort_results(mut self: Pin<&mut Self>, column: i32, ascending: bool) {
        unsafe { self.as_mut().begin_reset_model() };
        {
            let mut s = self.as_mut().rust_mut();
            // Stable sort so equal keys keep their search-order
            // (matches `ripgrep`'s output stability — important
            // when sorting by path with many lines per file).
            match column {
                1 => s.rows.sort_by(|a, b| {
                    let ord = a.line.cmp(&b.line).then_with(|| a.column.cmp(&b.column));
                    if ascending { ord } else { ord.reverse() }
                }),
                2 => s.rows.sort_by(|a, b| {
                    let ord = a.preview_match.cmp(&b.preview_match);
                    if ascending { ord } else { ord.reverse() }
                }),
                _ => s.rows.sort_by(|a, b| {
                    // Path sort uses the relative path string so the
                    // dedupe order is intuitive (alphabetic by
                    // file-tree, not full absolute path).
                    let ord = a.relative_path.cmp(&b.relative_path);
                    if ascending { ord } else { ord.reverse() }
                }),
            }
            s.rebuild_visible();
        }
        unsafe { self.as_mut().end_reset_model() };
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
    generation: u64,
    rows: Vec<ResultRow>,
    files_scanned: i32,
    files_matched: i32,
) {
    // Drop hops from a prior search whose generation no longer
    // matches the controller's current one — see `start_search`.
    if pin.as_ref().rust().active_generation != generation {
        return;
    }
    if rows.is_empty() {
        // Still update the scan counters — they tick on every file
        // even when no match comes out of it.
        let raw_added = pin.as_ref().rust().match_count;
        pin.as_mut().set_match_count(raw_added);
        pin.as_mut().set_files_scanned(files_scanned);
        pin.as_mut().set_files_matched(files_matched);
        return;
    }
    // Raw count is what we add to `match_count` (the user-facing
    // "total matches" — never filtered) regardless of how many
    // pass the view filter.
    let raw_added = rows.len() as i32;
    // Filter first to learn how many visible rows we're really
    // inserting. The QAbstractListModel contract requires
    // begin/endInsertRows to bracket EXACTLY the number of rows
    // `rowCount()` will grow by; announcing more than we append
    // corrupts QML's ListView indexing.
    let kept = pin.as_ref().rust().filter_batch_for_view(&rows);
    if kept.is_empty() {
        // Match count still grows — we accumulate the raw matches
        // even when the view filter hides them. Without this, the
        // status counter would be wrong when a within-filter is
        // active.
        let new_count = pin.as_ref().rust().match_count + raw_added;
        pin.as_mut().set_match_count(new_count);
        pin.as_mut().set_files_scanned(files_scanned);
        pin.as_mut().set_files_matched(files_matched);
        // Even though no row is visible, the raw rows still need to
        // land in `self.rows` so files-mode dedup against future
        // batches sees them.
        pin.as_mut().rust_mut().rows.extend(rows);
        return;
    }

    let parent = QModelIndex::default();
    let visible_first = pin.as_ref().rust().row_count();
    let visible_last = visible_first + kept.len() as i32 - 1;
    unsafe {
        pin.as_mut()
            .begin_insert_rows(&parent, visible_first, visible_last)
    };
    pin.as_mut().rust_mut().append_with_visible(rows, kept);
    unsafe { pin.as_mut().end_insert_rows() };

    let new_count = pin.as_ref().rust().match_count + raw_added;
    pin.as_mut().set_match_count(new_count);
    pin.as_mut().set_files_scanned(files_scanned);
    pin.as_mut().set_files_matched(files_matched);
}

/// Worker-thread entry point for container searches. Runs the
/// grexa-containers pipeline synchronously and returns the formatted
/// summary so the GUI-thread hop can populate the model in one shot.
fn run_container_search(
    target_kind: i32,
    container_id: &str,
    container_path: &str,
    pattern: &str,
    regex: bool,
    case_sensitive: bool,
) -> Result<Vec<ResultRow>, String> {
    use grexa_containers::{
        ContainerRuntimeKind, ContainerSearchOptions, LiveProbe, detect_runtimes,
        runtime::{CliRuntime, RuntimeOperations, SystemCommandRunner},
        search_container,
    };

    let runtimes = detect_runtimes(&LiveProbe);
    let kind_match: ContainerRuntimeKind = match target_kind {
        1 => ContainerRuntimeKind::Docker,
        2 | 3 => ContainerRuntimeKind::Podman,
        _ => return Err("unknown runtime kind".into()),
    };
    let want_rootless = target_kind == 2;
    let runtime = runtimes
        .into_iter()
        .find(|r| {
            r.kind == kind_match
                && (kind_match == ContainerRuntimeKind::Docker || r.rootless == want_rootless)
        })
        .ok_or_else(|| format!("{:?} runtime not available", kind_match))?;

    let cli = CliRuntime::new(runtime, SystemCommandRunner);
    let containers = cli
        .list_containers()
        .map_err(|e| format!("list_containers failed: {e}"))?;
    let info = containers
        .into_iter()
        .find(|c| c.id == container_id || c.id.starts_with(container_id))
        .ok_or_else(|| format!("container {container_id} not found"))?;

    let options = ContainerSearchOptions {
        container_path: container_path.to_string(),
        pattern: pattern.to_string(),
        case_sensitive,
        regex,
    };
    let summary = search_container(&cli, &info, &options)
        .map_err(|e| format!("container search failed: {e}"))?;

    let mut rows = Vec::with_capacity(summary.hits.len());
    for hit in summary.hits {
        let path_buf = std::path::PathBuf::from(&hit.container_path);
        rows.push(ResultRow {
            full_path: path_buf.clone(),
            relative_path: path_buf,
            line: hit.line_number as u32,
            column: hit.column_number as u32,
            preview_before: String::new(),
            preview_match: hit.line_content,
            preview_after: String::new(),
        });
    }
    Ok(rows)
}

fn finish_container_search(
    mut pin: Pin<&mut ffi::SearchController>,
    generation: u64,
    outcome: Result<Vec<ResultRow>, String>,
) {
    if pin.as_ref().rust().active_generation != generation {
        return;
    }
    pin.as_mut().set_busy(false);
    match outcome {
        Ok(rows) => {
            let added = rows.len() as i32;
            // Reset model + reinstall the rows in one go.
            let parent = QModelIndex::default();
            unsafe { pin.as_mut().begin_reset_model() };
            {
                let mut s = pin.as_mut().rust_mut();
                s.rows = rows;
                s.visible.clear();
            }
            // Filter through view rules.
            pin.as_mut().rust_mut().rebuild_visible();
            unsafe { pin.as_mut().end_reset_model() };
            let _ = parent;

            pin.as_mut().set_match_count(added);
            // Files matched is the unique file count in the result set.
            let unique_files: i32 = {
                let mut seen: std::collections::HashSet<std::path::PathBuf> =
                    std::collections::HashSet::new();
                for r in &pin.as_ref().rust().rows {
                    seen.insert(r.full_path.clone());
                }
                seen.len() as i32
            };
            pin.as_mut().set_files_matched(unique_files);
            pin.as_mut().set_status_text(QString::from(&format!(
                "Found {} in container ({})",
                plural_count("count-matches", added as usize),
                plural_count("count-files", unique_files as usize),
            )));
        }
        Err(err) => {
            pin.as_mut()
                .set_status_text(QString::from(&format!("Container error: {err}")));
        }
    }
    pin.as_mut().rust_mut().cancel_token = None;
    pin.as_mut().search_completed(false);
}

fn finish_replace(
    mut pin: Pin<&mut ffi::SearchController>,
    outcome: Result<grexa_core::ReplaceSummary, grexa_core::ReplaceError>,
) {
    pin.as_mut().set_replacing(false);
    match outcome {
        Ok(summary) => {
            // ReplaceSummary doesn't impl Serialize, so build the JSON
            // by hand. The QML side reads `files_modified` +
            // `matches_replaced` + `cancelled` to render the dialog.
            let json = format!(
                "{{\"files_modified\":{},\"files_unchanged\":{},\"matches_replaced\":{},\"cancelled\":{},\"elapsed_ms\":{},\"failure_count\":{}}}",
                summary.files_modified,
                summary.files_unchanged,
                summary.matches_replaced,
                summary.cancelled,
                summary.elapsed_ms,
                summary.failures.len()
            );
            pin.as_mut().set_last_replace_summary(QString::from(&json));
            // Flip the result-mode toggle to Files so the user sees
            // per-file counts (matching Grex's behavior).
            pin.as_mut().set_result_mode(1);
            unsafe { pin.as_mut().begin_reset_model() };
            pin.as_mut().rust_mut().rebuild_visible();
            unsafe { pin.as_mut().end_reset_model() };
            pin.as_mut().set_status_text(QString::from(&format!(
                "Replaced {} in {}",
                plural_count("count-matches", summary.matches_replaced),
                plural_count("count-files", summary.files_modified),
            )));
            // Replace is always notification-worthy — it rewrites
            // files on disk, so the user wants to be told.
            notify_desktop(
                "Replace complete",
                &format!(
                    "{} in {} (Grexa)",
                    plural_count("count-matches", summary.matches_replaced),
                    plural_count("count-files", summary.files_modified),
                ),
            );
            pin.as_mut().replace_completed(true);
        }
        Err(err) => {
            pin.as_mut()
                .set_status_text(QString::from(&format!("Replace error: {err}")));
            pin.as_mut().replace_completed(false);
        }
    }
}

/// CSV export — RFC 4180 with `"`-escaped fields. Header row first.
fn export_as_csv(rows: &[&ResultRow]) -> String {
    let mut buf = String::from("path,line,column,match\n");
    for r in rows {
        let path = csv_escape(&r.full_path.to_string_lossy());
        let m = csv_escape(&r.preview_match);
        use std::fmt::Write;
        let _ = writeln!(&mut buf, "{path},{},{},{m}", r.line, r.column);
    }
    buf
}

fn csv_escape(s: &str) -> String {
    let value = neutralize_spreadsheet_formula(s);
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value
    }
}

fn neutralize_spreadsheet_formula(value: &str) -> String {
    if value
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '=' | '+' | '-' | '@' | '\t' | '\r' | '\n'))
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

/// JSON export — one array of `{path, line, column, match}` objects.
fn export_as_json(rows: &[&ResultRow]) -> String {
    use serde_json::json;
    let arr: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "path": r.full_path.to_string_lossy(),
                "line": r.line,
                "column": r.column,
                "match": r.preview_match,
            })
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
}

/// Markdown table export — for sharing in PRs / issue trackers.
fn export_as_markdown(rows: &[&ResultRow]) -> String {
    let mut buf =
        String::from("| Path | Line | Column | Match |\n|------|------|--------|-------|\n");
    for r in rows {
        let path = md_escape(&r.full_path.to_string_lossy());
        let m = md_escape(&r.preview_match);
        use std::fmt::Write;
        let _ = writeln!(&mut buf, "| {path} | {} | {} | {m} |", r.line, r.column);
    }
    buf
}

fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// Substitute `{path}` / `{line}` / `{file}` placeholders in a
/// user-provided editor command template, then split on whitespace
/// to produce an argv. Quoted segments are honored. Designed for
/// the simple "kate --line {line} {path}" pattern; not a full shell.
///
/// Tokens recognized:
/// * `{path}` — absolute path passed to `open_in_editor`
/// * `{file}` — basename (last `/`-delimited segment)
/// * `{line}` — line number (1-based); empty when no line is given
fn expand_editor_template(
    template: &str,
    path: &str,
    line: Option<usize>,
) -> Vec<std::ffi::OsString> {
    let file = std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    let line_str = line.map(|n| n.to_string()).unwrap_or_default();
    let expanded = template
        .replace("{path}", path)
        .replace("{file}", &file)
        .replace("{line}", &line_str);
    split_shell_argv(&expanded)
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect()
}

/// Tiny shell-style argv splitter. Honors single and double quotes;
/// no command substitution, no variable expansion, no escaping
/// inside quotes. Anything more elaborate is the user's job —
/// they can write a wrapper script.
fn split_shell_argv(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;
    for c in s.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            c if c.is_whitespace() && !in_single && !in_double => {
                if !buf.is_empty() {
                    out.push(std::mem::take(&mut buf));
                }
            }
            c => buf.push(c),
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

/// Pick the editor preset from the persisted settings. The numeric
/// mapping matches the `editor_preset` qproperty and the order in
/// `crates/grexa-core/src/desktop.rs`.
fn editor_preset_from_settings(s: &grexa_core::DefaultSettings) -> grexa_core::EditorPreset {
    match s.editor_preset {
        0 => grexa_core::EditorPreset::Kate,
        1 => grexa_core::EditorPreset::KWrite,
        2 => grexa_core::EditorPreset::VsCode,
        3 => grexa_core::EditorPreset::VsCodium,
        4 => grexa_core::EditorPreset::SublimeText,
        5 => grexa_core::EditorPreset::JetBrains,
        6 => grexa_core::EditorPreset::GnomeTextEditor,
        7 => grexa_core::EditorPreset::Neovim,
        _ => grexa_core::EditorPreset::XdgOpen,
    }
}

/// Spawn a process detached from grexa so the editor or file manager
/// keeps running after we drop the `Child` and after grexa itself
/// quits. We redirect stdin/stdout/stderr to `/dev/null` so the
/// child doesn't share grexa's terminal, and ask Linux to drop the
/// process into a new session (`setsid`) so it survives SIGHUP when
/// the parent exits.
fn spawn_detached(argv: Vec<std::ffi::OsString>) {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;

    if argv.is_empty() {
        return;
    }
    let mut iter = argv.into_iter();
    let program = iter.next().unwrap();
    let args: Vec<std::ffi::OsString> = iter.collect();
    let mut cmd = std::process::Command::new(&program);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: `setsid` is signal-safe and side-effect-free on the
    // child's POV — we only call it in the pre-exec hook before the
    // process image is replaced.
    unsafe {
        cmd.pre_exec(|| {
            libc_setsid();
            Ok(())
        });
    }
    let _ = cmd.spawn();
}

unsafe extern "C" {
    #[link_name = "setsid"]
    fn libc_setsid() -> i32;
}

/// Best-effort `org.freedesktop.FileManager1.ShowItems` call via
/// `gdbus`. Returns `Ok(())` only on success. Falls back to the
/// xdg-open helper when this errors.
///
/// The URI string is percent-encoded so single quotes (which would
/// otherwise close the GVariant string literal) become `%27`, and
/// every other reserved character is also escaped. This matches the
/// `file://` URI form `org.freedesktop.FileManager1` expects.
fn try_filemanager1_reveal(path: &std::path::Path) -> Result<(), ()> {
    let abs = path.canonicalize().map_err(|_| ())?;
    let uri = format!("file://{}", percent_encode_path(&abs.to_string_lossy()));
    let status = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.FileManager1",
            "--object-path",
            "/org/freedesktop/FileManager1",
            "--method",
            "org.freedesktop.FileManager1.ShowItems",
        ])
        .arg(format!("['{uri}']"))
        .arg("''")
        .status()
        .map_err(|_| ())?;
    if status.success() { Ok(()) } else { Err(()) }
}

/// Percent-encode a filesystem path for inclusion in a `file://` URI.
/// Keeps `/`, ASCII alphanumerics, and the unreserved RFC 3986
/// characters (`-._~`). Encodes everything else, including `'` and
/// `"` which would otherwise terminate a GVariant string literal.
fn percent_encode_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~');
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `"1 match"` / `"0 matches"` / `"5 matches"` — singular when
/// `n == 1`. Routed through the workspace's Fluent bundle so the
/// inflection follows the user's locale rather than hardcoded English
/// rules. The valid keys today are `count-matches`, `count-files`,
/// `count-files-modified`, `count-matches-replaced`, `count-failures`
/// (all defined in `crates/grexa-i18n/locales/<lang>/grexa.ftl`).
fn plural_count(key: &str, n: usize) -> String {
    with_workspace(|w| {
        w.bundle
            .plural_count(key, n as i64)
            // Fall back to the bare count if the catalog is broken —
            // users see "5" instead of "5 matches", which is ugly but
            // doesn't crash.
            .unwrap_or_else(|_| n.to_string())
    })
}

/// Fire a desktop notification via `notify-send`. Best-effort — fails
/// silently if the binary isn't installed. We use shell-out instead
/// of D-Bus directly so the dependency surface stays in user-space
/// tooling rather than a Rust D-Bus crate.
fn notify_desktop(summary: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .arg("--app-name=Grexa")
        .arg("--icon=io.visorcraft.Grexa")
        .arg(summary)
        .arg(body)
        .spawn();
}

/// Push `text` to the system clipboard. Uses `wl-copy` (Wayland) when
/// `$WAYLAND_DISPLAY` is set; falls back to `xclip -selection clipboard`
/// otherwise. Both are commonly available on KDE/GNOME hosts and ship
/// in the Flatpak base runtimes Grexa targets.
fn copy_to_system_clipboard(text: &str) {
    let (program, args): (&str, Vec<&str>) = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        ("wl-copy", vec![])
    } else {
        ("xclip", vec!["-selection", "clipboard"])
    };
    let mut child = match std::process::Command::new(program)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(text.as_bytes());
    }
    let _ = child.wait();
}

/// Detect runtimes + list containers, return as a JSON string suitable
/// for the QML target-selector dropdown. Shape:
/// `{ "runtimes": [...], "containers": [{ "kind": 1, "rootless": true, "id": "...", "name": "...", "image": "...", "status": "..." }] }`.
/// `kind` is the same i32 mapping the qproperty uses: 1=Docker,
/// 2=Podman rootless, 3=Podman rootful.
fn build_containers_json() -> String {
    use grexa_containers::{
        ContainerRuntimeKind, LiveProbe, detect_runtimes,
        runtime::{CliRuntime, RuntimeOperations, SystemCommandRunner},
    };
    use serde_json::json;

    let runtimes = detect_runtimes(&LiveProbe);
    let mut runtime_descs: Vec<serde_json::Value> = Vec::new();
    let mut container_descs: Vec<serde_json::Value> = Vec::new();

    for runtime in runtimes {
        let kind = match (runtime.kind, runtime.rootless) {
            (ContainerRuntimeKind::Docker, _) => 1,
            (ContainerRuntimeKind::Podman, true) => 2,
            (ContainerRuntimeKind::Podman, false) => 3,
        };
        let label = match (runtime.kind, runtime.rootless) {
            (ContainerRuntimeKind::Docker, _) => "Docker".to_string(),
            (ContainerRuntimeKind::Podman, true) => "Podman (rootless)".to_string(),
            (ContainerRuntimeKind::Podman, false) => "Podman (rootful)".to_string(),
        };
        runtime_descs.push(json!({
            "kind": kind,
            "label": label,
            "available": runtime.is_available(),
        }));

        if !runtime.is_available() {
            continue;
        }
        let cli = CliRuntime::new(runtime, SystemCommandRunner);
        let listed = cli.list_containers().unwrap_or_default();
        for c in listed {
            container_descs.push(json!({
                "kind": kind,
                "id": c.id,
                "name": c.name,
                "image": c.image,
                "status": c.status,
                "state": c.state,
            }));
        }
    }

    serde_json::to_string(&json!({
        "runtimes": runtime_descs,
        "containers": container_descs,
    }))
    .unwrap_or_else(|_| "{\"runtimes\":[],\"containers\":[]}".into())
}

fn finish_search(
    mut pin: Pin<&mut ffi::SearchController>,
    generation: u64,
    outcome: Result<grexa_core::SearchSummary, grexa_core::SearchError>,
    cancelled: bool,
    path_str: &str,
) {
    // Same generation gate as `push_rows`: ignore late finishes
    // from a search that the user has already superseded.
    if pin.as_ref().rust().active_generation != generation {
        return;
    }
    pin.as_mut().set_busy(false);
    match outcome {
        Ok(summary) => {
            // The progress sink only queues row batches. A zero-match
            // search may never call `push_rows`, and multi-hit lines
            // can make row count differ from total match count, so
            // the final summary is the authoritative source for the
            // user-facing counters.
            pin.as_mut()
                .set_match_count(usize_to_i32_saturating(summary.matches));
            pin.as_mut()
                .set_files_scanned(usize_to_i32_saturating(summary.files_scanned));
            pin.as_mut()
                .set_files_matched(usize_to_i32_saturating(summary.files_matched));

            let path = PathBuf::from(path_str);
            with_workspace(|w| {
                let _ = w.recent_paths.add(path);
            });
            let history_added = if cancelled {
                false
            } else {
                let term = pin.as_ref().rust().last_term.clone();
                let regex = pin.as_ref().rust().last_regex;
                let case_sensitive = pin.as_ref().rust().last_case_sensitive;
                let files_search = pin.as_ref().rust().result_mode == 1;
                with_workspace(|w| {
                    let settings = w.settings.load().unwrap_or_default();
                    let mut options = SearchOptions::new(PathBuf::from(path_str), term);
                    options.regex = regex;
                    options.case_sensitive = case_sensitive;
                    options.respect_gitignore = settings.respect_gitignore;
                    options.include_hidden = settings.include_hidden_items;
                    options.include_binary = settings.include_binary_files;
                    options.include_system = settings.include_system_files;
                    options.include_subfolders = settings.include_subfolders;
                    options.include_symlinks = settings.include_symbolic_links;
                    options.match_file_names = settings.default_match_files;
                    options.exclude_dirs = settings.default_exclude_dirs;
                    w.history
                        .add(grexa_core::RecentSearch::from_options(
                            &options,
                            files_search,
                            summary.matches,
                        ))
                        .is_ok()
                })
            };
            let recent_count =
                with_workspace(|w| w.recent_paths.load().unwrap_or_default().len() as i32);
            let previous = pin.as_ref().rust().recent_path_count;
            let mut emitted_history_changed = false;
            if recent_count != previous {
                pin.as_mut().set_recent_path_count(recent_count);
                pin.as_mut().history_changed();
                emitted_history_changed = true;
            }
            if history_added && !emitted_history_changed {
                pin.as_mut().history_changed();
            }
            let mc = pin.as_ref().rust().match_count as usize;
            let fc = pin.as_ref().rust().files_matched as usize;
            let status = if cancelled {
                format!(
                    "Cancelled — {} in {}",
                    plural_count("count-matches", mc),
                    plural_count("count-files", fc),
                )
            } else {
                format!(
                    "Found {} in {} in {} ms",
                    plural_count("count-matches", summary.matches),
                    plural_count("count-files", summary.files_matched),
                    summary.elapsed_ms,
                )
            };
            pin.as_mut().set_status_text(QString::from(&status));
            // Fire a desktop notification when the search ran longer
            // than the 4-second threshold (matches Grex's behavior).
            // Cancelled and zero-match searches are skipped — the user
            // is probably looking at the window in those cases.
            if !cancelled && summary.elapsed_ms >= 4000 && summary.matches > 0 {
                notify_desktop(
                    "Search complete",
                    &format!(
                        "{} in {} (Grexa)",
                        plural_count("count-matches", summary.matches),
                        plural_count("count-files", summary.files_matched),
                    ),
                );
            }
        }
        Err(err) => {
            pin.as_mut()
                .set_status_text(QString::from(&format!("Error: {err}")));
        }
    }
    pin.as_mut().rust_mut().cancel_token = None;
    pin.as_mut().search_completed(cancelled);
}

fn usize_to_i32_saturating(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
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
    fn cancelling_invalidates_queued_worker_generation() {
        let mut state = SearchControllerRust::default();
        let token = CancelToken::new();
        state.cancel_token = Some(token.clone());
        state.active_generation = 41;

        state.cancel_active_search();

        assert!(token.is_cancelled());
        assert!(state.cancel_token.is_none());
        assert_eq!(state.active_generation, 42);
    }

    #[test]
    fn clearing_search_state_removes_stale_replace_target_and_status() {
        let mut state = SearchControllerRust {
            status_text: QString::from("Found 1 match"),
            match_count: 1,
            files_matched: 1,
            files_scanned: 2,
            has_searched: true,
            last_path: "/tmp/project".into(),
            last_term: "TODO".into(),
            last_regex: true,
            last_case_sensitive: true,
            ..Default::default()
        };
        state.append_batch(vec![ResultRow {
            full_path: PathBuf::from("/tmp/project/a.rs"),
            relative_path: PathBuf::from("a.rs"),
            line: 1,
            column: 1,
            preview_before: String::new(),
            preview_match: "TODO".into(),
            preview_after: String::new(),
        }]);

        state.clear_search_state();

        assert_eq!(state.row_count(), 0);
        assert_eq!(state.match_count, 0);
        assert_eq!(state.files_matched, 0);
        assert_eq!(state.files_scanned, 0);
        assert!(!state.has_searched);
        assert!(state.status_text.is_empty());
        assert!(state.last_path.is_empty());
        assert!(state.last_term.is_empty());
        assert!(!state.last_regex);
        assert!(!state.last_case_sensitive);
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

    #[test]
    fn csv_export_neutralizes_spreadsheet_formulas() {
        let row = ResultRow {
            full_path: PathBuf::from("=cmd.txt"),
            relative_path: PathBuf::from("=cmd.txt"),
            line: 1,
            column: 1,
            preview_before: String::new(),
            preview_match: "=HYPERLINK(\"https://example.invalid\",\"TODO\")".into(),
            preview_after: String::new(),
        };
        let rows = vec![&row];

        let csv = export_as_csv(&rows);

        assert!(csv.contains("\"'=HYPERLINK(\"\"https://example.invalid\"\""));
        assert!(csv.contains("'=cmd.txt"));
    }
}
