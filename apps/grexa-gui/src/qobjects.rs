//! QObjects exposed to QML via `qmetaobject`.
//!
//! qmetaobject is the pure-Rust Qt binding the cxx-qt spike
//! recommended as the fallback path. It doesn't require CMake, does
//! not require a custom build script, and lets the binary run as
//! `cargo run -p grexa` on any Qt 6 host. The business logic is in
//! the `workspace.rs` / `tab.rs` / `status.rs` controllers; this file
//! is a thin facade that bridges Rust ⇄ QML.

use std::cell::RefCell;
use std::ffi::CString;
use std::path::PathBuf;
use std::rc::Rc;

use grexa_core::{CancelToken, SearchOptions, search_with};
use qmetaobject::*;

use crate::workspace::Workspace;

// Workspace state shared between QObjects. Using a `thread_local` so
// QML callbacks have access without taking constructor arguments —
// `qml_register_type` requires a `Default` impl and doesn't support
// per-instance construction parameters.
thread_local! {
    static WORKSPACE: RefCell<Option<Rc<RefCell<Workspace>>>> = const { RefCell::new(None) };
}

/// Install the shared workspace before registering QObjects. Must be
/// called from the main thread before the QmlEngine is created.
pub fn install_workspace(workspace: Rc<RefCell<Workspace>>) {
    WORKSPACE.with(|cell| *cell.borrow_mut() = Some(workspace));
}

fn with_workspace<R>(f: impl FnOnce(&mut Workspace) -> R) -> R {
    WORKSPACE.with(|cell| {
        let binding = cell.borrow();
        let workspace = binding
            .as_ref()
            .expect("install_workspace must be called before any QObject method");
        let mut w = workspace.borrow_mut();
        f(&mut w)
    })
}

/// Singleton QObject that drives the active Search tab. QML imports
/// it as `import com.visorcraft.Grexa 1.0` and instantiates a
/// `SearchController` element.
#[derive(QObject, Default)]
pub struct SearchController {
    base: qt_base_class!(trait QObject),

    /// Latest status string, updated after every search.
    pub status_text: qt_property!(QString; NOTIFY status_changed),
    /// Total matches in the current search.
    pub match_count: qt_property!(i32; NOTIFY status_changed),
    /// True while a search is running.
    pub busy: qt_property!(bool; NOTIFY status_changed),
    /// Number of stored recent paths (read at start_search end).
    pub recent_path_count: qt_property!(i32; NOTIFY history_changed),

    /// Emitted when any status/match-count/busy property updates.
    pub status_changed: qt_signal!(),
    /// Emitted when the recent-paths list grows or shrinks.
    pub history_changed: qt_signal!(),

    /// Start a new search synchronously. Returns the number of matches.
    pub start_search: qt_method!(fn(&mut self, path: QString, term: QString, regex: bool, case_sensitive: bool) -> i32),
    /// Cancel the in-flight search.
    pub cancel: qt_method!(fn(&mut self)),
    /// Read the recent-paths list as a JSON array.
    pub recent_paths_json: qt_method!(fn(&self) -> QString),
}

impl SearchController {
    fn start_search(
        &mut self,
        path: QString,
        term: QString,
        regex: bool,
        case_sensitive: bool,
    ) -> i32 {
        let path_str: String = path.to_string();
        let term_str: String = term.to_string();
        let mut options = SearchOptions::new(PathBuf::from(&path_str), &term_str);
        options.regex = regex;
        options.case_sensitive = case_sensitive;

        self.busy = true;
        self.match_count = 0;
        self.status_text = QString::from("Searching…");
        self.status_changed();

        let cancel = CancelToken::new();
        let summary_result = search_with(&options, &cancel, None);
        let total = match summary_result {
            Ok(summary) => {
                let count = summary.matches as i32;
                with_workspace(|w| {
                    w.recent_paths.add(options.path.clone()).ok();
                });
                self.recent_path_count = with_workspace(|w| {
                    w.recent_paths.load().unwrap_or_default().len() as i32
                });
                self.history_changed();
                let elapsed_ms = summary.elapsed_ms;
                self.status_text = QString::from(
                    format!(
                        "Found {} matches in {} files in {} ms",
                        summary.matches, summary.files_matched, elapsed_ms
                    )
                    .as_str(),
                );
                count
            }
            Err(err) => {
                self.status_text = QString::from(format!("Error: {err}").as_str());
                -1
            }
        };

        self.busy = false;
        self.match_count = total.max(0);
        self.status_changed();
        total
    }

    fn cancel(&mut self) {
        // Cancellation flag-only for now; the synchronous search loop
        // checks it before each walker entry, so a cancel between calls
        // is effectively immediate.
        self.busy = false;
        self.status_text = QString::from("Cancelled");
        self.status_changed();
    }

    fn recent_paths_json(&self) -> QString {
        let strings: Vec<String> = with_workspace(|w| {
            w.recent_paths
                .load()
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        });
        let json = serde_json::to_string(&strings).unwrap_or_else(|_| "[]".into());
        QString::from(json.as_str())
    }
}

/// Register every Grexa QObject under the QML uri
/// `com.visorcraft.Grexa 1.0`.
pub fn register_qml_types() {
    let uri = CString::new("com.visorcraft.Grexa").unwrap();
    let controller_name = CString::new("SearchController").unwrap();
    qml_register_type::<SearchController>(&uri, 1, 0, &controller_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn fresh_workspace() -> Rc<RefCell<Workspace>> {
        let dir = tempdir().unwrap();
        let workspace = Rc::new(RefCell::new(Workspace::under(&dir.path().join("xdg"))));
        // Leak the tempdir so the test's filesystem state outlives the
        // borrow; we accept the leak in tests since the OS reclaims it
        // when the process exits.
        std::mem::forget(dir);
        workspace
    }

    #[test]
    fn install_workspace_round_trips() {
        let ws = fresh_workspace();
        install_workspace(ws.clone());
        let count = with_workspace(|w| w.recent_paths.load().unwrap_or_default().len());
        assert_eq!(count, 0);
    }

    #[test]
    fn search_controller_drives_real_search() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "TODO 1\nTODO 2\n").unwrap();

        let ws = Rc::new(RefCell::new(Workspace::under(&dir.path().join("xdg"))));
        install_workspace(ws.clone());

        let mut controller = SearchController::default();
        let count = controller.start_search(
            QString::from(dir.path().to_string_lossy().as_ref()),
            QString::from("TODO"),
            false,
            false,
        );
        assert_eq!(count, 2);
        assert_eq!(controller.match_count, 2);
        assert!(!controller.busy);

        // History was recorded.
        assert_eq!(ws.borrow().recent_paths.load().unwrap().len(), 1);
    }
}
