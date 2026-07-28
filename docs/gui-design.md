# GUI Design

Grexa's desktop application is Qt 6 and Kirigami QML backed by Rust QObjects
generated through the workspace-pinned cxx-qt stack. The build is pure Cargo;
there is no host CMake project and no second GUI implementation.

## Runtime boot

[`apps/grexa-gui/src/main.rs`](../apps/grexa-gui/src/main.rs) performs this
order:

1. initialize tracing;
2. acquire the best-effort single-instance lock;
3. initialize `cxx_qt_lib` and the Grexa cxx-qt crate;
4. register the `com.visorcraft.Grexa 1.0` QML module;
5. configure Qt style and bundled icon lookup;
6. construct the process-wide `Workspace`;
7. construct `QGuiApplication`;
8. set application metadata and desktop file name;
9. construct `QQmlApplicationEngine`;
10. load
    `qrc:/qt/qml/com/visorcraft/Grexa/qml/Main.qml`;
11. enter the Qt event loop.

The initialization calls must happen before `QGuiApplication` construction.
The `Workspace` must be installed before QML creates controller objects.

## Source layout

```text
apps/grexa-gui/
├── Cargo.toml
├── build.rs
├── resources/
│   ├── empty-search.svg
│   ├── grex-mark.png
│   └── grexa.png
├── src/
│   ├── icon_theme.cpp
│   ├── icon_theme.rs
│   ├── main.rs
│   ├── workspace.rs
│   └── qobjects/
│       ├── ai.rs
│       ├── db.rs
│       ├── regex_builder.rs
│       ├── search.rs
│       ├── settings.rs
│       └── workspace_handle.rs
└── qml/
    ├── Main.qml
    ├── SearchPage.qml
    ├── SearchBar.qml
    ├── ResultRow.qml
    ├── HistoryPage.qml
    ├── ProfilesPage.qml
    ├── RegexBuilderPage.qml
    ├── SettingsPage.qml
    ├── DatabasePage.qml
    ├── RecordCard.qml
    ├── ContextPreviewDialog.qml
    ├── AiChatPanel.qml
    ├── AboutPage.qml
    ├── CreditsPage.qml
    ├── LicensesPage.qml
    ├── GplLicenseDialog.qml
    ├── DesignTokens.qml
    └── shared App*.qml controls
```

## Controller boundary

Each controller has one `#[cxx_qt::bridge]` module and a plain Rust backing
struct:

| QObject | Owns |
| ------- | ---- |
| `SearchController` | Qt result model, search/replace workers, tabs snapshots, history/profile bridge, container discovery, export, desktop actions |
| `SettingsController` | settings QProperties, save/reload, bundled licenses/credits, version and commit metadata |
| `RegexBuilderController` | pattern compilation, sample matching, serialized highlight ranges |
| `AiController` | opt-in checks, key operations, endpoint test, chat and evidence-summary workers |
| `DbController` | grexa-db open/query/validate/view operations, indexes, filesystem watcher |

The backing `*Rust` state is directly unit-tested without starting Qt. cxx-qt
itself verifies the QProperty, signal, invokable, and model signatures during
the generated C++ build.

### cxx-qt naming

Rust declarations are snake_case. Their QML names are camelCase:

```text
Rust qsignal record_paths_ready  -> QML handler onRecordPathsReady
Rust invokable list_views        -> QML call listViews()
Rust qproperty status_text       -> QML property statusText
```

Using the Rust spelling in QML is a fatal object-creation error.

### Thread rule

QObject properties and Qt models are touched only on the GUI thread.

```text
QML
  │ invokes
  ▼
QObject method
  │ starts
  ▼
Rust worker thread
  │ performs blocking engine/process/HTTP/filesystem work
  │ queues closure through cxx_qt::Threading
  ▼
GUI-thread model/property update
```

Search rows are coalesced at a frame-sized interval. Counters are throttled and
finalized once. Database and container probes also run off-thread.

## State ownership

### Process-wide state

`Workspace` owns:

- `SettingsStore`;
- `RecentPathsDb`;
- `SearchHistoryDb`;
- `SearchProfilesDb`;
- the active Fluent `Bundle`.

It is installed in a thread-local handle before QML loads. Controllers use
`with_workspace` rather than constructing independent stores.

### Search-page state

`SearchPage.qml` owns the tab list and form state:

- stable tab ID and label;
- path and term;
- regex and case flags;
- Content/Files mode;
- loaded-result filter.

Before switching away, QML calls `saveTabSnapshot(tabId)`. The Rust controller
stores result rows, counters, status, and the last search options. Returning
calls `restoreTabSnapshot(tabId)`.

There are at most eight tabs. Inactive snapshots share a 2,000,000-row budget;
old inactive snapshots are evicted rather than allowing unbounded session
growth.

### Persistent state

Settings and grexa-db records persist across launches. Search tabs and chat
bubbles do not.

## QML page map

| Page/component | Responsibility |
| -------------- | -------------- |
| `Main.qml` | ApplicationWindow, navigation, shared controllers, global shortcuts, recovery dialog |
| `SearchPage.qml` | Tabs, targets, result mode, filters, result list, export, replace and AI drawers |
| `SearchBar.qml` | Path, term, Regex/Case/Whole Word chips, Browse and Search |
| `ResultRow.qml` | Highlighted row, preview click, action context menu |
| `HistoryPage.qml` | Filtered completed-search list and form restore |
| `ProfilesPage.qml` | Filtered named profiles, load, delete |
| `RegexBuilderPage.qml` | Presets, pattern editor, sample, errors, highlights and matches |
| `SettingsPage.qml` | Auto-saved settings cards and key/endpoint actions |
| `DatabasePage.qml` | grexa-db schema, filter, validation and view UI |
| `RecordCard.qml` | Record path and frontmatter presentation |
| `ContextPreviewDialog.qml` | Numbered context lines and match marker |
| `AiChatPanel.qml` | AI opt-in state, evidence summary, messages, composer |
| `AboutPage.qml` | Product, version, commit, feature and project links |
| `CreditsPage.qml` | Runtime acknowledgements and filterable crate table |
| `LicensesPage.qml` | Bundled license/credits document viewer |

Shared controls (`AppTextField`, `AppComboBox`, `AppCheckBox`, `AppSpinBox`,
`AppSlider`, `AppFlatButton`, `PrimaryButton`, `FlagChip`, `Card`, `EmptyState`,
and `NavItem`) centralize palette and accessibility behavior.

## Theming

`DesignTokens.qml` is the single visual token source:

- spacing and radii;
- typography and weights;
- surface, text, separator, accent, warning, and error colors;
- animation durations;
- reduced-motion and high-contrast transforms.

Themes flow through three layers:

1. `Kirigami.Theme` attached properties on the window/page;
2. Qt `palette` overrides for Qt Quick Controls;
3. shared `App*.qml` wrappers.

The third layer is required because `org.kde.desktop` controls can disable
theme inheritance at the control boundary. New forms should use the shared
wrappers unless a raw control has been verified under system, light, dark,
OLED black, high-contrast, and reduced-motion settings.

The AppImage bundles Breeze icon themes, Qt SVG plugins, and the KDE desktop
control style when available. `icon_theme.cpp` adds bundled icon paths before
QML starts.

## Localization

`Main.qml` exposes:

```qml
function i18n(key) { return searchController.i18n(key); }
function i18nPlural(key, n) {
    return searchController.i18n_plural(key, n)
}
```

All shipped QML strings must use these Fluent-backed helpers. Add a key to
English, German, and Japanese together, then run:

```bash
python3 scripts/check_locale_sync.py
cargo test -p grexa-i18n
```

Do not add `qsTr()` strings.

## Desktop integration

The GUI uses small external/native boundaries:

- `QtQuick.Dialogs.FolderDialog` for directory selection;
- `org.freedesktop.FileManager1.ShowItems` through `gdbus`;
- `xdg-open` as reveal/editor fallback;
- editor-specific argv builders;
- `wl-copy` or `xclip` for clipboard actions;
- `gio trash`;
- `notify-send`;
- DBus activation or `wmctrl` for an already-running instance.

Every user-controlled path is passed as an argv element, not shell text.

## Build pipeline

[`build.rs`](../apps/grexa-gui/build.rs):

1. emits the Git commit SHA through `vergen-git2`;
2. declares `com.visorcraft.Grexa` version 1.0;
3. lists every QML file;
4. registers every bridge source;
5. bundles images through Qt resources;
6. compiles `icon_theme.cpp`;
7. lets cxx-qt generate and build the C++/QML module.

Editing QML requires a Cargo rebuild because QML ships inside the binary.

### Add a QML file

1. Add the SPDX header.
2. Create the file under `apps/grexa-gui/qml/`.
3. Add it to `QmlModule::qml_files` in `build.rs`.
4. Use Fluent keys for user text.
5. Use shared controls and accessibility metadata.
6. Run `cargo build -p grexa`.
7. Run `just verify-gui target/debug/grexa`.

### Add or change a QObject

1. Keep pure state in a testable Rust backing struct.
2. Add the bridge declaration and implementation.
3. Add a new bridge source to `build.rs` when creating a module.
4. Keep blocking work off the Qt thread.
5. Queue property/model changes back through `Threading::queue`.
6. Use camelCase names from QML.
7. Add the smallest Rust-side test for the behavior.
8. Build and run the GUI launch gate.

## Runtime failure diagnosis

The application reports:

```text
QML payload did not instantiate
```

when the root object fails. The underlying `QQmlError` list is not exposed by
the current cxx-qt-lib path used here.

Use `qmllint` against generated modules:

```bash
cargo build -p grexa
qmllint \
  -I target/debug/build/grexa-*/out/qt-build-utils/qml_modules \
  apps/grexa-gui/qml/Main.qml
```

Then run:

```bash
just verify-gui target/debug/grexa
```

For AppImage-only failures, retry once with:

```bash
QML2_IMPORT_PATH=/usr/lib/qt6/qml \
  target/appimage/Grexa-<version>-x86_64.AppImage
```

If that works, the bundle is missing a QML module. Fix AppImage deployment;
do not make the host import path a runtime requirement.
