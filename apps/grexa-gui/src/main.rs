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

use cxx_qt_lib::{QFont, QGuiApplication, QQmlApplicationEngine, QString, QUrl};
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

    // Single-instance guard. We use a UNIX advisory lock on a
    // lockfile in $XDG_RUNTIME_DIR so a second `grexa` invocation
    // exits cleanly rather than spawning a duplicate window.
    // Best-effort: when the runtime dir is unavailable (some
    // containers / sandboxes) the check is skipped.
    let _instance_lock = match acquire_single_instance_lock() {
        Some(file) => Some(file),
        None => {
            // `acquire_single_instance_lock` returns None for both
            // "couldn't open the lockfile" (continue silently) and
            // "lock was held by another process" (exit). The
            // distinction is signaled via the GREXA_INSTANCE_BUSY
            // env var the helper sets when it sees a held lock.
            if std::env::var_os("GREXA_INSTANCE_BUSY").is_some() {
                tracing::info!(
                    "Another Grexa instance is already running. Exiting; the existing window remains active."
                );
                return;
            }
            None
        }
    };

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

    // App-wide font. Inter first; fall through to Cantarell (GNOME),
    // Noto Sans (most distros), then the platform default.
    let mut font = QFont::default();
    font.set_family(&QString::from("Inter, Cantarell, Noto Sans, Sans Serif"));
    font.set_pixel_size(13);
    if let Some(app) = app.as_mut() {
        app.set_application_font(&font);
    }
    let mut engine = QQmlApplicationEngine::new();
    if engine.is_null() {
        tracing::error!("could not construct QQmlApplicationEngine — exiting");
        return;
    }

    // Wire `objectCreationFailed` so a broken QML payload yields a
    // loud log line instead of an empty window with a silent event
    // loop. The signal fires for every root URL that fails to load
    // (Qt 6.4+); the only realistic root in our binary is Main.qml,
    // so any fire is fatal — set a flag and short-circuit before
    // `exec()` starts.
    let load_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if let Some(engine) = engine.as_mut() {
        let flag = load_failed.clone();
        engine
            .on_object_creation_failed(move |_pin, url| {
                tracing::error!("QML failed to load: {}", url.to_string());
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            })
            .release();
    }

    if let Some(engine) = engine.as_mut() {
        let url = QUrl::from("qrc:/qt/qml/com/visorcraft/Grexa/qml/Main.qml");
        engine.load(&url);
    }

    if load_failed.load(std::sync::atomic::Ordering::SeqCst) {
        tracing::error!("QML payload did not instantiate — exiting before the event loop.");
        std::process::exit(2);
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

/// Try to acquire an exclusive `flock` on `$XDG_RUNTIME_DIR/grexa.lock`.
/// Returns:
/// * `Some(File)` — we hold the lock; keep the file alive for the
///   process's lifetime.
/// * `None` and sets `GREXA_INSTANCE_BUSY=1` — another instance holds
///   the lock; the caller should exit.
/// * `None` and no env var — the lockfile couldn't be opened (no
///   runtime dir / not writable); continue without single-instance
///   guarantees.
fn acquire_single_instance_lock() -> Option<std::fs::File> {
    use std::os::fd::AsRawFd;
    // Prefer $XDG_RUNTIME_DIR (a tmpfs that's wiped on logout), fall
    // back to $XDG_CACHE_HOME/grexa, then $HOME/.cache/grexa. The
    // app-specific subdirectory keeps the lockfile from littering the
    // cache root and matches our `~/.config/grexa` / `~/.local/share/
    // grexa` convention.
    let lock_dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(|d| {
            let mut p = std::path::PathBuf::from(d);
            p.push("grexa");
            p
        })
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME").map(|c| {
                let mut p = std::path::PathBuf::from(c);
                p.push("grexa");
                p
            })
        })
        .or_else(|| {
            std::env::var_os("HOME").map(|h| {
                let mut p = std::path::PathBuf::from(h);
                p.push(".cache");
                p.push("grexa");
                p
            })
        })?;
    if std::fs::create_dir_all(&lock_dir).is_err() {
        return None;
    }
    let lock_path = lock_dir.join("grexa.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        // The lockfile is empty by contract — `flock` operates on the
        // fd, not the contents. Truncate clears anything that ever
        // landed in there and makes the `create+write` open mode
        // unambiguous to the `clippy::suspicious_open_options` lint.
        .truncate(true)
        .open(&lock_path)
        .ok()?;
    // `flock` is the simplest Linux primitive for an advisory
    // single-instance lock. LOCK_EX | LOCK_NB returns EWOULDBLOCK
    // (errno 11) when another process holds the lock.
    let fd = file.as_raw_fd();
    let rc = unsafe { libc_flock(fd, LOCK_EX | LOCK_NB) };
    if rc == 0 {
        Some(file)
    } else {
        // SAFETY: env var manipulation. We're single-threaded at this
        // point (Qt hasn't booted yet).
        unsafe {
            std::env::set_var("GREXA_INSTANCE_BUSY", "1");
        }
        None
    }
}

// Minimal libc binding for `flock`. We don't pull in the `libc` crate
// for this single call — the syscall ABI is stable and the symbol
// lives in glibc, musl, and Bionic.
unsafe extern "C" {
    #[link_name = "flock"]
    fn libc_flock(fd: i32, operation: i32) -> i32;
}

const LOCK_EX: i32 = 2;
const LOCK_NB: i32 = 4;
