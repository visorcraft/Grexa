// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Thread-local handle to the shared `Workspace`.
//!
//! cxx-qt's `qml_element` registration requires the QObject to be
//! `Default`, so the workspace can't be passed via constructor. We
//! install it from `main.rs` before booting the QML engine; every
//! QObject method accesses it via `with_workspace(...)`.
//!
//! The `Rc<RefCell<…>>` is intentionally GUI-thread-only. The async
//! search worker (see `search.rs`) does NOT touch the workspace
//! directly; it owns a clone of `SearchOptions` + `CancelToken`,
//! and the results are routed back through `cxx_qt::Threading::queue`
//! before they touch any workspace state.

use std::cell::RefCell;
use std::rc::Rc;

use crate::workspace::Workspace;

thread_local! {
    static WORKSPACE: RefCell<Option<Rc<RefCell<Workspace>>>> = const { RefCell::new(None) };
}

/// Install the shared workspace. Must be called once on the GUI thread
/// before any QObject method runs.
pub fn install_workspace(workspace: Rc<RefCell<Workspace>>) {
    WORKSPACE.with(|cell| *cell.borrow_mut() = Some(workspace));
}

/// Run `f` against the installed workspace.
///
/// Panics if `install_workspace` hasn't been called yet — that would
/// be a wiring bug in `main.rs`, not a user-facing condition.
pub fn with_workspace<R>(f: impl FnOnce(&mut Workspace) -> R) -> R {
    WORKSPACE.with(|cell| {
        let binding = cell.borrow();
        let workspace = binding
            .as_ref()
            .expect("install_workspace must be called before any QObject method");
        let mut w = workspace.borrow_mut();
        f(&mut w)
    })
}
