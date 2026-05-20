# GUI Design + Spike Outcome

## Spike outcome: `cxx-qt` 0.8 is the production Rust ⇄ Qt bridge

PLAN.md Phase 1 called for a Rust ⇄ Qt bridge spike before
committing to the GUI stack. Two paths were evaluated.

**`cxx-qt` 0.8** — accepted, pure-Cargo. The crate generates the
QObject's C++ side at build time, registers `#[qml_element]`
QObjects under a `QmlModule` URI specified in `build.rs`, and
bundles `.qml` files into the binary via Qt's resource system.
`cargo build -p grexa --release` produces a working binary that
boots `QGuiApplication`, builds a `QQmlApplicationEngine`, and
loads `qrc:/qt/qml/com/visorcraft/Grexa/Main.qml`. The QObject
surface is split across `apps/grexa-gui/src/qobjects/`; each
QObject family owns its own `#[cxx_qt::bridge]` module.
`apps/grexa-gui/build.rs` calls `CxxQtBuilder::new_qml_module(...)`.
Verified locally with `QT_QPA_PLATFORM=offscreen target/release/grexa`
running the Qt event loop. Shared persistent GUI state lives in
`workspace.rs`; per-tab UI state lives in `SearchPage.qml` snapshots
and `SearchController`'s Rust-side snapshot map.

A previous spike rejected cxx-qt after the link step failed with
`undefined symbol: cxx_qt_init_crate_cxx_qt_lib`. That failure
turned out to be cache state from a partial attempt — a fresh
checkout with `CxxQtBuilder::new_qml_module(...)` (the API used
when the crate IS a QML module) plus `cxx_qt::init_crate!` calls in
`main.rs` links cleanly on cxx-qt 0.8.1 against system Qt 6.11.
`qmetaobject` 0.2 was the production bridge until this PR landed
and is no longer pulled in by `apps/grexa-gui`.

What ships today:

1. **A working Qt 6 binary.** `cargo run -p grexa` registers
   `SearchController` with Qt's metaobject system via cxx-qt and
   launches the Kirigami QML shell.
2. **A SearchController QObject** declared as `#[qml_element]` with
   `status_text` (QString), `match_count` (i32), `busy` (bool), and
   `recent_path_count` (i32) qproperties (each with auto-generated
   change signals), a `history_changed` qsignal, and
   `start_search` / `cancel` / `recent_paths_json` qinvokables. The
   real `grexa-core` search engine drives it; the recent-paths
   store records every path; shared stores are accessed through a
   thread-local `Workspace` so QML instances see the same state.
3. **A complete QML page set** at `apps/grexa-gui/qml/` — Main +
   Search + Regex Builder + Settings + About + Credits + Licenses +
   Context Preview + AiChatPanel + DesignTokens — bundled into the binary
   via Qt's resource system at `qrc:/qt/qml/com/visorcraft/Grexa/...`.
4. **Unit tests** that exercise the Rust-side QObject backing state
   without instantiating Qt, including search streaming, recent-path JSON,
   Regex Builder evaluation, Settings reload, license bundling, and credits
   parsing under `src/qobjects/`. The cxx-qt-generated QObjects are tested by
   `cargo build -p grexa` itself: a regression in property signatures,
   qinvokable types, or qsignal generation trips the C++ compile in
   `build.rs`.

## Module map

```
apps/grexa-gui/
├── Cargo.toml          # depends on every other Grexa crate + cxx-qt + cxx-qt-lib
├── build.rs            # CxxQtBuilder::new_qml_module(...) — registers QML module + files
├── src/
│   ├── main.rs         # logging + workspace install + QGuiApplication + QQmlApplicationEngine
│   ├── qobjects/       # cxx-qt QObjects + workspace TLS handle
│   └── workspace.rs    # `Workspace`: persistent stores + Fluent bundle
└── qml/                # bundled into binary at qrc:/qt/qml/com/visorcraft/Grexa/
    ├── Main.qml                # Kirigami ApplicationWindow + nav rail + shortcuts
    ├── SearchPage.qml          # path + term + filters + tabs + result list
    ├── SearchBar.qml           # path picker + term field + flag chips + Search button
    ├── FlagChip.qml            # toggle chip (regex / case-sensitive); parent owns state
    ├── ResultRow.qml           # one match row + right-click context menu
    ├── HistoryPage.qml         # completed-search list with debounced filter
    ├── ProfilesPage.qml        # saved-search presets with debounced filter
    ├── RegexBuilderPage.qml    # presets + sample + live matches
    ├── SettingsPage.qml        # auto-save sections + Saved / Save-failed pill
    ├── ContextPreviewDialog.qml# gutter + match-line highlight
    ├── AiChatPanel.qml         # disabled / empty / busy / error states + Clear
    ├── AboutPage.qml           # version + license + credits
    ├── CreditsPage.qml         # card + table summary of third-party credits
    ├── LicensesPage.qml        # tabbed bundled license document viewer
    ├── GplLicenseDialog.qml    # bundled GPL-3.0 text viewer
    ├── NavItem.qml             # sidebar nav entry
    ├── Card.qml                # rounded surface used by SettingsPage sections
    ├── EmptyState.qml          # shared empty-state illustration + copy
    ├── PrimaryButton.qml       # filled-accent primary action button
    ├── AppTextField.qml        # themed TextField (re-states palette to dodge qqc2-desktop-style's inherit:false)
    ├── AppComboBox.qml         # themed ComboBox (same pattern)
    ├── AppCheckBox.qml         # themed CheckBox (replaces indicator delegate too)
    ├── AppSpinBox.qml          # themed SpinBox (same pattern)
    ├── AppFlatButton.qml       # themed flat Button — sets `flat: true` + Button colorSet overrides + icon.color
    └── DesignTokens.qml        # spacing / radius / colors / typography / a11y
```

### Theming model

User palettes (`Settings → Appearance`) map to a token table in
`DesignTokens.qml` that mirrors upstream Grex's `MainWindow.xaml.cs`
color stops. Tokens flow through three layers:

1. **`Kirigami.Theme`** overrides on `ApplicationWindow` + each
   `Page` cascade theme colors to children that respect Kirigami's
   attached property.
2. **Qt `palette.*`** overrides on the same items reach
   QtQuick.Controls that read Qt's QPalette (Fusion-style backends
   and any control that uses `palette.base` etc.).
3. **`App*.qml` wrappers** re-state both layers at the *instance*
   level on TextField / ComboBox / CheckBox / SpinBox / flat Button.
   This is necessary because `qqc2-desktop-style` (the default
   QtQuick.Controls style on Plasma) hardcodes
   `Kirigami.Theme.inherit: false` on each of those control types,
   blocking parent-level overrides. Instance-level attached
   properties win over component defaults, so the wrappers force the
   inheritance back on and re-state the colors from our tokens. New
   forms should use the wrappers, not the raw `Controls.X`.

## Wiring contracts

Every QML page binds to one or more controller objects in
`src/qobjects/`. The contracts are stable; the QML side can evolve
without touching Rust as long as it uses the same invokables,
properties, and model roles.

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
  sha (compile-time via `vergen` in a future change), bundled license
  text
- Outputs: "Licenses" navigates to the full bundled document
  viewer; "Credits" navigates to the card-and-table third-party
  credits page.

### Credits

- Inputs: `SettingsController::third_party_credits_json()` parses the
  bundled `docs/credits-third-party.md` supplement into table-ready
  crate rows; runtime components are listed from `CREDITS.md`.
- Outputs: runtime dependency credits and a filterable Cargo crate
  table with project links.

### Licenses

- Inputs: `SettingsController` exposes `include_str!` bundles for
  `docs/credits-third-party.md`, `CREDITS.md`, and `LICENSE`.
- Outputs: tabbed in-app document viewer for third-party license
  texts, acknowledgments, and Grexa's GPL v3 text, with line filtering
  and copy support. The GPL dialog uses the same Rust-bundled license
  text rather than QML resource reads.

## Cross-tab state

Search tabs are in-session QML state. `SearchPage.qml` owns the tab
strip, active tab id, and form fields, while `SearchController` stores
per-tab result snapshots keyed by the stable tab id. Switching tabs
saves the outgoing snapshot with `save_tab_snapshot(id)` and restores
the incoming snapshot with `restore_tab_snapshot(id)`. Persistent
history, profiles, settings, and recent paths still flow through the
shared `Workspace`.

## Build pipeline (current)

Pure Cargo. `cargo build -p grexa` produces a self-contained binary
that runs cxx-qt's compile-time code generator (driven by
`apps/grexa-gui/build.rs`) to emit C++ for each `#[cxx_qt::bridge]`,
compiles it with `cc`, links against `libQt6Core.so` /
`libQt6Qml.so` / `libQt6Gui.so` from the system Qt runtime, and
embeds every `.qml` file under `apps/grexa-gui/qml/` into the
binary via Qt's resource system. No CMake, no host bootstrap
changes beyond the Qt 6 dev packages already required by the prior
qmetaobject build.

QML files load from `qrc:/qt/qml/com/visorcraft/Grexa/...` at
runtime. Editing a QML file requires a `cargo build` cycle because
the file is baked into the binary at build time — that is the
cxx-qt-native flow.
