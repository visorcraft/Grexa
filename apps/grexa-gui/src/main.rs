// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Grexa GUI shell entry point.
//!
//! Uses `cxx-qt` 0.8 for the Rust ⇄ Qt bridge. The QObjects defined in
//! `qobjects.rs` are auto-registered with QML under
//! `com.visorcraft.Grexa 1.0` by the `qml_module()` declaration in
//! `build.rs`. The QML files in `apps/grexa-gui/qml/` are bundled into
//! the binary via Qt's resource system and loaded from
//! `qrc:/qt/qml/com/visorcraft/Grexa/Main.qml`.
//!
//! Runtime contract:
//!
//! 1. Initialize structured logging (mirrors `grexa-cli`).
//! 2. Build the shared `Workspace` and install it via
//!    `qobjects::install_workspace`.
//! 3. Initialize cxx-qt's static initializers
//!    (`cxx_qt::init_crate!` + `cxx_qt::init_qml_module!`).
//! 4. Boot a `QGuiApplication` + `QQmlApplicationEngine`, load
//!    `qrc:/qt/qml/com/visorcraft/Grexa/Main.qml`, run the event loop.

use std::cell::RefCell;
use std::rc::Rc;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use grexa_core::AppPaths;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

mod controller;
mod qobjects;
mod status;
mod tab;
mod workspace;

fn main() {
    let _log_guard = init_tracing();
    tracing::info!("Grexa GUI shell starting");

    // Trigger cxx-qt-lib's and our crate's static initializers so the
    // QML module and QObject types are registered before the engine
    // tries to resolve `import com.visorcraft.Grexa 1.0`.
    cxx_qt::init_crate!(cxx_qt_lib);
    cxx_qt::init_crate!(grexa);
    cxx_qt::init_qml_module!("com.visorcraft.Grexa");

    let workspace = Rc::new(RefCell::new(workspace::Workspace::new()));
    qobjects::install_workspace(workspace);

    let mut app = QGuiApplication::new();
    if app.is_null() {
        tracing::error!("could not construct QGuiApplication — exiting");
        return;
    }
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_application_name(&QString::from("Grexa"));
        app.as_mut()
            .set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
        app.as_mut()
            .set_organization_name(&QString::from("VisorCraft"));
        app.as_mut()
            .set_organization_domain(&QString::from("visorcraft.io"));
    }
    let mut engine = QQmlApplicationEngine::new();
    if engine.is_null() {
        tracing::error!("could not construct QQmlApplicationEngine — exiting");
        return;
    }

    if let Some(engine) = engine.as_mut() {
        let url = QUrl::from("qrc:/qt/qml/com/visorcraft/Grexa/Main.qml");
        engine.load(&url);
    }

    if let Some(app) = app.as_mut() {
        let code = app.exec();
        if code != 0 {
            tracing::warn!("Qt event loop exited with code {code}");
        }
    }
}

fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter =
        EnvFilter::try_from_env("GREXA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);

    let paths = AppPaths::from_env();
    let log_path = paths.state_dir.join("grexa-gui.log");
    let (file_layer, guard) = match std::fs::create_dir_all(&paths.state_dir) {
        Ok(()) => match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => {
                let (writer, guard) = tracing_appender::non_blocking(file);
                let layer = tracing_subscriber::fmt::layer()
                    .with_writer(writer)
                    .with_target(true)
                    .with_ansi(false)
                    .with_level(true);
                (Some(layer), Some(guard))
            }
            Err(_) => (None, None),
        },
        Err(_) => (None, None),
    };

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer);
    if let Some(layer) = file_layer {
        registry.with(layer).init();
    } else {
        registry.init();
    }
    guard
}
