# GUI Design + cxx-qt Spike Outcome

## Spike outcome: deferred to a dedicated PR

PLAN.md Phase 1 calls for a `cxx-qt` spike before committing to the
Rust ⇄ Qt bridge. The spike was attempted in this session and the
findings are recorded here.

**Recommendation: build the GUI in a dedicated PR series with CMake
installed.** The current Cargo-only repo can't host a clean cxx-qt
build because `cxx-qt-build` 0.7 still requires a CMake configure /
build pipeline to generate the C++ shims that pair with the Rust
QObjects. Layering CMake on top of the workspace is justified for the
GUI work but is too disruptive to land alongside the non-GUI items.

What this session does instead:

1. Ships a structured Rust GUI host at `apps/grexa-gui/src/main.rs`
   that links every core crate so we know the workspace compiles
   end-to-end against the GUI.
2. Ships a Kirigami QML skeleton under `apps/grexa-gui/qml/` that
   declares every page the full UI needs. Each placeholder page calls
   out what data Rust feeds it.
3. Wires a runtime spawn of `qml6` against the bundled QML so a user
   can launch `cargo run -p grexa` and see the navigation rail + four
   placeholder pages.
4. Records the wiring decisions here so the next engineer can pick
   them up without re-deciding.

## Module map

```
apps/grexa-gui/
├── Cargo.toml          # depends on every other Grexa crate
├── src/
│   ├── main.rs         # logging + controller bootstrap + qml6 spawn
│   └── controller.rs   # `Controllers` struct: settings, bundle,
│                       # cancel token, AI client, command runner
└── qml/
    ├── Main.qml             # Kirigami ApplicationWindow + nav rail
    ├── SearchPage.qml       # Phase 4 destination
    ├── RegexBuilderPage.qml # Phase 9 destination
    ├── SettingsPage.qml     # Phase 10 destination
    └── AboutPage.qml        # Phase 4 destination
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

## Build pipeline (planned)

Once CMake is added to the host environment:

1. `cargo build` runs the workspace; `cargo build -p grexa` also
   triggers `cxx-qt-build` which generates C++ glue + a small `.qrc`.
2. `cmake` is invoked from the `cxx-qt-build` build script with a
   minimal `CMakeLists.txt` under `apps/grexa-gui/`.
3. `cmake` produces a shared object that `apps/grexa-gui/target` links
   against; `cargo` ties the final binary together.
4. QML files are bundled via Qt's resource system (`qrc`) so the
   release binary is self-contained.

The current `qml6`-spawn approach is good enough for a placeholder and
keeps the build pure-Cargo.

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
