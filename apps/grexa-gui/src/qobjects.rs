//! QObjects exposed to QML via `cxx-qt` 0.8.
//!
//! The Rust ⇄ Qt bridge is now cxx-qt's compile-time-generated
//! bindings — pure Cargo, no CMake, no qmetaobject crate. The
//! business logic still lives in the `workspace.rs` / `tab.rs` /
//! `status.rs` controllers; this file is a thin facade that exposes
//! the workspace state to QML.

use std::cell::RefCell;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use grexa_core::{CancelToken, SearchOptions, search_with};

use crate::workspace::Workspace;

// Workspace state shared between QObjects. Uses a `thread_local` so
// QML callbacks have access without taking constructor arguments —
// cxx-qt registers the QObject with QML and needs `Default`.
thread_local! {
    static WORKSPACE: RefCell<Option<Rc<RefCell<Workspace>>>> = const { RefCell::new(None) };
}

/// Install the shared workspace before booting the QML engine.
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

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, status_text)]
        #[qproperty(i32, match_count)]
        #[qproperty(bool, busy)]
        #[qproperty(i32, recent_path_count)]
        type SearchController = super::SearchControllerRust;

        /// Run a synchronous search. Returns the match count, or -1 on
        /// error. Updates `status_text`, `match_count`, `busy`, and
        /// `recent_path_count` along the way.
        #[qinvokable]
        fn start_search(
            self: Pin<&mut SearchController>,
            path: &QString,
            term: &QString,
            regex: bool,
            case_sensitive: bool,
        ) -> i32;

        /// Cancel the in-flight search. The current implementation runs
        /// synchronously, so this just updates the status text.
        #[qinvokable]
        fn cancel(self: Pin<&mut SearchController>);

        /// Read the recent-paths list as a JSON array.
        #[qinvokable]
        fn recent_paths_json(self: &SearchController) -> QString;

        /// Fired when the recent-paths list grows or shrinks.
        #[qsignal]
        fn history_changed(self: Pin<&mut SearchController>);
    }
}

/// Rust-side state for the `SearchController` QObject. Owned by the
/// generated C++ class via `super::SearchControllerRust`.
#[derive(Default)]
pub struct SearchControllerRust {
    status_text: QString,
    match_count: i32,
    busy: bool,
    recent_path_count: i32,
}

impl SearchControllerRust {
    /// Pure Rust core of the search invocation. Tests use this
    /// directly to skip Qt object construction.
    pub fn run_search(&mut self, path: &str, term: &str, regex: bool, case_sensitive: bool) -> i32 {
        let mut options = SearchOptions::new(PathBuf::from(path), term);
        options.regex = regex;
        options.case_sensitive = case_sensitive;

        self.busy = true;
        self.match_count = 0;
        self.status_text = QString::from("Searching…");

        let cancel = CancelToken::new();
        let summary_result = search_with(&options, &cancel, None);
        let total = match summary_result {
            Ok(summary) => {
                let count = summary.matches as i32;
                with_workspace(|w| {
                    w.recent_paths.add(options.path.clone()).ok();
                });
                self.recent_path_count =
                    with_workspace(|w| w.recent_paths.load().unwrap_or_default().len() as i32);
                self.status_text = QString::from(&format!(
                    "Found {} matches in {} files in {} ms",
                    summary.matches, summary.files_matched, summary.elapsed_ms
                ));
                count
            }
            Err(err) => {
                self.status_text = QString::from(&format!("Error: {err}"));
                -1
            }
        };

        self.busy = false;
        self.match_count = total.max(0);
        total
    }

    pub fn run_cancel(&mut self) {
        self.busy = false;
        self.status_text = QString::from("Cancelled");
    }

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

impl qobject::SearchController {
    fn start_search(
        mut self: Pin<&mut Self>,
        path: &QString,
        term: &QString,
        regex: bool,
        case_sensitive: bool,
    ) -> i32 {
        let path_str = path.to_string();
        let term_str = term.to_string();
        let history_before = self.as_ref().rust().recent_path_count;

        let total =
            self.as_mut()
                .rust_mut()
                .run_search(&path_str, &term_str, regex, case_sensitive);

        let new_status = self.as_ref().rust().status_text.clone();
        let new_match_count = self.as_ref().rust().match_count;
        let new_busy = self.as_ref().rust().busy;
        let new_recent_count = self.as_ref().rust().recent_path_count;

        self.as_mut().set_status_text(new_status);
        self.as_mut().set_match_count(new_match_count);
        self.as_mut().set_busy(new_busy);
        if new_recent_count != history_before {
            self.as_mut().set_recent_path_count(new_recent_count);
            self.history_changed();
        }
        total
    }

    fn cancel(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().run_cancel();
        let new_status = self.as_ref().rust().status_text.clone();
        let new_busy = self.as_ref().rust().busy;
        self.as_mut().set_status_text(new_status);
        self.as_mut().set_busy(new_busy);
    }

    fn recent_paths_json(&self) -> QString {
        QString::from(&self.rust().recent_paths_json_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn fresh_workspace() -> Rc<RefCell<Workspace>> {
        let dir = tempdir().unwrap();
        let workspace = Rc::new(RefCell::new(Workspace::under(&dir.path().join("xdg"))));
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

        let mut state = SearchControllerRust::default();
        let count = state.run_search(dir.path().to_string_lossy().as_ref(), "TODO", false, false);
        assert_eq!(count, 2);
        assert_eq!(state.match_count, 2);
        assert!(!state.busy);
        assert_eq!(ws.borrow().recent_paths.load().unwrap().len(), 1);
    }

    #[test]
    fn cancel_sets_cancelled_status() {
        let mut state = SearchControllerRust {
            busy: true,
            ..Default::default()
        };
        state.run_cancel();
        assert!(!state.busy);
        assert_eq!(state.status_text.to_string(), "Cancelled");
    }

    #[test]
    fn recent_paths_json_round_trips() {
        let dir = tempdir().unwrap();
        let ws = Rc::new(RefCell::new(Workspace::under(&dir.path().join("xdg"))));
        install_workspace(ws.clone());

        let state = SearchControllerRust::default();
        let json = state.recent_paths_json_string();
        assert_eq!(json, "[]");
    }
}
