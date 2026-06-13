// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Grexa GUI shell entry point.
//!
//! Uses `cxx-qt` 0.8 for the Rust ⇄ Qt bridge. The QObjects defined in
//! `qobjects/` are auto-registered with QML under
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

mod icon_theme;
mod qobjects;
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

    // Lay down the user-local desktop entry + icon theme BEFORE
    // QGuiApplication boots. xdg-desktop-portal queries the
    // application's registered `.desktop` file as soon as the Qt
    // app contacts the compositor; if we wrote the file after
    // QGuiApplication::new(), the portal would log
    // `Could not register app ID: App info not found for
    // 'com.visorcraft.Grexa'` on the user's very first launch (the
    // file exists by the second launch, so the warning self-heals).
    // Front-loading the write keeps the first launch clean too.
    ensure_user_desktop_integration();

    // Note: an earlier revision tried to force
    // QT_QUICK_CONTROLS_STYLE=Fusion here so Qt's palette would drive
    // input backgrounds (Light theme white-on-white bug). Empirically,
    // on this KDE Plasma 6 host the engine still resolved
    // `qqc2-desktop-style` regardless of the env var, so the dance
    // was dead. The theme story is instead handled in QML by the
    // `App{TextField,ComboBox,CheckBox,SpinBox,FlatButton}` wrappers
    // (apps/grexa-gui/qml/App*.qml), which re-state our token palette
    // at the instance level — that wins over qqc2-desktop-style's
    // `Kirigami.Theme.inherit: false` component default. If a future
    // Qt revision changes the style resolution order we can revisit
    // forcing Fusion from here, but until then it adds no value.

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
            .set_organization_domain(&QString::from("visorcraft.com"));
        // The AppImage does not inherit the host's Breeze theme, so force it
        // explicitly. The theme files are bundled via breeze-icons / qt6-svg.
        icon_theme::set_icon_theme("breeze");
    }

    // Wayland's `app_id` and X11's `WM_CLASS` map to this string.
    // Setting it BEFORE the first window is shown ties the live
    // application to the com.visorcraft.Grexa.desktop file (and its
    // Icon= entry), so the taskbar / dock / alt-tab switcher
    // resolves the pink-gecko icon from the user's hicolor theme.
    // The `.desktop` was already written above (before
    // QGuiApplication::new()), so the portal can resolve this id
    // immediately on first launch.
    cxx_qt_lib::QGuiApplication::set_desktop_file_name(&QString::from("com.visorcraft.Grexa"));

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
            .on_object_creation_failed(move |eng, url| {
                // cxx-qt-lib 0.8 exposes neither `QQmlComponent::errors()` nor a
                // message-handler hook, so we cannot print the individual
                // `QQmlError` lines here. The next-most-useful diagnostic for a
                // *packaged* build — where a failed load almost always means a
                // QML module is missing from the bundle (e.g. Kirigami's
                // `org.kde.desktop` style absent from an AppImage) — is the set
                // of directories the engine searched for modules.
                tracing::error!("QML failed to load: {}", url.to_string());
                tracing::error!(qml_import_paths = %eng.import_path_list(),
                    "engine QML import search paths");
                tracing::error!(
                    "A failed load in a packaged build (AppImage/Flatpak) almost \
                     always means a required QML module is missing from the bundle. \
                     Re-run with QML2_IMPORT_PATH=/usr/lib/qt6/qml — if it then \
                     loads, the bundle is missing that module."
                );
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
            })
            .release();
    }

    if let Some(engine) = engine.as_mut() {
        let url = if cfg!(debug_assertions) {
            let manifest_dir =
                std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
            let fs_path = std::path::PathBuf::from(manifest_dir).join("qml/Main.qml");
            if fs_path.exists() {
                tracing::info!("dev mode: loading QML from filesystem at {}", fs_path.display());
                QUrl::from(&format!("file://{}", fs_path.display()))
            } else {
                QUrl::from("qrc:/qt/qml/com/visorcraft/Grexa/qml/Main.qml")
            }
        } else {
            QUrl::from("qrc:/qt/qml/com/visorcraft/Grexa/qml/Main.qml")
        };
        engine.load(&url);
    }

    if load_failed.load(std::sync::atomic::Ordering::SeqCst) {
        tracing::error!("QML payload did not instantiate — exiting before the event loop.");
        std::process::exit(2);
    }

    // Start background mirror cleanup thread. Prunes container mirrors
    // older than 24 hours every 30 minutes so the cache directory doesn't
    // grow unbounded when the user searches many containers in a session.
    start_mirror_cleanup_thread();

    if let Some(app) = app.as_mut() {
        let code = app.exec();
        if code != 0 {
            tracing::warn!("Qt event loop exited with code {code}");
        }
    }
}

/// Spawn a detached thread that calls `prune_mirrors` on a schedule.
/// The thread exits when the process terminates; no join needed.
fn start_mirror_cleanup_thread() {
    const PRUNE_INTERVAL_SECS: u64 = 30 * 60; // 30 minutes
    const MAX_AGE_SECS: u64 = 24 * 60 * 60; // 24 hours

    std::thread::spawn(move || {
        // First prune at startup (mirrors from a previous run may be stale).
        if let Err(err) = grexa_containers::prune_mirrors(MAX_AGE_SECS) {
            tracing::debug!(error = %err, "initial mirror prune failed");
        }

        loop {
            std::thread::sleep(std::time::Duration::from_secs(PRUNE_INTERVAL_SECS));
            if let Err(err) = grexa_containers::prune_mirrors(MAX_AGE_SECS) {
                tracing::debug!(error = %err, "scheduled mirror prune failed");
            }
        }
    });
}

fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter =
        EnvFilter::try_from_env("GREXA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);

    let paths = AppPaths::from_env();
    let log_path = paths.state_dir.join("grexa-gui.log");

    // Read the privacy toggle from the persisted settings. When
    // `privacy_redact_paths` is true, every line written to the log
    // file has $HOME replaced with `~` so a copy-pasted diagnostic
    // doesn't leak the user's account name. Stderr stays unredacted
    // because that path is the local terminal, not a shared
    // diagnostic surface.
    let redact = grexa_core::SettingsStore::new(&paths)
        .load()
        .map(|s| s.privacy_redact_paths)
        .unwrap_or(false);
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);

    let (file_layer, guard) = match std::fs::create_dir_all(&paths.state_dir) {
        Ok(()) => match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => {
                let writer = RedactingWriter::new(file, redact, home);
                let (writer, guard) = tracing_appender::non_blocking(writer);
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
///   the lock; the caller should attempt DBus activation and exit.
/// * `None` and no env var — the lockfile couldn't be opened (no
///   runtime dir / not writable); continue without single-instance
///   guarantees.
fn acquire_single_instance_lock() -> Option<std::fs::File> {
    use std::os::fd::AsRawFd;
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
        .truncate(true)
        .open(&lock_path)
        .ok()?;
    let fd = file.as_raw_fd();
    let rc = unsafe { libc_flock(fd, LOCK_EX | LOCK_NB) };
    if rc == 0 {
        Some(file)
    } else {
        request_dbus_activate();
        unsafe {
            std::env::set_var("GREXA_INSTANCE_BUSY", "1");
        }
        None
    }
}

/// Ask an already-running Grexa instance to present its window via
/// DBus. Best-effort — if the DBus method call fails, fall back to
/// `wmctrl` to try raising the window, then the second instance exits
/// and the user switches manually.
fn request_dbus_activate() {
    let status = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "com.visorcraft.Grexa",
            "--object-path",
            "/com/visorcraft/Grexa",
            "--method",
            "com.visorcraft.Grexa.Activate",
        ])
        .status();
    match status {
        Ok(s) if s.success() => {}
        _ => {
            let _ = std::process::Command::new("wmctrl")
                .args(["-a", "Grexa"])
                .status();
        }
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

/// Populate `$XDG_DATA_HOME` with our `.desktop` file and the
/// hicolor icon set so the running session can resolve our
/// `app_id` to the pink-gecko icon. The bytes are baked into the
/// binary via `include_bytes!`, so a dev box with no packaged
/// install gets correct branding from `cargo run`.
///
/// Refresh policy: a stamp file at `$XDG_DATA_HOME/grexa/icon-rev`
/// records the version that last wrote into the user theme. On
/// startup we re-extract every file whenever the stamp is missing
/// or doesn't match the running binary's `CARGO_PKG_VERSION` — so
/// an upgraded build replaces stale icons, but unchanged builds
/// skip the work. Files at `/usr/share/...` shipped by packagers
/// always take precedence because XDG resolves them with higher
/// priority than `$XDG_DATA_HOME`.
///
/// After a write we ping `kbuildsycoca6` (Plasma),
/// `update-desktop-database` (generic), and `gtk-update-icon-cache`
/// (GNOME) so the icon shows up without a logout-login cycle. The
/// helpers run detached with stdio routed to `/dev/null` so they
/// don't block startup. All side effects are best-effort —
/// failures fall through silently.
fn ensure_user_desktop_integration() {
    let data_home = match std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| {
                let mut p = std::path::PathBuf::from(h);
                p.push(".local/share");
                p
            })
        }) {
        Some(p) => p,
        None => return,
    };

    let desktop_template = include_bytes!("../../../packaging/com.visorcraft.Grexa.desktop");
    // The packaged template uses `Exec=grexa %f` which is fine for a
    // distro install where `/usr/bin/grexa` is on $PATH. For a dev run
    // out of `target/release/grexa`, `gio` / `GAppInfo` validates the
    // `Exec=` token against $PATH and rejects the file when the binary
    // isn't there — which makes xdg-desktop-portal log
    // `Could not register app ID: App info not found`. Rewriting `Exec=`
    // to the absolute path of the running binary fixes both cases:
    // distro install resolves to `/usr/bin/grexa`, dev install resolves
    // to the cargo target dir. Either way it's a real, on-disk path
    // that GAppInfo can validate.
    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| "grexa".to_string());
    let exec_token = desktop_exec_token(&exec_path);
    let desktop_bytes_owned: Vec<u8> = String::from_utf8_lossy(desktop_template)
        .replace("Exec=grexa %f", &format!("Exec={exec_token} %f"))
        .into_bytes();
    let desktop_bytes = desktop_bytes_owned.as_slice();

    // Refresh stamp includes both the crate version AND the running
    // binary's path — so a dev rebuild that moves the binary (or a
    // distro upgrade) triggers a re-extract of the .desktop with the
    // correct Exec= line.
    let stamp_path = data_home.join("grexa/icon-rev");
    let want_rev = format!("{}|{}", env!("CARGO_PKG_VERSION"), exec_path);
    let have_rev = std::fs::read_to_string(&stamp_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let force_rewrite = have_rev != want_rev;

    let scalable_svg = include_bytes!("../../../packaging/icons/scalable/com.visorcraft.Grexa.svg");
    let icon_16 =
        include_bytes!("../../../packaging/icons/16x16/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_24 =
        include_bytes!("../../../packaging/icons/24x24/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_32 =
        include_bytes!("../../../packaging/icons/32x32/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_48 =
        include_bytes!("../../../packaging/icons/48x48/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_64 =
        include_bytes!("../../../packaging/icons/64x64/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_96 =
        include_bytes!("../../../packaging/icons/96x96/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_128 =
        include_bytes!("../../../packaging/icons/128x128/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_192 =
        include_bytes!("../../../packaging/icons/192x192/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_256 =
        include_bytes!("../../../packaging/icons/256x256/apps/com.visorcraft.Grexa.png").as_slice();
    let icon_512 =
        include_bytes!("../../../packaging/icons/512x512/apps/com.visorcraft.Grexa.png").as_slice();

    let pairs: [(&str, &[u8]); 12] = [
        ("applications/com.visorcraft.Grexa.desktop", desktop_bytes),
        ("icons/hicolor/scalable/apps/com.visorcraft.Grexa.svg", scalable_svg),
        ("icons/hicolor/16x16/apps/com.visorcraft.Grexa.png", icon_16),
        ("icons/hicolor/24x24/apps/com.visorcraft.Grexa.png", icon_24),
        ("icons/hicolor/32x32/apps/com.visorcraft.Grexa.png", icon_32),
        ("icons/hicolor/48x48/apps/com.visorcraft.Grexa.png", icon_48),
        ("icons/hicolor/64x64/apps/com.visorcraft.Grexa.png", icon_64),
        ("icons/hicolor/96x96/apps/com.visorcraft.Grexa.png", icon_96),
        ("icons/hicolor/128x128/apps/com.visorcraft.Grexa.png", icon_128),
        ("icons/hicolor/192x192/apps/com.visorcraft.Grexa.png", icon_192),
        ("icons/hicolor/256x256/apps/com.visorcraft.Grexa.png", icon_256),
        ("icons/hicolor/512x512/apps/com.visorcraft.Grexa.png", icon_512),
    ];

    let mut wrote_anything = false;
    for (rel, bytes) in pairs.iter() {
        let target = data_home.join(rel);
        if target.exists() && !force_rewrite {
            continue;
        }
        if let Some(parent) = target.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            continue;
        }
        if std::fs::write(&target, bytes).is_ok() {
            wrote_anything = true;
        }
    }

    if wrote_anything {
        // Best-effort cache refresh so the icon shows up without
        // a session restart. Detached spawns with stdio routed
        // away — we don't block GUI startup waiting for them.
        let null = || std::process::Stdio::null();
        let _ = std::process::Command::new("kbuildsycoca6")
            .stdin(null())
            .stdout(null())
            .stderr(null())
            .spawn();
        let _ = std::process::Command::new("update-desktop-database")
            .arg(data_home.join("applications"))
            .stdin(null())
            .stdout(null())
            .stderr(null())
            .spawn();
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg("-t")
            .arg(data_home.join("icons/hicolor"))
            .stdin(null())
            .stdout(null())
            .stderr(null())
            .spawn();

        // Stamp the version we just laid down. If this fails we'll
        // just re-extract next launch — cheap and correct.
        if let Some(parent) = stamp_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&stamp_path, &want_rev);
    }
}

fn desktop_exec_token(path: &str) -> String {
    let needs_quotes = path.bytes().any(|b| {
        b.is_ascii_whitespace()
            || matches!(
                b,
                b'"' | b'\''
                    | b'\\'
                    | b'>'
                    | b'<'
                    | b'~'
                    | b'|'
                    | b'&'
                    | b';'
                    | b'$'
                    | b'*'
                    | b'?'
                    | b'#'
                    | b'('
                    | b')'
                    | b'`'
            )
    });
    let mut escaped = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            '%' => escaped.push_str("%%"),
            '"' | '\\' | '`' | '$' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    if needs_quotes {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

/// `std::io::Write` adapter that replaces the user's `$HOME` with
/// `~` before forwarding bytes to the inner writer. Designed for
/// the `tracing-appender` pipeline so log lines that include
/// absolute paths don't leak the user's account when diagnostics
/// get copy-pasted into a bug report.
///
/// Best-effort: works on the byte stream the tracing layer hands
/// down. Partial writes that split `$HOME` across two `write()`
/// calls are still redacted because `tracing-appender` buffers a
/// full event before flushing.
struct RedactingWriter {
    inner: std::fs::File,
    pattern: Option<Vec<u8>>,
}

impl RedactingWriter {
    fn new(inner: std::fs::File, redact: bool, home: Option<std::path::PathBuf>) -> Self {
        let pattern = if redact {
            home.and_then(|h| {
                let s = h.to_string_lossy().into_owned();
                if s.is_empty() {
                    None
                } else {
                    Some(s.into_bytes())
                }
            })
        } else {
            None
        };
        Self { inner, pattern }
    }
}

impl std::io::Write for RedactingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &self.pattern {
            Some(pat) if !pat.is_empty() => {
                // Linear scan + replace. The buffer is small (one
                // tracing event) so allocating a new Vec is cheap.
                let mut out = Vec::with_capacity(buf.len());
                let mut i = 0;
                while i < buf.len() {
                    if buf[i..].starts_with(pat) {
                        out.push(b'~');
                        i += pat.len();
                    } else {
                        out.push(buf[i]);
                        i += 1;
                    }
                }
                self.inner.write_all(&out)?;
                // Return the original len so the caller sees a
                // "consumed all bytes" success — tracing-appender
                // would otherwise loop on a short write.
                Ok(buf.len())
            }
            _ => self.inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::desktop_exec_token;

    #[test]
    fn desktop_exec_token_quotes_paths_with_spaces() {
        assert_eq!(desktop_exec_token("/tmp/Grexa Build/grexa"), "\"/tmp/Grexa Build/grexa\"");
    }

    #[test]
    fn desktop_exec_token_escapes_field_codes_and_quotes() {
        assert_eq!(desktop_exec_token("/tmp/100%/a\"b/grexa"), "\"/tmp/100%%/a\\\"b/grexa\"");
    }

    #[test]
    fn desktop_exec_token_quotes_reserved_desktop_entry_characters() {
        assert_eq!(desktop_exec_token("/tmp/build&test/grexa"), "\"/tmp/build&test/grexa\"");
    }
}
