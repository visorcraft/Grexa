# AGENTS.md

Guidelines for AI assistants working on this repository.

Grexa is a Linux/Qt 6 port of [Grex](https://github.com/visorcraft/grex)
(the upstream Windows/WinUI tool). It is a Rust workspace plus a Qt 6 /
Kirigami GUI shell built with [cxx-qt](https://github.com/KDAB/cxx-qt) —
pure Cargo, no CMake.

## Build commands

Everything goes through `just` (the wrapper around `cargo`). Direct
`cargo` invocations are equivalent if `just` isn't installed.

```bash
just ci             # fmt check + clippy + tests — the same gate CI runs
just build          # cargo build --workspace (debug)
just build-release  # cargo build --workspace --release
just test           # cargo test --workspace
just lint           # cargo clippy --workspace --all-targets -- -D warnings
just fmt            # cargo fmt --all
just deny           # cargo deny --all-features check
just audit          # cargo audit
just manpage        # writes target/man/grexa-cli.1
just completions    # writes target/completions/grexa-cli.{bash,zsh,fish}
just run-cli ARGS   # cargo run -p grexa-cli -- ARGS
just run-gui        # cargo run -p grexa
just package        # cargo package --workspace --allow-dirty (release dry-run)
```

**CI parity:** `just ci` runs `fmt --check`, `clippy -D warnings`, and
`cargo test --workspace` in that order. A red local `just ci` is a red
CI run.

**Toolchain pinning:** `rust-toolchain.toml` pins Rust 1.95 (stable,
Rust 2024 edition). Do not bump without updating
`docs/build-and-test.md`'s prerequisites table.

**Qt is required for the GUI crate only.** `cargo build -p grexa-cli`
or `cargo test -p grexa-core` work on a box with no Qt installed.
`cargo build -p grexa` needs `qt6-base-dev` + `qt6-declarative-dev` +
`qt6-tools-dev` + `clang` (cxx-qt drives `cc` to compile generated C++).

## Workspace layout

```
grexa/
├── apps/grexa-gui/             # Qt 6 / Kirigami GUI binary (crate: grexa)
│   ├── Cargo.toml              # depends on every other Grexa crate + cxx-qt
│   ├── build.rs                # CxxQtBuilder::new_qml_module(...) — bundles QML into binary
│   ├── src/
│   │   ├── main.rs             # QGuiApplication + QQmlApplicationEngine boot
│   │   ├── qobjects.rs         # cxx_qt::bridge — SearchController QObject
│   │   ├── controller.rs       # singletons the GUI relies on
│   │   ├── workspace.rs        # multi-tab workspace state
│   │   ├── tab.rs              # per-tab state
│   │   └── status.rs           # Fluent-aware status formatter
│   └── qml/                    # bundled at build time → qrc:/qt/qml/com/visorcraft/Grexa/
├── crates/
│   ├── grexa-core/             # search, replace, encoding, gitignore, glob,
│   │                           # context preview, sorting, settings, history,
│   │                           # profiles, document extraction, desktop helpers
│   ├── grexa-cli/              # grexa-cli binary — all search/replace flags
│   ├── grexa-containers/       # Docker + Podman runtime adapters
│   ├── grexa-ai/               # OpenAI-compatible HTTP client + keyring storage
│   └── grexa-i18n/             # Fluent-backed localization (en / de / ja)
├── docs/                       # behavior contracts + Grex audits + release notes
├── packaging/                  # Flatpak / AppImage / distro recipes + desktop file
├── scripts/                    # bench, locale sync, post-package smoke
├── CLAUDE.md                   # shim → AGENTS.md
├── AGENTS.md                   # this file
├── CREDITS.md                  # third-party attribution
├── LICENSE                     # GPL-3.0 verbatim
├── README.md                   # user-facing entry point
├── deny.toml                   # cargo-deny license + advisory policy
├── rust-toolchain.toml         # pins Rust 1.95
├── rustfmt.toml                # workspace fmt config
├── Justfile                    # task runner
└── Cargo.toml                  # workspace manifest
```

## Architecture

### Crate boundaries

- **`grexa-core`** is the engine. Everything algorithmic — pattern
  matching (`regex` + `fancy-regex` cascade), filesystem walking
  (`ignore` crate with `WalkBuilder::require_git(false)`), encoding
  detection (`chardetng` + `encoding_rs`), atomic replace
  (`tempfile::NamedTempFile::persist`), document extraction
  (OOXML/ODF/PDF), and the on-disk stores (`settings.json`,
  `recent_paths.json`, `search_history.json`, `profiles.json`).
  Pure Rust, no async runtime, no UI dependencies.
- **`grexa-cli`** is the headless face. Builds on `grexa-core` and
  `clap`; emits text / JSON / CSV. Owns the man-page and shell-
  completion generators.
- **`grexa-containers`** is the Docker / Podman adapter. Shells out
  to `docker`/`podman` via a `CommandRunner` trait (mocked in tests
  with `MockCommandRunner`; production uses `SystemCommandRunner`).
- **`grexa-ai`** is the optional OpenAI-compatible HTTP client. Uses
  `ureq` (sync HTTP) + `rustls`. API keys live in the system secret
  service via the `keyring` crate — they never round-trip through
  QML.
- **`grexa-i18n`** is the Fluent bundle. Catalogs are at
  `crates/grexa-i18n/locales/<lang>/grexa.ftl`. English is the source
  of truth; non-English locales must contain every English key.
- **`grexa`** (in `apps/grexa-gui`) is the Qt binary. It links every
  other crate but contains no algorithm logic of its own. The QML
  layer talks to one cxx-qt-generated `SearchController` QObject;
  business state lives in `Workspace` (TLS-installed before the
  engine boots).

### cxx-qt patterns

- **One bridge module per QObject family.** Today there is one:
  `qobject` in `apps/grexa-gui/src/qobjects.rs`. New QObjects join
  the same module unless they're a separate logical surface.
- **Bridge declaration vs. impl:** `#[cxx_qt::bridge] pub mod qobject
  { extern "RustQt" { #[qobject] type SearchController = super::SearchControllerRust; … } }`
  declares the QObject. The methods are implemented in
  `impl qobject::SearchController { … }` against `Pin<&mut Self>`.
  Property setters (`set_status_text`, etc.) are auto-generated from
  `#[qproperty(...)]` and emit change signals automatically.
- **State backing struct:** Each QObject owns a `*Rust` struct
  declared in the outer module. Test the Rust logic against this
  struct directly — Qt isn't needed for unit tests.
- **QML registration is automatic.** The build script's
  `QmlModule::new("com.visorcraft.Grexa").version(1, 0).qml_files([...])`
  is the only registration. New `.qml` files MUST be added to the
  list in `apps/grexa-gui/build.rs` or they won't ship.
- **Editing QML requires `cargo build`.** Files are bundled into the
  binary as Qt resources at build time. There is no filesystem
  hot-reload path.
- **Init order:** in `main.rs`, call `cxx_qt::init_crate!(cxx_qt_lib)`
  + `cxx_qt::init_crate!(grexa)` + `cxx_qt::init_qml_module!("com.visorcraft.Grexa")`
  BEFORE constructing `QGuiApplication`. Setting application
  name/version/organization via `set_*` happens after.

## XDG paths

`AppPaths::from_env()` honors the freedesktop spec; tests use
`AppPaths::under(tempdir)` so the user's real config is never
touched.

| Path | Default location | Purpose |
| ---- | --------------- | ------- |
| `paths.config_dir` | `$XDG_CONFIG_HOME/grexa` (`~/.config/grexa`) | `settings.json` |
| `paths.data_dir`   | `$XDG_DATA_HOME/grexa` (`~/.local/share/grexa`) | `recent_paths.json`, `search_history.json`, `profiles.json` |
| `paths.cache_dir`  | `$XDG_CACHE_HOME/grexa` (`~/.cache/grexa`) | (reserved) |
| `paths.state_dir`  | `$XDG_STATE_HOME/grexa` (`~/.local/state/grexa`) | `grexa-gui.log`, `replace-journal.json` |

API keys live in the Secret Service (KWallet / GNOME Keyring) under
service `io.visorcraft.Grexa`, never on disk.

## Tests

```bash
cargo test --workspace                                  # 298 tests (v0.3)
cargo test -p grexa-core --test gitignore_parity        # 61 cases
cargo test -p grexa-core --test property                # 4 proptests
cargo test -p grexa-core --test root_safety             # 3 pseudo-FS
cargo test -p grexa-containers --features container-live -- live::   # 4 live-daemon
```

Default suite uses `MockCommandRunner` for container tests — no
daemon required. Container-live tests need `podman` or `docker` on
`$PATH` and self-skip if neither is reachable. The full grep-style
parity matrix against Grex's gitignore behavior lives in
`crates/grexa-core/tests/gitignore_parity.rs` with cases that have
inline `DIVERGES from Grex` comments where applicable.

GUI tests in `apps/grexa-gui/src/qobjects.rs` exercise the Rust-side
`SearchControllerRust` struct directly — no Qt runtime required.

## Localization

UI strings flow through Fluent (`.ftl` catalogs). The English
catalog at `crates/grexa-i18n/locales/en/grexa.ftl` is the source of
truth.

To add a new translation key:

1. Add it to `crates/grexa-i18n/locales/en/grexa.ftl` with the
   English text. Use plural variants where the message changes by
   count: `key = { $count -> [one] one match *[other] {$count} matches }`.
2. Add the same key to every other locale (de, ja) with a placeholder
   if the translation isn't ready — `python3 scripts/check_locale_sync.py`
   enforces parity in CI.
3. Use `Bundle::format("key-name", &args)` from Rust;
   `i18n("key-name")` from QML.

The locale-sync check is also a `cargo test`
(`every_locale_has_same_key_set_as_english`) so `just ci` catches
drift.

## Code conventions

- **Edition 2024, Rust 1.95+.** Do not require nightly.
- **No async runtime.** `grexa-core` is sync-only; `grexa-ai` uses
  `ureq` (sync HTTP). The GUI is sync today; an eventual worker
  thread will use `cxx_qt::Threading::queue` to hop signals back.
- **SPDX REUSE headers required.** Every new source file gets:
  ```
  // SPDX-FileCopyrightText: 2026 VisorCraft LLC
  // SPDX-License-Identifier: GPL-3.0-only
  ```
  (or `#` form for Python/shell, `//` for `.qml`). No verbose
  GPL paragraph; the SPDX short form is what we use repo-wide.
- **GPL-3.0-only.** New dependencies must use a license in
  `deny.toml`'s allowlist. The default-allow set is MIT, Apache-2.0,
  Apache-2.0 WITH LLVM-exception, BSD-3-Clause, CDLA-Permissive-2.0,
  GPL-3.0-only, ISC, LGPL-2.1-or-later, Unicode-3.0, Unlicense,
  Zlib, 0BSD. Anything outside that requires a `licenses.clarify`
  entry justified in review.
- **No qmetaobject.** The qmetaobject crate has been removed from the
  workspace and must not return. cxx-qt is the only Rust ⇄ Qt
  bridge.
- **No CMake on the host.** The build is pure Cargo. cxx-qt-build's
  internal cc-driven pipeline replaces what would otherwise be
  CMake-driven.
- **Don't bypass the grex audit docs.** When changing behavior that's
  documented in `docs/grex-*-audit.md`, update the audit doc in the
  same change. The audits pin Grex's behavior; if Grexa intentionally
  diverges, the divergence is recorded in
  `docs/linux-decisions.md`.

### Style nits

- Follow existing patterns in adjacent files.
- Default to no comments. Only add one when the WHY is non-obvious.
- Don't explain WHAT the code does; well-named identifiers do that.
- `tracing::*!` for structured logs. Default filter is `info`;
  override via the `GREXA_LOG` env var.
- All user-facing strings route through `grexa-i18n::Bundle` (Rust)
  or `i18n(...)` (QML). Hard-coded English literals are temporary
  scaffolds at most.

## Adding a new search flag (end-to-end)

1. Add the field to `crates/grexa-core/src/models.rs` (`SearchOptions`
   or the relevant sibling struct). Tests in the same file pin the
   default.
2. Implement the behavior in `crates/grexa-core/src/search.rs` (or
   the appropriate module). Add unit tests below the function.
3. Wire the CLI flag in `crates/grexa-cli/src/main.rs` (clap derive).
   The flag name should match Grex's CLI for parity unless the audit
   doc says otherwise. Add a CLI integration test in
   `crates/grexa-cli/tests/cli.rs`.
4. If the field affects how a `RecentSearch` is keyed, update
   `RecentSearch::key()` in `crates/grexa-core/src/storage.rs` and
   the seven-field dedup test (`recent_search_key_round_trips`).
5. Update the GUI: add a binding in `apps/grexa-gui/qml/SearchPage.qml`
   and route it through the `SearchController` invokable in
   `apps/grexa-gui/src/qobjects.rs`.
6. Add a Fluent key for any new labels in
   `crates/grexa-i18n/locales/en/grexa.ftl` and sync to de/ja.
7. Update `docs/usage.md` if the flag is user-facing and
   `docs/reference.md` for the settings/CLI reference tables.
8. `just ci` until green.

## Adding a new locale

1. Create `crates/grexa-i18n/locales/<lang>/grexa.ftl`.
2. Copy the English file's structure; translate keys.
3. `python3 scripts/check_locale_sync.py` to confirm parity.
4. List the locale in `crates/grexa-i18n/src/lib.rs` (the `Locale`
   enum) and in `README.md`'s status line.
5. Update `docs/translations.md`'s "currently shipped" list.

## Common pitfalls

- **Forgetting to add a new `.qml` file to `build.rs`.** It won't be
  bundled; QML import fails at runtime with `module not installed`.
- **Editing `Cargo.lock` by hand.** Let Cargo own it; never
  hand-edit. The lockfile IS checked in for reproducible builds.
- **Hand-formatting after `cargo fmt`.** Recent rustfmt versions
  changed some defaults; if a workspace-wide `cargo fmt` rewrites
  unrelated files, that's the format catching up — commit it
  separately or accept the noise.
- **Adding a feature flag for a hypothetical future use.** Don't.
  Add it when it's needed.
- **Mocking the database/filesystem in tests.** `grexa-core` tests
  use `tempfile::TempDir` against the real filesystem. Mocking is
  reserved for the `CommandRunner` boundary in `grexa-containers`
  where the dependency is a subprocess.

## Reporting

- Bugs / feature requests: GitHub issues at
  <https://github.com/visorcraft/grexa/issues>.
- Security: see `docs/security.md` for the disclosure policy.
- Build/test fails on your distro: file with full output of
  `just ci 2>&1` and `cargo --version`.

## Reference docs

| Doc | Purpose |
| --- | ------- |
| [docs/release-notes-0.3.0.md](docs/release-notes-0.3.0.md) | v0.3.0 release notes (polish + responsiveness) |
| [docs/release-notes-0.2.0.md](docs/release-notes-0.2.0.md) | v0.2.0 release notes (Phase 20 GUI parity) |
| [docs/release-notes-0.1.0.md](docs/release-notes-0.1.0.md) | v0.1.0 release notes |
| [docs/features.md](docs/features.md) | End-to-end feature list |
| [docs/usage.md](docs/usage.md) | User workflows (KDE, Docker, Podman, AI, CLI) |
| [docs/architecture.md](docs/architecture.md) | Module map + data paths |
| [docs/build-and-test.md](docs/build-and-test.md) | Distro prerequisites + dev loop |
| [docs/reference.md](docs/reference.md) | Settings schema + CLI reference |
| [docs/translations.md](docs/translations.md) | Localization pipeline for translators |
| [docs/security.md](docs/security.md) | Threat model + secret storage |
| [docs/linux-decisions.md](docs/linux-decisions.md) | Intentional divergences from Grex |
| [docs/migration-from-grex.md](docs/migration-from-grex.md) | Bringing Grex settings into Grexa |
| [docs/gui-design.md](docs/gui-design.md) | cxx-qt bridge + QML module map |
| [docs/feature-parity.md](docs/feature-parity.md) | Grex ↔ Grexa parity matrix |
| [docs/grex-*-audit.md](docs/) | Per-component behavior pin from upstream Grex |
| [CREDITS.md](CREDITS.md) | Third-party attribution |
| [LICENSE](LICENSE) | GPL-3.0 full text |
