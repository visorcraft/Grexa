// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! QObjects exposed to QML via `cxx-qt` 0.8.
//!
//! Each submodule owns one `#[cxx_qt::bridge]` and the Rust-side
//! state struct backing its QObjects. The cxx-qt build script in
//! `apps/grexa-gui/build.rs` lists every source file here so the
//! generated C++ side is compiled and linked into the binary, and
//! every `#[qml_element]` is auto-registered under
//! `com.visorcraft.Grexa 1.0`.

pub mod workspace_handle;

pub mod ai;
pub mod regex_builder;
pub mod search;
pub mod settings;

pub use workspace_handle::install_workspace;
