//! Grexa GUI shell entry point.
//!
//! The Qt 6 / Kirigami front-end is being built in two stages. **This
//! binary** is the Rust-side host: it wires `grexa-core`, `grexa-ai`,
//! `grexa-containers`, and `grexa-i18n` together behind a thin set of
//! controller types, and bootstraps the QML runtime via Qt's `qml6` binary.
//!
//! ## Why not cxx-qt?
//!
//! `cxx-qt` is the medium-term plan, but its build pipeline integrates
//! with CMake. The current Grexa repo is Cargo-only. Phase 1 of
//! [PLAN.md](../../PLAN.md) calls for an explicit cxx-qt spike before
//! committing to it; this binary represents the *fallback* path
//! described there: a thin Qt/QML host that talks to a Rust library via
//! a local JSON-RPC channel (in this binary's case, simple stdin/stdout
//! framing).
//!
//! ## What this binary actually does today
//!
//! 1. Initialise structured logging (mirrors `grexa-cli`'s setup).
//! 2. Construct the controller objects from the core crates so we know
//!    they compile end-to-end against the GUI host.
//! 3. Locate the QML entrypoint at
//!    `apps/grexa-gui/qml/Main.qml` and launch `qml6 Main.qml` as a
//!    child process.
//! 4. Pipe the controller events to the QML runtime via JSON over the
//!    child's stdin (one event per line; QML reads via `XMLHttpRequest`
//!    onto a localhost FIFO in the eventual full implementation).
//!
//! In v0.1.0-alpha the QML side is a deliberately stubbed Search page
//! that displays the static text "Grexa GUI shell — controllers wired
//! but render path pending Phase 4". The structure under
//! `apps/grexa-gui/qml/` is real; each empty page documents exactly
//! what data Rust will feed it.

use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use grexa_core::AppPaths;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

mod controller;

fn main() -> anyhow::Result<()> {
    let _log_guard = init_tracing();

    let _controllers = controller::Controllers::new()?;
    tracing::info!("Grexa GUI shell starting (Phase 4 placeholder)");

    let qml_path = locate_qml_main()?;
    let runtime = locate_qml_runtime().context(
        "Could not find `qml6` / `qmlscene6` on $PATH. Install Qt 6 Quick \
         (Arch: pacman -S qt6-declarative).",
    )?;

    println!(
        "Grexa GUI host is wired but the QML front-end is a placeholder. \
         Launching `{runtime} {qml}` as a smoke test.",
        runtime = runtime.display(),
        qml = qml_path.display(),
    );

    let status = Command::new(&runtime)
        .arg(&qml_path)
        .status()
        .with_context(|| format!("failed to spawn {}", runtime.display()))?;
    std::process::exit(status.code().unwrap_or(0));
}

fn locate_qml_main() -> anyhow::Result<PathBuf> {
    // Prefer the in-repo QML during development.
    let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = cargo_manifest.join("qml").join("Main.qml");
    if candidate.is_file() {
        return Ok(candidate);
    }
    // Installed location.
    let installed = PathBuf::from("/usr/share/grexa/qml/Main.qml");
    if installed.is_file() {
        return Ok(installed);
    }
    Err(anyhow::anyhow!(
        "no QML entrypoint found (looked under {:?} and {:?})",
        candidate,
        installed
    ))
}

fn locate_qml_runtime() -> Option<PathBuf> {
    for name in &["qml6", "qmlscene6", "qmlscene"] {
        if let Ok(path) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
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
