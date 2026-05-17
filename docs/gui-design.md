# GUI Design + Spike Outcome

## Spike outcome: `qmetaobject` chosen as the Rust ⇄ Qt bridge

PLAN.md Phase 1 calls for a Rust ⇄ Qt bridge spike before committing
to the GUI stack. Two paths were evaluated.

**`cxx-qt` 0.8** — rejected. Two smoke attempts (with and without a
`links = "..."` Cargo manifest field, with and without
`interface.export()` in `build.rs`, with and without explicit
`cxx_qt::init_crate!(cxx_qt_lib)` calls) all failed to link with the
same error:

```
ld.lld: error: undefined symbol: cxx_qt_init_crate_cxx_qt_lib
>>> referenced by public-initializer.cpp:11
>>>   target/debug/build/<crate>/out/cxx-qt-build/initializers/<crate>/public-initializer.cpp:11
>>> referenced by main.rs:N (init_crate! call site)
```

The symbol is emitted by an auto-generated `public-initializer.cpp`
that the cxx-qt-build script writes into `OUT_DIR`. It cannot be
suppressed by Rust source changes. The cxx-qt-lib build script
`.export()`s the corresponding C++ initializer but downstream
binaries don't pick it up under pure Cargo — the CMake harness in
the cxx-qt repo's own examples threads the export through. We
re-evaluate when cxx-qt ships a pure-Cargo flow.

**`qmetaobject` 0.2** — accepted. Pure Rust, no build script, no
CMake. `cargo build -p grexa` produces a working binary that
registers QObjects under `com.visorcraft.Grexa 1.0`, boots a
`QmlEngine`, and runs the full Qt event loop. Verified locally with
`QT_QPA_PLATFORM=offscreen target/release/grexa` exiting 0 after the
QML loads. The QObject surface is in `apps/grexa-gui/src/qobjects.rs`;
the workspace controllers (`workspace.rs`, `tab.rs`, `status.rs`)
remain the source of truth for business logic.

What ships today:

1. **A working Qt 6 binary.** `cargo run -p grexa` registers
   `SearchController` with Qt's metaobject system and launches the
   Kirigami QML shell.
2. **A SearchController QObject** with `status_text`, `match_count`,
   `busy`, and `recent_path_count` properties, `status_changed` and
   `history_changed` signals, and `start_search` / `cancel` /
   `recent_paths_json` slots. The real `grexa-core` search engine
   drives it; the recent-paths store records every path; the
   workspace state is shared via a thread-local pointer so QML
   instances see the same state.
3. **A complete QML page set** at `apps/grexa-gui/qml/` — Main +
   Search + Regex Builder + Settings + About + Context Preview +
   AiChatPanel + DesignTokens.
4. **Unit tests** that exercise the QObject end-to-end against a real
   tempdir (`search_controller_drives_real_search` in `qobjects.rs`).

## Module map

```
apps/grexa-gui/
├── Cargo.toml          # depends on every other Grexa crate + qmetaobject
├── src/
│   ├── main.rs         # logging + workspace install + QmlEngine boot
│   ├── qobjects.rs     # SearchController QObject (qmetaobject) + workspace TLS
│   ├── controller.rs   # `Controllers` struct: settings, bundle, cancel
│   ├── tab.rs          # `TabState` per-tab state
│   ├── workspace.rs    # `Workspace`: tabs + persistent stores + replace
│   └── status.rs       # `format_status` Fluent-aware status formatter
└── qml/
    ├── Main.qml                # Kirigami ApplicationWindow + nav rail
    ├── SearchPage.qml          # path + term + filters + result list
    ├── RegexBuilderPage.qml    # presets + sample + live matches
    ├── SettingsPage.qml        # every settings section
    ├── ContextPreviewDialog.qml# gutter + match-line highlight
    ├── AiChatPanel.qml         # disabled / empty / busy / error states
    ├── AboutPage.qml           # version + license + credits
    └── DesignTokens.qml        # spacing / radius / colors / typography
```

## Wiring contracts

Every QML page binds to one or more controller objects in
`controller.rs`. The contracts are stable; the QML side can evolve
without touching Rust as long as it uses the same keys.

### Search

- Inputs (Rust → QML):
  - `i18n.search-status-ready` / `…running` / `…cancelled` / `…error`
    via `Bundle::format`
  - Streaming `ProgressEvent` (`FileScanned`, `FileSkipped`, `Match`)
    coalesced into 256-row batches before delivery
  - `SearchSummary` at completion
- Outputs (QML → Rust):
  - `SearchOptions` constructed from the form fields
  - `CancelToken::cancel()` on Stop button
  - `ReplaceOptions` on the replace flow

### Regex Builder

- Inputs: live `PatternEngine` results for sample text
- Outputs: pattern string to apply to the active Search tab

### Settings

- Inputs: `DefaultSettings` loaded from `SettingsStore`
- Outputs: each toggle triggers `SettingsStore::save` (instant save)
- Special: AI API key set / clear via `grexa_ai::store_api_key`,
  `delete_api_key`; the value never round-trips through QML

### About

- Inputs: version (compile-time `env!("CARGO_PKG_VERSION")`), commit
  sha (compile-time via `vergen` in a future change), license text

## Cross-tab state

Each Search tab is its own state object. The controller layer holds a
`Vec<TabState>` plus an `active: usize`. Tab creation / close /
rename / drag is GUI-only; the Rust side only needs:

- `new_tab() -> TabId`
- `close_tab(id)`
- `set_active(id)`
- `get_tab(id) -> &TabState`

## Build pipeline (current)

Pure Cargo. `cargo build -p grexa` produces a self-contained binary
linked against `libQt6Core.so` / `libQt6Qml.so` from the system Qt
runtime. No CMake, no build script in `apps/grexa-gui`.

If a future PR wants cxx-qt's compile-time generated bindings (sharper
typing, automatic property notification, QML enum exposure), it should:

1. Add `cxx-qt` + `cxx-qt-build` + `cxx-qt-lib` to the workspace.
2. Introduce a `CMakeLists.txt` under `apps/grexa-gui` and update the
   CI host bootstrap to install CMake + Qt6 dev packages.
3. Migrate `qobjects.rs` from `qmetaobject` to `cxx_qt::bridge`.

The qmetaobject implementation is the production path until that
migration. QML files load via `QmlEngine::load_file`; an installed
binary picks them up from `/usr/share/grexa/qml/`.

## What lands when the dedicated GUI PR opens

Phase 1 (the spike itself):
- Add `cxx-qt-build`, `cxx-qt`, `cxx-qt-lib` to apps/grexa-gui
- Drop the `qml6`-spawn fallback
- Add a `CMakeLists.txt` and document the dual-build (Cargo + CMake)
- Stand up one Rust `QObject` (the `SearchController`) and one QML
  binding test

Phase 4 (Search UI MVP):
- Replace `SearchPage.qml` placeholder with the real path/term/mode
  pickers, filter pane, command strip, virtualized result tables
- Wire `ProgressEvent` → `ListModel` batch inserts
- Add Tabs, Stop button, search-within-results

Phase 5 (Linux desktop integration):
- Portal file picker (`org.freedesktop.portal.FileChooser`)
- KIO-FUSE path support
- KNotifications via `KNotifications::sendEvent`
- KDE color-scheme tracking

Phase 9 (Regex Builder), Phase 10 (Settings), Phase 14 (Context
Preview), Phase 18 (Visual Polish): all incremental QML work on top
of the same controller scaffolding.

## Why ship this skeleton at all?

Three reasons:

1. **Verifies end-to-end wiring.** The Rust host has to link every
   core crate. Today that passes `cargo build -p grexa` and
   `cargo test -p grexa`. A regression in any consumed crate breaks
   the workspace clippy gate.
2. **Documents the contract.** Every page that doesn't exist yet
   still has a QML file describing what it expects. Future engineers
   don't have to re-derive the layout from PLAN.md.
3. **Gives the user a "GUI launches" smoke test.** `cargo run -p grexa`
   on a Qt 6 + Kirigami box pops a window with four pages, navigation
   rail, and KDE-native styling — proof the Qt path works even before
   the full UI lands.
