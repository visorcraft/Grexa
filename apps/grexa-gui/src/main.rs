//! Grexa GUI shell entry point.
//!
//! Uses `qmetaobject` for the Rust ⇄ Qt bridge. The cxx-qt spike
//! (`docs/gui-design.md`) recommended this as the pure-Rust fallback
//! when CMake isn't on the dev / CI host; cxx-qt 0.8 expects a
//! CMake-driven Interface pipeline that's hostile to a Cargo-only
//! workspace.
//!
//! Runtime contract:
//!
//! 1. Initialize structured logging (mirrors `grexa-cli`).
//! 2. Build the shared `Workspace` and install it via
//!    `qobjects::install_workspace`.
//! 3. Register the Grexa QObjects (`SearchController`) under the
//!    `com.visorcraft.Grexa 1.0` QML module.
//! 4. Boot a `QmlEngine`, load `qml/Main.qml` from
//!    `CARGO_MANIFEST_DIR` during development or
//!    `/usr/share/grexa/qml/` when installed.
//! 5. Enter the Qt event loop.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use grexa_core::AppPaths;
use qmetaobject::*;
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

    let workspace = Rc::new(RefCell::new(workspace::Workspace::new()));
    qobjects::install_workspace(workspace);
    qobjects::register_qml_types();

    let mut engine = QmlEngine::new();
    match locate_qml_main() {
        Some(path) => {
            engine.load_file(path.to_string_lossy().to_string().into());
        }
        None => {
            // Inline smoke QML — used only when neither the dev path
            // nor the installed path is reachable. Confirms the
            // Rust→QML registration works.
            engine.load_data(
                r#"
import QtQuick
import com.visorcraft.Grexa 1.0
Item {
    SearchController { id: ctl }
    Component.onCompleted: {
        console.log("Grexa GUI smoke:", ctl.statusText)
        Qt.quit()
    }
}
"#
                .into(),
            );
        }
    }
    engine.exec();
}

fn locate_qml_main() -> Option<PathBuf> {
    let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = cargo_manifest.join("qml").join("Main.qml");
    if candidate.is_file() {
        return Some(candidate);
    }
    let installed = PathBuf::from("/usr/share/grexa/qml/Main.qml");
    if installed.is_file() {
        return Some(installed);
    }
    None
}

fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter = EnvFilter::try_from_env("GREXA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
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

    let registry = tracing_subscriber::registry().with(env_filter).with(stderr_layer);
    if let Some(layer) = file_layer {
        registry.with(layer).init();
    } else {
        registry.init();
    }
    guard
}
