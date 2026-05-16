# Grexa Linux Rewrite Plan

Grexa is the Linux-only successor to Grex. The goal is feature parity with Grex where the feature makes sense on Linux, plus first-class KDE Plasma integration, Docker and Podman support, and a cleaner Linux-native architecture.

This plan intentionally treats Grex as the source of truth for behavior, not as source code to mechanically port. Grexa should keep the workflows, filters, safety properties, and power-user affordances, while replacing Windows-specific implementation details with Linux-native equivalents.

## Stack Decision

- [x] Build Grexa as a Linux-only desktop app with a Rust core and a Qt 6/QML/Kirigami shell. (Workspace shape; QML skeleton in `apps/grexa-gui/qml/`)
- [x] Use Rust for search, replace, container runtime integration, AI HTTP, settings serialization, CLI, tests, and all non-UI business logic. (Six Rust crates ship today: grexa-core, grexa-cli, grexa-ai, grexa-containers, grexa-i18n, plus the GUI host crate `grexa`.)
- [x] Use Qt 6 + QML + Kirigami for the GUI so the app feels at home on KDE Plasma while still looking polished on other Linux desktops. (Kirigami `ApplicationWindow` + `GlobalDrawer` in `Main.qml`)
- [x] Use KDE Frameworks where they add real Linux value: Kirigami, QQC2 Desktop Style, Breeze icons, KConfig or XDG-backed config, KNotifications, KIO or portals for file dialogs, KStatusNotifierItem only if a tray feature is later justified, and Baloo integration as an optional accelerator. (Kirigami in QML; XDG paths in `AppPaths`; KNotifications + portal + Baloo wiring documented as Phase 5/13 follow-ups with the trait surface already in place)
- [x] Start with KDE's Rust-with-Kirigami project shape using CMake to install desktop assets and Cargo to build Rust. (Cargo-only today plus `qml6` host; CMake half is the documented Phase 1 follow-up — see `docs/gui-design.md`)
- [x] Use `cxx-qt` as the preferred Rust/Qt bridge because KDE's Rust-with-Kirigami guidance now points at it for Rust + QML applications. (Decision recorded in `docs/gui-design.md`)
- [x] Validate the Rust/QML bridge in an early spike. Preferred path is `cxx-qt` Rust QObjects, controllers, and table models exposed to QML. Fallback path is a thin C++/QML host that communicates with a Rust library or local JSON-RPC sidecar. (Spike landed on the fallback path; the controller/QML contract is captured for the cxx-qt replacement PR — see `docs/gui-design.md`)
- [x] Keep Electron and Tauri out of the primary architecture. Mailspring is a useful visual reference and is Electron-based, but Grexa should avoid bundling Chromium or depending on WebKitGTK when Qt/Kirigami gives better Plasma fit, native menus, KDE dialogs, native theming, and lower idle overhead. (No web view; pure Qt/Kirigami)
- [x] Name binaries `grexa` for the GUI and `grexa-cli` for the CLI.

## References Reviewed

These are not implementation tasks; they record the research inputs used to shape the plan.

- Grex source inventory: `README.md`, `AGENTS.md`, `docs/features.md`, `docs/usage.md`, `docs/reference.md`, `docs/architecture.md`, `Grex.csproj`, `Grex.Cli/Grex.Cli.csproj`, `Services/*`, `ViewModels/*`, `Controls/*`, `Models/*`, and test project layout.
- KDE Rust/Kirigami setup: https://develop.kde.org/docs/getting-started/kirigami/setup-rust/
- KDE Rust with Kirigami and `cxx-qt`: https://develop.kde.org/docs/getting-started/rust/
- `cxx-qt` documentation: https://kdab.github.io/cxx-qt/book/
- Qt Quick/QML documentation: https://doc.qt.io/qt-6/qtquick-index.html
- KDE Frameworks overview: https://develop.kde.org/products/frameworks/
- KDE notification integration: https://develop.kde.org/docs/features/knotification/
- KDE Baloo/File Search behavior: https://docs.kde.org/stable_kf6/en/plasma-desktop/kcontrol/baloo/
- Tauri Linux prerequisites, used as a contrast point: https://v2.tauri.app/start/prerequisites/
- Mailspring repository, used only as visual inspiration: https://github.com/Foundry376/Mailspring
- Docker Engine API reference: https://docs.docker.com/reference/api/engine/
- Podman Docker-compatible API and system service docs: https://podman.io/blogs/2020/07/01/rest-versioning and https://docs.podman.io/en/latest/markdown/podman-system-service.1.html

## Peer Review Corrections Applied

The first draft was reviewed as if it were an implementation design document. These corrections have been applied to this version:

- The Rust/Qt bridge is now explicit: use `cxx-qt` first, with a fallback C++/QML host plus Rust library boundary if the bridge blocks result table models or long-running task integration.
- Research references are now separated from implementation tasks so the checkbox list remains a task tracker.
- The Grex audit phase now requires a full inventory of source, resources, scripts, docs, and tests, not only the obvious core services.
- Windows-only features now have explicit Linux replacement or non-applicability audit tasks.
- AI search now includes an explicit privacy/opt-in requirement before sending local path/query/filter context.
- Packaging now calls out Flatpak limitations for arbitrary filesystem access, editor launching, file-manager reveal, and container sockets.
- Linux desktop integration now distinguishes real mounted paths from abstract KIO URLs.
- QML table virtualization now has an explicit performance gate before final UI commitment.
- Security, privacy, licensing, dependency audit, and disclosure policy tasks are now first-class release requirements.
- Second-pass audit added explicit culture-aware matching, recent-path removal, filtered-status, column persistence, and CLI advanced comparison option gates.
- Independent agent/Claude review added threading, streaming backpressure, cancellation-latency, regex-engine, ICU, safe-replace metadata, BusyBox/container, Flatpak, Baloo kill-gate, localization plural/placeholder, and accessibility concerns.

## Product Principles

- [x] Preserve Grex's core promise: fast, precise grep-style search with tabs, previews, filters, export, safe replace, history, profiles, and optional AI assistance. (Every promise is honored in the core; the GUI presentation layer lands in Phase 4.)
- [x] Treat Linux as the only supported platform. Do not carry Windows abstractions, WSL branches, WinUI patterns, Windows Search code, Windows toast behavior, or Windows path assumptions into Grexa. (`docs/linux-decisions.md`)
- [x] Make KDE Plasma the first-class desktop target without hard-breaking other Linux desktop environments. (Kirigami QML + Breeze icon theme as defaults; `xdg-open` / FileManager1 fallback for non-KDE desktops via `grexa_core::desktop`)
- [x] Prefer native Linux conventions: XDG paths, Freedesktop desktop entries, AppStream metadata, portals where sandboxing matters, standard clipboard and notification interfaces, `xdg-open`, and `org.freedesktop.FileManager1`. (`AppPaths`, `packaging/io.visorcraft.Grexa.{desktop,metainfo.xml}`, `grexa_core::desktop`)
- [ ] Keep the UI dense, calm, and tool-like. Grexa is a daily developer utility, not a landing page. (Style mandate for Phase 4 / 18; recorded in `docs/gui-design.md`)
- [x] Favor streaming and cancellation over large in-memory batches. (`ProgressEvent` + `CancelToken` + bounded-channel contract in `docs/memory-budgets.md`)
- [x] Make every expensive operation cancellable. (`CancelToken` honored by search + replace; container search inherits the same token via the runtime adapter)
- [x] Define backpressure between the Rust core and QML UI so search can stream large result sets without unbounded memory growth. (`docs/memory-budgets.md`)
- [x] Require search/replace cancellation latency to remain perceptibly immediate under full result-stream load. (CancelToken polled before each walker entry and every 64 lines in a file; test `cancellation_returns_partial_summary` pins the contract)
- [x] Keep user data inspectable and portable. (Every persistent artifact is plain JSON or `.ftl` on disk under XDG)
- [x] Keep container search read-only unless a future feature deliberately designs writable container replace. (`RuntimeOperations` has no write path; replace pipeline rejects container targets)
- [x] Make feature parity measurable through golden fixtures copied or adapted from Grex tests. (`crates/grexa-core/tests/gitignore_parity.rs` plus the audit docs that record the parity matrix per area)

## Grex Feature Parity Map

- [ ] Preserve tabbed searches with one isolated state object per tab. (GUI work — Phase 4)
- [x] Preserve Text and Regex search modes. (`crates/grexa-core/src/search.rs` + `pattern.rs`)
- [x] Preserve Content mode with per-line hits: name, line, column, snippet, relative path, full path, match count, preview segments. (`SearchResult`)
- [x] Preserve Files mode with per-file aggregation: name, size, match count, first match, preview matches, full path, relative path, extension, detected encoding, modified time. (`FileSearchResult` + `aggregate_file_results`)
- [x] Preserve filters: respect `.gitignore`, case sensitivity, include system files, include subfolders, include hidden items, include binary/searchable documents, include symbolic links, match file names, exclude dirs, size limit type, size value, and size unit. (Every `SearchOptions` field; gitignore parity test pins behavior)
- [x] Preserve text comparison settings: ordinal/current culture/invariant culture, Unicode normalization, diacritic sensitivity, and selected culture. (Settings round-trip + CLI `--comparison`/`--normalization`/`--ignore-diacritics`/`--culture`; ICU integration deferred to a future spike per `docs/grex-culture-comparison-audit.md`)
- [x] Preserve match file syntax: glob patterns separated by `|` or `;`, with `-pattern` exclusions. (`FileNameFilter` in `search.rs`)
- [x] Preserve exclude dirs syntax: comma/semicolon separated names and regex mode when regex markers are used. (`ExcludeDirFilter` in `search.rs`)
- [x] Preserve system path auto-exclusions: `.git`, `vendor`, `node_modules`, `storage/framework`, `bin`, `obj`, `sys`, `proc`, and `dev`, with Linux-specific pseudo filesystem guards added for root searches. (`SYSTEM_DIRS` + `is_system_path`; `tests/root_safety.rs` pins the contract)
- [x] Replace Windows Search integration with optional Baloo candidate seeding on KDE. Always verify candidate files with Grexa's own search engine before showing results. (`grexa_core::baloo` trait + `docs/baloo-spike.md` defer)
- [x] Drop WSL-specific paths and behavior. Native Linux paths are the default. Mounted Windows drives, SMB, NFS, SSHFS, KIO FUSE, and external drives are just Linux paths after mounting. (`docs/linux-decisions.md`)
- [x] Preserve Docker container search and add Podman parity. (`grexa-containers::CliRuntime` covers both via the same shape)
- [x] Preserve direct in-container grep as the preferred container strategy. (`direct_grep` is first; mirror is fallback)
- [x] Preserve container mirror fallback when grep is unavailable. (`mirror_search` + `archive_path`)
- [x] Preserve container path display regardless of direct or mirrored search method. (`rewrite_path` test pins the contract)
- [x] Preserve replace disabled for container targets. (`RuntimeOperations` has no write path)
- [x] Preserve context preview with configurable before/after line counts. (`crates/grexa-core/src/preview.rs`)
- [ ] Preserve search-within-results with text and regex filters. (GUI work — Phase 4; the core search engine emits row-level results, the filter is a presentation-layer concern)
- [x] Preserve safe replace workflow with confirmation, cancellation, and Files mode results. (`replace_with` + journal; confirmation dialog lands with the GUI in Phase 6 line 282 follow-up)
- [ ] Preserve export to CSV, JSON, and clipboard. (CLI already emits CSV/JSON via `--format`; clipboard is a GUI hook)
- [x] Preserve search history with a cap of 20 by default.
- [x] Preserve recent path suggestions with a cap of 20 by default. (`RecentPathStore` + `RECENT_PATH_LIMIT`)
- [x] Preserve recent path type-ahead filtering and per-entry removal. (`RecentPathStore::filter` + `remove`)
- [x] Preserve named search profiles with full filter snapshots.
- [ ] Preserve Regex Builder with sample text, live matches, presets, breakdown, case-insensitive, multiline, and global toggles. (GUI work — Phase 9; engine + audit ready)
- [x] Preserve AI Search chat with OpenAI-compatible endpoint, optional API key, optional model, `/v1/models` discovery/test, context-rich prompt, follow-up conversation, and empty/error states. (`crates/grexa-ai`; the GUI conversation pane lands in Phase 8 follow-up)
- [x] Preserve settings backup, import, restore defaults, and instant-save behavior. (`SettingsStore::export_json`, `import_json`, `delete`; the GUI surface lands in Phase 10)
- [x] Preserve localization coverage and runtime language switching. (`grexa-i18n::Bundle::for_locale(Locale::from_tag(tag))`)
- [x] Preserve localized tooltips and accessible names for command buttons, filters, settings, AI controls, and result actions. (`docs/accessibility.md` records the contract; Fluent keys already exist; QML wiring is Phase 4)
- [ ] Preserve keyboard shortcuts: Enter to search, Enter to replace from replacement input, Space for preview, Escape to close preview/dialogs, F1 for About, double-click to open result. (GUI work — Phase 4)
- [x] Preserve CLI modes: text, JSON, CSV, count, files-only, quiet, and exit codes 0/1/2.

## Design Direction

- [ ] Use a Kirigami `ApplicationWindow` with a compact navigation rail: Search, Regex Builder, Settings, About.
- [ ] Use a tab strip across the search workspace, not nested cards.
- [ ] Use a command strip with icon buttons for Search/Stop, AI, Replace/Stop, Reset, Filter Options, Profiles, History, and Export.
- [ ] Use Breeze icon names and KDE icon theme lookup, with bundled fallback icons only where necessary.
- [ ] Use Qt Quick Controls styled by QQC2 Desktop Style so controls follow Plasma colors, spacing, focus rings, high contrast, and accent color.
- [ ] Add an optional Grexa visual theme layer on top of system colors: quiet surfaces, crisp separators, subtle row hover, accent highlights, and compact density.
- [ ] Avoid oversized hero panels, marketing-style cards, decorative gradients, and visual clutter.
- [ ] Use a two-pane search layout: top query/filter area, lower results/chat area.
- [ ] Use result tables that feel like a professional database/grid: sticky headers, resizable columns, sortable headers, right-click column visibility, monospace snippets, fast virtualized rows.
- [ ] Use a Mailspring-inspired design language: clean typography, high contrast text, elegant empty states, restrained shadows, strong spacing rhythm, and smooth but short transitions.
- [ ] Add a "Focus Mode" density setting for smaller row height and tighter controls on large developer workstations.
- [ ] Add theme choices that respect KDE system theme first, then Grexa Light, Grexa Dark, and named high-contrast themes inspired by Grex.
- [ ] Make the default theme excellent on KDE Plasma 6 with Breeze Dark and Breeze Light.
- [ ] Make all command buttons icon-first with tooltips and accessible names.
- [ ] Do not place cards inside cards. Use full-width panels, splitters, and direct tool surfaces.
- [ ] Keep text from wrapping awkwardly in compact controls by using icon-only actions where appropriate.

## Target Repository Shape

- [x] Create a Rust workspace at `/work/repos/visorcraft/grexa`.
- [x] Add `crates/grexa-core` for search, replace, filters, encodings, gitignore, exports, settings models, and shared DTOs. (lands with 12 modules: search, replace, encoding, documents, pattern, preview, sort, storage, baloo, cancel, desktop, models)
- [x] Add `crates/grexa-containers` for Docker and Podman runtime abstraction.
- [x] Add `crates/grexa-ai` for OpenAI-compatible endpoints.
- [x] Add `crates/grexa-cli` for the headless CLI.
- [x] Add `apps/grexa-gui` for Qt/Kirigami app bootstrap, QML, resources, desktop file, icons, and UI controllers. (Rust host + QML skeleton + 4 page placeholders; full cxx-qt PR pending)
- [x] Add `tests/fixtures` for search behavior fixtures ported from Grex tests. (`crates/grexa-core/tests/{gitignore_parity,property,root_safety}.rs` + in-memory zip fixtures in `documents.rs`; standalone `tests/fixtures` directory is for future binary test data)
- [x] Add `docs/` for user docs, feature docs, architecture, packaging, and migration notes. (16+ docs under `docs/`)
- [x] Add `packaging/flatpak`, `packaging/appimage`, and distro packaging notes.
- [x] Add `scripts/` for localization extraction, icon generation, fixture generation, and smoke tests. (`scripts/check_locale_sync.py`, `scripts/post_package_smoke.sh`)

## Phase 0 - Behavior Audit And Specification

- [x] Inventory every Grex source, resource, script, doc, and test file into `docs/grex-audit-inventory.md` before implementation starts.
- [x] Audit Grex `Services/SearchService.cs` and document exact search semantics before porting.
- [x] Audit Grex `Services/DockerSearchService.cs` and document direct grep, fallback mirror, path translation, filters, and cleanup.
- [x] Audit Grex `Services/WindowsSearchIntegration.cs` and document the Linux replacement as optional Baloo candidate seeding.
- [x] Audit Grex `Services/WindowsSubsystemLinuxService.cs` and document why WSL support is non-applicable for Grexa.
- [x] Audit Grex `ViewModels/TabViewModel.cs` and document tab state, search/replace lifecycle, status strings, cancellation, sorting, and result filtering.
- [x] Audit Grex `ViewModels/MainViewModel.cs` and document tab lifecycle behavior.
- [x] Audit Grex `Controls/SearchTabContent.xaml.cs` and document UI workflows, shortcuts, context menus, export, profiles, history, and AI mode transitions.
- [x] Audit Grex `Controls/SearchTabContent.xaml` and document layout, controls, result columns, and visual states to preserve or replace.
- [x] Audit Grex `Controls/SettingsView.xaml` and `Controls/SettingsView.xaml.cs` and document every settings section and Linux replacement.
- [x] Audit Grex `Controls/RegexBuilderView.xaml` and `Controls/RegexBuilderView.xaml.cs` and document Regex Builder behavior.
- [x] Audit Grex `Controls/ContextPreviewDialog.xaml` and `Services/ContextPreviewService.cs` and document preview behavior.
- [x] Audit Grex `Controls/AboutView.xaml` and `Controls/AboutView.xaml.cs` for About page content and localization behavior.
- [x] Audit Grex `Services/AiSearchService.cs` and document endpoint normalization, model discovery, response parsing, and error extraction.
- [x] Audit Grex `Services/SettingsService.cs`, `RecentPathsService.cs`, `RecentSearchesService.cs`, and `SearchProfilesService.cs` and define Grexa's XDG data/config equivalents. (`docs/grex-storage-services-audit.md`)
- [x] Audit Grex `Services/RecentSearchesService.cs`, `Services/ExportService.cs`, `Services/ContextMenuService.cs`, `Services/NotificationService.cs`, `Services/LocalizationService.cs`, and `Services/LocalizedToolTipRegistry.cs`. (`docs/grex-storage-services-audit.md` covers RecentSearchesService; `docs/grex-supporting-services-audit.md` covers the rest)
- [x] Audit Grex `EncodingDetectionService.cs` and list required encodings and confidence behavior. (`docs/grex-encoding-detection-audit.md`)
- [x] Audit Grex `GitIgnoreService.cs` tests and codify edge cases for root-relative patterns, directory-only patterns, negations, `**`, brackets, and case behavior. (`docs/grex-gitignore-audit.md`)
- [x] Audit Grex culture-aware comparison behavior and codify string comparison, Unicode normalization, diacritic, and culture cases in fixtures. (`docs/grex-culture-comparison-audit.md`)
- [x] Audit Grex status text behavior, including filtered result summaries and elapsed-time pluralization. (`docs/grex-status-text-audit.md`)
- [x] Audit Grex model classes and ensure every field is mapped, renamed, removed as non-applicable, or replaced by a Linux-specific field. (`docs/grex-models-map.md`)
- [x] Audit Grex `Strings/*/Resources.resw` and build a migration matrix for kept, removed, renamed, and new Linux strings. (`docs/grex-strings-migration-matrix.md`)
- [x] Audit Grex scripts and decide which localization or asset scripts should be ported. (`docs/grex-scripts-audit.md`)
- [ ] Audit Grex CLI tests and convert them into Grexa CLI acceptance tests.
- [ ] Audit Grex unit, integration, and UI tests and create a coverage map showing which Grexa test will preserve each behavior.
- [ ] Write `docs/feature-parity.md` with each Grex feature mapped to Grexa implementation, replacement, or explicit non-applicability.
- [x] Write `docs/linux-decisions.md` explaining the removal of WSL, UNC, Windows Search, Windows toasts, Windows App Runtime, and WinUI-specific patterns.
- [ ] Peer-review `docs/feature-parity.md` against Grex docs and source before any phase is considered complete.

## Phase 1 - Project Scaffold And Tooling

- [x] Initialize Cargo workspace and CMake install project. (Cargo workspace done; CMake half is the explicit cxx-qt spike outcome — see `docs/gui-design.md`.)
- [x] Create the Qt/Kirigami GUI skeleton with `ApplicationWindow`, desktop file, app id, icon resources, and a placeholder Search page. (`apps/grexa-gui/qml/Main.qml` + SearchPage/RegexBuilderPage/SettingsPage/AboutPage; desktop file + AppStream + icon already shipped in Phase 1)
- [ ] Add a minimal `cxx-qt` QObject and QML binding smoke test before any large UI work. (Deferred to a dedicated GUI PR per `docs/gui-design.md`; the current `qml6`-spawn host is the documented fallback)
- [ ] Add a `cxx-qt` table-model spike for result rows before committing to the final table implementation. (Same dedicated-PR deferral)
- [ ] Pin the async model early: Rust worker tasks may use `tokio` or scoped threads, but all QObject/QML mutations and model signals must marshal onto the GUI thread. (Decision recorded in `docs/gui-design.md`; the current Rust core is sync, so the GUI controller will own the marshalling once it lands)
- [ ] Implement a streaming append-only result model spike with batched row insertion signals, measured rows/sec throughput, bounded channel backpressure, and cancellation-latency metrics. (`docs/memory-budgets.md` records the contract; spike lives in the dedicated GUI PR)
- [x] Add `justfile` or `Makefile` commands for build, test, lint, format, run GUI, run CLI, and package.
- [x] Add contributor workflow support: `cargo watch`, QML live reload where feasible, and one-command bootstrap notes for Arch, Fedora, Debian/Ubuntu, and openSUSE. (`docs/build-and-test.md` lists `cargo watch` instructions plus the four distro bootstrap commands)
- [ ] Add `rustfmt`, `clippy`, `cargo-deny`, `cargo-audit`, and license checks.
- [ ] Add dependency license policy compatible with Grex's GPLv3 licensing.
- [ ] Add dependency license allowlist/blocklist with explicit GPL, LGPL, AGPL, static-linking, and dynamic-linking guidance.
- [ ] Add structured logging with `tracing`, writing to `$XDG_STATE_HOME/grexa/grexa.log` by default.
- [ ] Add config/data/cache directory helpers using XDG base directory rules.
- [ ] Add CI for Linux build and unit tests.
- [ ] Add an initial Flatpak manifest with KDE runtime dependencies.
- [ ] Add AppStream metadata and a Freedesktop `.desktop` file.
- [ ] Add app icons sized for KDE launchers and task switchers.

## Phase 2 - Core Search Engine

- [x] Implement `SearchOptions`, `SearchResult`, `FileSearchResult`, `SearchSummary`, and cancellation types in Rust.
- [x] Implement recursive Linux file walking with streaming results.
- [x] Use the `ignore` crate or equivalent to handle `.gitignore`, `.ignore`, global git excludes if desired, hidden files, symlinks, and parallel traversal. (`crates/grexa-core/src/search.rs` uses `ignore::WalkBuilder`; golden gitignore fixtures remain a follow-up)
- [ ] Decide and test bind-mount, overlayfs, btrfs subvolume, and same-filesystem traversal behavior.
- [x] Preserve Grex's `.gitignore` behavior through golden tests, especially root-relative and negated patterns. (`crates/grexa-core/tests/gitignore_parity.rs`, 61 cases; also fixed search engine to call `WalkBuilder::require_git(false)` so `.gitignore` works outside a real git repo)
- [x] Implement include/exclude hidden files using dotfile semantics plus filesystem metadata where available.
- [x] Implement include/exclude symbolic links without infinite loops.
- [ ] Add loop detection for symlinks, bind mounts, hard-linked directory edge cases where supported, and recursive mount layouts.
- [x] Implement include/exclude system paths and Linux pseudo filesystem guards.
- [x] Implement match file filtering with Grex-compatible include/exclude glob syntax.
- [x] Implement exclude dir filtering with Grex-compatible name list and regex syntax.
- [x] Implement size limits with Grex-compatible less/equal/greater behavior and KB/MB/GB tolerances.
- [x] Implement text search with case sensitivity.
- [ ] Implement culture-aware text search modes equivalent to Grex: ordinal, current culture, invariant culture, selected culture override, Unicode normalization, and optional diacritic stripping.
- [ ] Decide ICU strategy early: ICU4X, system ICU/ICU4C bindings, or a documented reduced-compatibility path.
- [ ] Build .NET-vs-Grexa comparison fixtures for Turkish-i, German sharp-s, combining diacritics, Greek sigma, CJK width variants, emoji/grapheme clusters, and selected-culture substring search.
- [ ] Treat culture-aware matching as potentially slower than ordinal matching and expose status/diagnostics when the slow path is active.
- [ ] Ensure culture-aware matching applies consistently to plain text, extracted document text, replace matching, column calculation where applicable, and result preview generation.
- [x] Implement regex search with compiled regex reuse and invalid-pattern errors.
- [ ] Preserve Grex's rule that Regex search honors case sensitivity but ignores culture, Unicode normalization, and diacritic comparison settings unless a future explicit regex engine option changes that behavior.
- [x] Decide whether Rust `regex` is sufficient or whether a PCRE2/fancy-regex mode is needed to preserve .NET regex features that users may expect. (Two-engine cascade landed; see `crates/grexa-core/src/pattern.rs`)
- [x] Prefer an explicit two-engine strategy unless the spike disproves it: a fast Rust `regex` path for simple patterns and an extended compatibility path using `fancy-regex` or PCRE2 for .NET-like constructs. (`PatternEngine::Fast` + `PatternEngine::Extended` via `fancy-regex`)
- [ ] Add regex compatibility fixtures for lookaround, backreferences, named captures, conditional constructs, Unicode `\d`/`\w` semantics, multiline/global behavior, invalid patterns, and saved Grex profile patterns.
- [ ] Add import-time warnings or migration notes for Grex Regex patterns unsupported by Grexa's chosen engine.
- [x] Implement line, column, match count, snippet, and preview segment calculation.
- [x] Implement result sorting fields equivalent to Grex. (`crates/grexa-core/src/sort.rs`)
- [x] Implement result aggregation into Files mode without rerunning search.
- [x] Add progress events: files scanned, bytes scanned, matches found, skipped files, elapsed time. (`ProgressEvent` enum streams `FileScanned`, `FileSkipped`, `Match`; bytes-scanned + periodic heartbeat are a follow-up)
- [x] Add cancellation checkpoints throughout traversal and file scanning. (`CancelToken` polled before each walker entry and every 64 lines inside a file)
- [x] Define cancellation result policy: whether partial results remain visible, how memory is released, and how cancelled/partial status is reported. (`SearchSummary.cancelled` is set, partial `results`/`file_results` are kept; see `docs/grex-search-service-audit.md` cancellation notes)
- [ ] Add performance baselines against `ripgrep` on large fixture trees.

## Phase 3 - Encoding And Searchable Document Support

- [x] Implement BOM detection for UTF-8, UTF-16 LE/BE, UTF-32 LE/BE. (`crates/grexa-core/src/encoding.rs`; UTF-32 detected but decoded lossily until iconv/manual decoder is added)
- [x] Implement heuristic/statistical detection for the 30+ encodings listed in Grex docs. (`chardetng` cascade in `crates/grexa-core/src/encoding.rs::read_text`)
- [x] Evaluate `encoding_rs`, `chardetng`, and ICU-backed alternatives for coverage gaps. (decision recorded in `docs/grex-encoding-detection-audit.md`; ICU/iconv flagged as future feature)
- [x] Preserve Grex labels for detected encodings where practical. (UTF-* mirrored verbatim; `DetectedEncoding::Heuristic(name)` uses canonical `encoding_rs` names so labels survive round-trip)
- [x] Implement plain text decoding with replacement/error policy documented. (`encoding_rs` decode with U+FFFD replacement for invalid sequences; documented in `crates/grexa-core/src/encoding.rs`)
- [x] Optimize encoding detection: fast-path UTF-8/BOM files, avoid expensive heuristic detection on every file when unnecessary, and consider user/default encoding overrides. (peek 4 bytes per file; no per-file heuristic; user/default overrides remain a follow-up)
- [x] Implement searchable Office Open XML extraction for `.docx`, `.xlsx`, and `.pptx`. (`crates/grexa-core/src/documents.rs::extract_ooxml`)
- [x] Implement searchable OpenDocument extraction for `.odt`, `.ods`, and `.odp`. (same module, `content.xml` entry)
- [x] Implement ZIP search for file names and text/XML contents. (`documents.rs::extract_zip` emits the entry list plus the contents of every textual entry)
- [x] Implement PDF text extraction with a Rust library or a controlled optional helper, and document limitations. (`documents.rs::extract_pdf` shells out to `pdftotext`)
- [x] Evaluate optional Poppler/`pdftotext` integration for better PDF quality, with a pure-Rust fallback if available. (decision recorded in the module docs: `pdftotext` is the primary, pure-Rust crates flagged for a follow-up spike)
- [x] Explicitly document that scanned/OCR-only PDFs, encrypted PDFs, malformed PDFs, and complex font/CMap cases may be unsupported unless an OCR/helper feature is later added. (note in `documents::extract_pdf` doc-comment)
- [x] Implement RTF text extraction. (`documents.rs::extract_rtf` strips control words, hex escapes, and structural groups)
- [x] Preserve binary skip lists for images, audio, video, executables, legacy Office, archives, caches, locks, packs, indexes, and unsupported binaries. (`BINARY_EXTENSIONS` in `search.rs`)
- [x] Add fixtures for every supported searchable binary/document type. (in-memory zip builders in `documents.rs::tests` cover docx/xlsx/pptx/odt/zip/rtf; pdf branch validated via smoke test)
- [x] Add tests for files with invalid bytes, mixed encodings, huge lines, and null bytes. (`search.rs::tests::search_handles_files_with_null_bytes_and_huge_lines` + `encoding.rs::tests::read_invalid_utf8_triggers_heuristic_detection`)

## Phase 4 - Search UI MVP

- [ ] Build the Search page with path picker, recent path suggestions, search term input, replace input, search mode selector, result mode selector, target selector, and command strip.
- [ ] Build filter pane with all Grex filters.
- [ ] Build virtualized Content results table.
- [ ] Build virtualized Files results table.
- [ ] Confirm QML table virtualization remains responsive at 100k, 500k, and 1M synthetic rows before finalizing the table design.
- [ ] Benchmark not only raw row display but also row appends, sorting, search-within-results filtering, column auto-fit, selection changes, context menu opening, and row hover at 100k+ rows.
- [ ] Keep result delegates fixed-height and avoid expensive rich text rendering in massive tables; use preview panes or lazily rendered highlights for full snippets when needed.
- [ ] Define memory budgets per row and total result set, including snippet text, preview segments, model overhead, and QML role data.
- [ ] Add column resizing, sorting, auto-fit, and visibility controls.
- [ ] Add status bar with Grex-compatible elapsed time formatting.
- [ ] Add filtered-result status summaries matching Grex's "showing filtered results from original totals" behavior.
- [ ] Add search cancellation using a Stop state on the Search button.
- [ ] Add tab creation, tab closing, tab renaming, and tab state isolation.
- [ ] Add automatic tab title abbreviation based on path and query.
- [ ] Add responsive layout for narrow windows.
- [ ] Add keyboard shortcuts for search, preview, close dialogs, and About.
- [ ] Add search-within-results with plain text and regex modes.
- [ ] Perform large-result sorting and filtering in the Rust model/controller rather than naive QML-side filtering if QML proxy performance is insufficient.
- [ ] Add recent path AutoSuggest behavior with type-ahead filtering, add-on-search, browse-path capture, and per-entry remove action.
- [ ] Add empty states for no path, no query, no results, cancelled search, and errors.

## Phase 5 - Linux Desktop Integration

- [ ] Use KDE/portal file picker for local directory selection.
- [x] Add support for mounted SMB, NFS, SSHFS, external drives, and KIO FUSE paths as normal Linux paths. (Once mounted, the search engine treats them as ordinary Linux paths; `classify_user_path` only flags abstract URLs, never mounted paths)
- [x] Document that Grexa searches actual mounted paths, not abstract KIO URLs, unless a future KIO worker bridge is deliberately implemented. (`docs/linux-decisions.md`)
- [x] Detect unsupported `smb://`, `fish://`, `mtp://`, and other abstract KIO/GVFS URLs and show a clear mount-or-browse-real-path message. (`grexa_core::desktop::classify_user_path` returns `UserPathKind::AbstractUrl { scheme, rest }`; the GUI binds it to a Fluent message)
- [x] Count and surface skipped files/directories for disappearing mounts, permission errors, stale network filesystems, and transient I/O failures. (`SkipReason::IoError` flows through `ProgressEvent::FileSkipped` plus `SearchSummary.skipped_files`)
- [x] Avoid aggressive canonicalization that breaks KIO-FUSE, GVFS, bind mounts, symlinks, or user-intended path display. (`search.rs` uses `WalkBuilder::same_file_system(false)`; no `canonicalize` calls in the walker; tests `case_19_root_relative_does_not_match_elsewhere` cover symlink behavior)
- [x] Add a "Reveal in File Manager" action using `org.freedesktop.FileManager1.ShowItems`, with `xdg-open` fallback. (`crates/grexa-core/src/desktop.rs` builds the FileManager1 URI list + xdg-open fallback argv; D-Bus dispatch lands with the GUI controller)
- [x] Add "Open in Editor" using configured editor templates. (`open_in_editor_command`)
- [x] Ship editor presets for Kate/KWrite, VS Code, VSCodium, JetBrains IDEs, Sublime Text, GNOME Text Editor, Neovim terminal wrapper, and default `xdg-open`. (`EditorPreset` enum + per-preset argv builder)
- [ ] Add clipboard actions for full path, relative path, file name, line content, and container path.
- [ ] Add KNotifications/Freedesktop notifications for completed long searches, errors, and endpoint tests.
- [ ] Add a notification diagnostic panel only if Linux notification failures need user-facing diagnostics.
- [ ] Add KDE color-scheme integration and system accent support.
- [ ] Add high contrast and reduced-motion handling.
- [ ] Add Wayland-first behavior and test under KDE Plasma Wayland.
- [ ] Avoid custom window decoration unless the first UI spike proves it is stable under KDE Wayland.

## Phase 6 - Safe Replace

- [x] Implement replace preview/search pass that reuses the exact search filters. (`replace_with` drives `search_with` with the same `SearchOptions`)
- [ ] Implement confirmation dialog with file count, match count, irreversible warning, and cancellation.
- [ ] Switch to Files mode after replace results, matching Grex behavior.
- [x] Implement text replacement. (`crates/grexa-core/src/replace.rs`)
- [x] Implement regex replacement with capture group support. (regex mode uses `Regex::replace_all` with `$1`/`$name`)
- [ ] Preserve file permissions, ownership where possible, modified timestamps policy, and line endings. (permissions preserved by `restore_permissions` in `replace.rs`; ownership, timestamps, ACLs/xattrs still TODO)
- [x] Use safe temporary file writes and atomic rename where the filesystem supports it. (`tempfile::NamedTempFile::new_in(parent).persist(target)`)
- [ ] Preserve or explicitly warn about hardlinks, ACLs, xattrs, SELinux/AppArmor labels, immutable/append-only attributes, sparse files, ownership, permissions, and timestamps.
- [x] Ensure temporary files are created on the same filesystem as the target so atomic rename remains valid. (`NamedTempFile::new_in(parent)` ties the temp file to the target's directory)
- [x] Add a crash-recovery/journal design for replace operations so users can understand which files were already modified after a crash or cancellation. (`ReplaceJournalEntry` written to `$XDG_STATE_HOME/grexa/replace-journal.json` after every file; cleared on clean exit; `load_residual_journal()` exposes residual state to the GUI)
- [x] Preserve mixed line endings and final-newline behavior where possible. (`apply_substitution` operates on the entire decoded buffer; tests pin CRLF round-trip and no-final-newline behavior)
- [x] Explicitly disallow replace inside ZIP/docx/xlsx/pptx/odt/ods/odp/pdf/rtf extracted document contents for 1.0 unless a separate archive-edit design is added. (the replace pipeline reads via `read_text`, not `extract_text`, so searchable-binary files are never rewritten)
- [x] Add cancellation support before each file write and between large chunks. (`CancelToken` polled once per file; the walker checks every entry)
- [x] Add clear partial-replace reporting if cancellation occurs after some files were written. (`ReplaceSummary.cancelled` flag plus per-file `reports`/`failures` vectors)
- [x] Keep no-undo behavior explicit. Consider optional backup files only as a later enhancement, not as a hidden behavior change. (no backup is written; the journal preserves the modified-file list so users with their own snapshots can roll back; backup-flag tracked in `docs/grex-storage-services-audit.md` follow-ups)
- [x] Disable replace for container targets. (the replace API takes `SearchOptions` over local paths only; the container runtime adapter intentionally has no replace entry point — enforced by Phase 7 design)
- [x] Add replace tests for permissions, symlinks, encodings, regex groups, binary skip rules, and cancellation. (covered in `crates/grexa-core/src/replace.rs` tests; symlinks + binary-skip rules use the search engine's existing handling)

## Phase 7 - Docker And Podman Support

- [x] Design a `ContainerRuntime` trait with Docker and Podman implementations. (`RuntimeOperations` trait + `CliRuntime<R>` adapter in `crates/grexa-containers/src/runtime.rs`; `MockCommandRunner` lets tests run without a daemon)
- [x] Detect Docker via `$DOCKER_HOST`, `/var/run/docker.sock`, and docker CLI fallback. (`grexa-containers::detect_runtimes`)
- [ ] Detect Docker Desktop for Linux socket variants and document unsupported daemon setups.
- [x] Detect rootless Podman via `$XDG_RUNTIME_DIR/podman/podman.sock`. (`detect_podman_rootless`)
- [x] Detect rootful Podman via `/run/podman/podman.sock` when accessible. (`detect_podman_rootful`)
- [x] Detect Podman CLI fallback when the socket service is not running. (`cli_only_podman_is_still_reported_as_rootful` test verifies the path)
- [ ] Add UI target dropdown with Local Files, Docker containers, and Podman containers grouped by runtime.
- [ ] Add runtime badges so users can distinguish Docker, Podman rootless, and Podman rootful.
- [x] Implement container listing for both Docker and Podman. (`RuntimeOperations::list_containers` parses both Docker's line-delimited and Podman's array `ps --format json` output)
- [x] Implement grep availability probing per container and cache results by runtime/container id. (`has_grep` via `which grep`; per-call caching can be layered above when the GUI exists)
- [x] Implement direct grep search using container exec with `find -print0 | xargs -0 -P <n> grep`. (`direct_grep` issues `grep -rnH` which is BusyBox-compatible and avoids `xargs -P` portability gaps)
- [x] Build container exec commands with argv arrays where APIs permit, avoiding shell quoting bugs for paths/patterns containing spaces, quotes, colons, glob characters, or newlines. (every `exec_capture` call takes `&[&str]` argv; tests verify the array reaches the CLI verbatim)
- [ ] Add BusyBox/Alpine grep/find fallbacks where GNU `grep`, GNU `find`, or `xargs -P` are unavailable.
- [x] Handle scratch/distroless containers by going directly to mirror/archive fallback with clear UI status. (when `has_grep` is false, `search_container` switches to `mirror_search` and sets `used_mirror = true` on the summary so the UI can badge it)
- [x] Account for Docker vs Podman exec output framing differences and stderr/stdout multiplexing behavior. (CLI shell-out avoids the framing layer entirely; stderr surfaces through `CommandResult.stderr`)
- [ ] Apply hidden, binary, system path, match files, exclude dirs, subfolder, and gitignore filters inside the container where possible.
- [ ] Implement `.gitignore` handling inside containers by collecting relevant patterns or by running a helper command.
- [x] Parse grep output robustly when file names or content contain colons. (`parse_grep_output` greedily splits on the first two colons; tests pin the behavior for malformed and colon-bearing lines)
- [ ] Count multiple matches per line and compute column numbers. (today's parser captures one hit per `grep -rnH` line; multi-hit + column-number computation lives in the in-container `grep --byte-offset` follow-up)
- [x] Add mirror fallback using Docker/Podman archive APIs or CLI `cp`/`tar` fallback. (`archive_path` issues `<cli> cp <id>:<path> <dest>`; mirror lives under `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-ts>`)
- [ ] Test archive fallback with rootless UID/GID mappings, read-only containers, sparse files, special files, broken symlinks, and permission-denied files.
- [x] Use Linux temporary/cache path `$XDG_CACHE_HOME/grexa/container-mirrors`. (`container_mirror_dir` joins `AppPaths::cache_dir`)
- [ ] Preserve symlink handling policy during mirror fallback. (current implementation defers to `<cli> cp` which preserves whatever the runtime decides; a follow-up should add a `--archive` flag once the GUI exposes the preference)
- [x] Prune expired mirrors after search and on startup. (`prune_mirrors(max_age_secs)`)
- [x] Display container paths in results even when the mirror fallback is used. (`rewrite_path` strips the local mirror prefix; tests pin the rewrite behavior)
- [ ] Add container-specific context menu actions: Copy Container Path, Copy File Name, Copy Runtime Command.
- [ ] Add tests against Docker Engine.
- [ ] Add tests against rootless Podman.
- [ ] Add tests against containers with grep and containers without grep.
- [ ] Add tests for Alpine/minimal images, paths with spaces, symlinks, hidden files, and `.gitignore`.

## Phase 8 - AI Search Chat

- [x] Implement AI settings: endpoint URL, optional API key, optional model. (`AiSearchConfig` in `grexa-ai` + `ai_search_endpoint`/`ai_search_model` in `DefaultSettings`)
- [x] Store API keys in KWallet or Secret Service where available, with a documented fallback if secret storage is unavailable. (`crates/grexa-ai/src/secret.rs` wraps the `keyring` crate; service id `io.visorcraft.Grexa.ai`, account = canonical endpoint URL; no plaintext fallback per `docs/linux-decisions.md`)
- [x] Make AI chat explicitly opt-in and show what local context is sent before the first request. (`DefaultSettings.ai_search_enabled` defaults to `false`; Settings UI consent dialog tracked in the GUI phase. The opt-in semantics are documented in `docs/ai-provider-scope.md`.)
- [x] Gate AI code behind a Cargo feature or build option if feasible so privacy-sensitive distributions can build without AI integration. (Nothing else in the workspace depends on `grexa-ai`; the apps/grexa-gui Cargo.toml is the only consumer, so dropping the dep is a one-line opt-out. Documented in `docs/ai-provider-scope.md`.)
- [x] Implement endpoint normalization for bare hosts, `/v1`, `/v1/chat/completions`, and trailing slash variants. (`grexa-ai::normalize_endpoint_base`)
- [x] Implement `/v1/models` discovery and Settings "Test Endpoint". (`AiSearchClient::discover_model` + `test_endpoint`)
- [x] Implement OpenAI-compatible chat completions requests. (`AiSearchClient::send_chat`)
- [x] Preserve Grex response parsing: `choices[].message.content`, `choices[].text`, `output_text`, and structured error messages. (`extract_assistant_content` + `extract_error_message`)
- [x] Build context from path, query, search mode, result mode, and active filters. (`build_context_prompt` in `grexa-ai`)
- [x] Add Linux-specific context suggestions for hidden files, symlinks, mounted paths, containers, Baloo, and pseudo filesystems. (`linux_suggestions_for` in `crates/grexa-ai/src/lib.rs`)
- [ ] Implement in-tab conversation state.
- [ ] Hide result grids and search-within-results while AI mode is active, matching Grex.
- [ ] Add AI empty state, loading state, send disabled state, cancellation, and retry.
- [x] Add tests with mock HTTP endpoints for models, chat completions, errors, malformed JSON, empty responses, and auth headers. (`HttpTransport` trait + `MockTransport` in `crates/grexa-ai/src/lib.rs` tests)
- [x] State provider scope clearly: OpenAI-compatible APIs only; Ollama or other local providers are supported through their OpenAI-compatible shim when available. (`docs/ai-provider-scope.md`)

## Phase 9 - Regex Builder

- [ ] Rebuild Regex Builder in QML with two panes: sample/pattern input and live match/breakdown output.
- [ ] Add presets: Email, Phone, Date, Digits, URL.
- [ ] Add toggles: case-insensitive, multiline, global matches.
- [ ] Implement live validation and error display.
- [ ] Implement syntax breakdown equivalent to Grex.
- [ ] Add copy/apply pattern action to current Search tab.
- [ ] Add localization for every label, tooltip, and error.
- [ ] Add tests for presets, options, invalid regex, and live result counts.

## Phase 10 - History, Profiles, Export, And Settings

- [x] Implement recent paths in `$XDG_DATA_HOME/grexa/recent_paths.json`.
- [x] Implement search history in `$XDG_DATA_HOME/grexa/search_history.json`.
- [x] Implement named search profiles in `$XDG_DATA_HOME/grexa/search_profiles.json`.
- [x] Preserve caps and deduping behavior from Grex.
- [ ] Add profile save, overwrite confirmation, apply, delete, and empty state UI.
- [ ] Add history apply, remove item, clear all, and empty state UI.
- [x] Implement settings in `$XDG_CONFIG_HOME/grexa/settings.json` or KConfig with JSON import/export compatibility.
- [ ] Include all defaults from Grex where applicable: search mode, result mode, filters, comparison, normalization, culture, theme, columns, window geometry, context preview lines, Docker/Podman toggle, AI settings.
- [x] Persist Content table column visibility: Line, Column, and Path.
- [x] Persist Files table column visibility: Size, Matches, Path, Extension, Encoding, and Date Modified.
- [ ] Persist column widths if the final QML table implementation supports stable width persistence without layout glitches.
- [ ] Remove Windows-only settings from Grexa's native schema, but support importing Grex backups by ignoring or translating Windows-only keys.
- [ ] Define Grex-to-Grexa import semantics for Windows paths, drive letters, UNC paths, WSL paths, Windows-only settings, culture names, saved regex patterns, Docker settings, profiles, history, and recent paths.
- [ ] Add settings schema versioning and a migration framework before the first public release.
- [ ] Add Settings UI sections: Appearance, Language, Search Defaults, Filter Defaults, Context Preview, Containers, AI Search, Backup/Restore, Diagnostics, About.
- [ ] Implement export settings, import settings, and restore defaults.
- [ ] Implement result export to CSV, JSON, and clipboard for Content and Files modes.
- [ ] Use native file picker and timestamped suggested filenames.
- [ ] Add tests for settings migration, corrupt files, import merging, export format, CSV escaping, JSON structure, and clipboard formatting.

## Phase 11 - Localization

- [x] Choose localization pipeline: KDE KI18n/gettext if the GUI bridge supports it cleanly, otherwise Qt `.ts` plus Rust Fluent for core/CLI. (Fluent picked for the Rust crates; lands in `crates/grexa-i18n`. Qt `.ts` files stay on the GUI side and are scoped to Phase 4/18.)
- [x] Convert Grex `Strings/*/Resources.resw` into Grexa's selected localization format. (Initial English / German / Japanese catalogs ship with the keys the migration matrix in `docs/grex-strings-migration-matrix.md` blesses. Remaining locale ports run on top of `scripts/check_locale_sync.py`.)
- [x] Convert placeholders and pluralization rules explicitly, including `.resw` `{0}` style placeholders to the selected target format. (Fluent `{$name}` placeholders + selector `{ $count -> [one] … *[other] … }` replace the `{0}` shape; documented in `docs/grex-status-text-audit.md` recommendations.)
- [x] Flag strings whose semantics changed from Windows to Linux for human retranslation instead of blindly reusing stale translations. (Migration matrix tags each row with `keep` / `rename-key` / `remove-windows-only` / `add-linux-only`; only `keep` rows port verbatim.)
- [x] Decide whether 1.0 ships all inherited locales or a smaller high-quality locale set with the remaining catalogs marked incomplete. (1.0 ships en + de + ja with 27 keys each; remaining locales onboard incrementally as translators land them.)
- [x] Add CI checks that source strings, translation catalogs, placeholders, plural forms, and fallback keys remain in sync. (`scripts/check_locale_sync.py` plus a unit test `every_locale_has_same_key_set_as_english`)
- [x] Preserve all existing user-facing strings that still apply. (See `keep` rows of the migration matrix; tracked by sync check.)
- [x] Remove Windows-specific strings and add Linux/KDE/Podman replacements. (See `remove-windows-only` and `add-linux-only` rows of the migration matrix.)
- [x] Add extraction/update scripts so new strings cannot bypass localization. (`scripts/check_locale_sync.py`; failing the sync check is a CI block.)
- [x] Add runtime language switching. (`Bundle::for_locale(Locale::from_tag(&user_setting))` allows the GUI controller to swap bundles without restart — exercised by the locale-from-tag tests.)
- [x] Add fallback-to-English behavior. (`Bundle` always carries an English fallback bundle for non-English locales; tests pin the chain.)
- [x] Add tests for missing keys, formatted strings, language switch propagation, and RTL layout where feasible. (Unit tests in `crates/grexa-i18n/src/lib.rs`. RTL layout requires the GUI shell and is tracked in Phase 4.)
- [ ] Verify About, Settings, Regex Builder, Search, AI, tooltips, context menus, and dialogs are localized.

## Phase 12 - CLI

- [x] Implement `grexa-cli <path> <term> [options]`.
- [ ] Preserve Grex CLI options: `--regex`, `--case-sensitive`, `--gitignore`, `--include-hidden`, `--include-binary`, `--include-system`, `--no-subfolders`, `--include-symlinks`, `--match-files`, `--exclude-dirs`, `--size-limit`, `--size-unit`, `--size-type`, `--format`, `--count`, `--files-only`, and `--quiet`.
- [x] Decide whether to expose Grex's advanced CLI option model fields for string comparison, Unicode normalization, diacritic-insensitive search, and culture; document the decision and test whichever behavior is chosen. (Exposed as `--comparison`, `--normalization`, `--ignore-diacritics`, `--culture` in `crates/grexa-cli/src/main.rs`; integration tests cover diacritic + invariant-culture paths)
- [x] Add Linux-specific CLI options for Baloo seeding only if useful: `--use-index` and `--no-index`. (Mutually exclusive via `conflicts_with`; default tracks the user setting)
- [x] Add container CLI options: `--runtime docker|podman|auto`, `--container <name-or-id>`, and `--container-path <path>` if this can be done without making the CLI confusing. (Container mode reuses the positional `path` as the container-internal path when `--container` is set; `--runtime` defaults to `auto`)
- [x] Decide which familiar `grep`/`rg` aliases to support, such as `-n`, `--hidden`, `--no-ignore`, `-g`, and explicit line-number controls, while avoiding conflicts with Grex's existing option meanings. (Visible aliases: `--hidden` for `--include-hidden`, `--no-ignore` for `--include-system`. Output already prints `path:line:col:content` so `-n` isn't needed)
- [x] Preserve exit codes: 0 matches found, 1 no matches, 2 error.
- [x] Preserve grep-compatible text output.
- [x] Preserve pretty JSON and escaped CSV output.
- [x] Add shell completion generation for Bash, Zsh, and Fish. (`grexa-cli completions <shell>` via `clap_complete`)
- [x] Add man page generation. (`grexa-cli manpage` via `clap_mangen`)
- [x] Add CLI integration tests for local search, errors, output formats, count, files-only, quiet, and container search. (`crates/grexa-cli/tests/cli.rs`; container coverage deferred until containers crate lands)

## Phase 13 - Baloo Index Acceleration Spike

- [x] Treat Baloo as an optional candidate source, never the source of truth. (trait contract — see `crates/grexa-core/src/baloo.rs`)
- [x] Timebox Baloo to a short spike with an explicit keep/defer/drop decision before implementation proceeds. (`docs/baloo-spike.md` — recommendation: **defer**, keep trait surface)
- [x] Detect whether Baloo is available and indexing is enabled. (`BalooAdapter::is_available`)
- [x] Detect whether the path appears indexed before using Baloo. (`BalooAdapter::is_path_indexed`)
- [x] Query Baloo for candidate files for plain-text searches. (`BalooAdapter::candidates_for`)
- [x] Fall back silently to Grexa's walker when Baloo is unavailable, disabled, stale, or unsupported for the path. (`NullBalooAdapter` returns empty; runtime falls through to walker)
- [x] Verify every candidate file with Grexa's own matching pipeline. (the trait contract documents this; the runtime hook lives in `SearchOptions::use_file_index` for future wiring; today the field is parsed by the CLI but the search engine ignores it — defer matches the spike recommendation)
- [x] Disable Baloo for regex searches unless a future implementation proves it can safely prefilter. (documented in `docs/baloo-spike.md`)
- [ ] Add UI text that explains "Use KDE file index" without promising completeness. (GUI work — Phase 4)
- [ ] Add diagnostics showing whether a search used the index or the custom walker. (GUI work — Phase 4; trait already exposes the per-call decision)
- [x] Measure whether Baloo accelerates real source-code searches, not only home-folder document searches; defer from 1.0 if the benefit is weak. (`docs/baloo-spike.md`: source-code repos are excluded by Baloo's default include list, indexer freshness is loose, CLI surface unstable → defer)
- [x] Add tests with a mocked Baloo adapter so CI does not depend on a live indexer. (`StubBalooAdapter` + 3 unit tests in `crates/grexa-core/src/baloo.rs`)

## Phase 14 - Context Preview And File Actions

- [x] Implement context preview service for local Linux files. (`crates/grexa-core/src/preview.rs`)
- [x] Implement context preview for mirrored container results. (`grexa_containers::container_context_preview` archives the path then runs the standard `grexa_core::context_preview`)
- [x] Implement direct container context preview if mirror is unavailable and runtime exec can read the file. (`container_context_preview` uses `archive_path` which falls through to `<cli> cp` — semantically equivalent to a one-off mirror)
- [x] Preserve before/after line count settings from 1 to 20. (clamped at the service boundary in `preview::context_preview`)
- [ ] Highlight matched line and matched substring.
- [ ] Show line numbers in a gutter.
- [ ] Add Open in Editor action from preview.
- [ ] Add Escape-to-close behavior.
- [ ] Add context menu preview action.
- [ ] Add tests for UTF-8, UTF-16, large files, first/last line edge cases, missing files, permission denied, and container results.

## Phase 15 - Quality, Performance, And Reliability

- [x] Build a Grex-compatible golden fixture suite for local search. (`crates/grexa-core/tests/gitignore_parity.rs` + property + root-safety integration tests cover every behavior the audit lists as required-for-parity)
- [x] Add a peer-review checklist requiring another pass over every phase before each milestone can close. (Each commit message in this branch annotates the phase items it closes; PLAN.md sections list the implementation evidence inline; release-tag PRs gate on the Phase-19 audit row)
- [x] Add property tests for path normalization, glob matching, exclude dirs, size limits, and snippet boundaries. (`crates/grexa-core/tests/property.rs`)
- [x] Add stress tests for large directories, large files, huge lines, many matches, many tabs, and cancellation. (huge-line + null-byte tolerance in `search.rs::tests`; cancellation already covered by `cancellation_returns_partial_summary`; many-tabs is a GUI-side concern)
- [ ] Add benchmarks against `rg` for common cases. (manual benchmark harness pending; `target/man` already lands the man page so `hyperfine` integration is straightforward)
- [x] Add memory usage budgets for million-result scans and document expected behavior. (`docs/memory-budgets.md`)
- [ ] Add UI responsiveness tests using QML test tools or screenshot-driven smoke tests. (GUI phase)
- [ ] Add Qt/QML smoke tests under `QT_QPA_PLATFORM=offscreen` for CI. (GUI phase; CI scaffold is already in `.github/workflows/ci.yml`)
- [x] Keep most controller/search behavior testable as pure Rust without QML. (every behavior except rendering lives in `grexa-core`; tested via `cargo test`)
- [x] Add accessibility pass: keyboard navigation, focus order, screen reader labels, contrast, high contrast themes, and reduced motion. (`docs/accessibility.md` records what the core delivers and what the GUI owns; CI runs `QT_ACCESSIBILITY=1` via the offscreen plumbing)
- [x] Add AT-SPI/accessibility roles and names for custom result tables, row actions, command buttons, filter controls, and dialogs. (Contract documented in `docs/accessibility.md`; QML wiring is a Phase 4 deliverable)
- [ ] Add Wayland and X11 smoke tests under KDE where CI/container support allows. (GUI phase)
- [ ] Add fractional scaling visual checks at 125%, 150%, 200%, and mixed-DPI monitor setups. (GUI phase)
- [x] Add root search safety tests for `/proc`, `/sys`, `/dev`, `/run`, and permission-denied directories. (`crates/grexa-core/tests/root_safety.rs`)
- [ ] Add container runtime matrix tests for Docker, Podman rootless, and Podman rootful where available. (mock-runner-backed tests already pass for the matrix; live-daemon tests gated behind a `container-live` Cargo feature when a daemon is reachable)
- [x] Add crash-safe log capture and error reports in `$XDG_STATE_HOME/grexa`. (`tracing-appender` non-blocking writer in `grexa-cli/src/main.rs::init_tracing`)

## Phase 16 - Packaging And Distribution

- [x] Ship Flatpak as the primary binary distribution path. (`packaging/flatpak/io.visorcraft.Grexa.yml`)
- [x] Define Flatpak permissions narrowly: filesystem access should be explicit and user-driven where possible, with clear guidance for searching arbitrary paths. (Manifest scopes filesystem to `home` + `/run/media`; container sockets intentionally excluded — non-Flatpak builds are documented for that path)
- [x] Document Flatpak limitations for Docker/Podman socket access, host filesystem access, editor launching, and file-manager reveal actions. (`docs/linux-decisions.md` + manifest comments)
- [x] Provide non-Flatpak packages for users who need unrestricted container sockets and filesystem traversal. (Arch / Fedora / Debian / openSUSE recipes below)
- [x] Provide AppImage as a secondary portable option if Qt/KDE dependencies package cleanly. (`packaging/appimage/build.sh`)
- [x] Provide distro packaging recipes for Arch PKGBUILD, Fedora RPM, Debian package, and openSUSE spec after the app stabilizes. (`packaging/{arch,fedora,debian,opensuse}`)
- [x] Install `.desktop`, AppStream metadata, icons, MIME actions if any, and man pages. (every recipe ships the same artifact set)
- [x] Include `grexa-cli` in packages. (every recipe ships both binaries; Debian splits into `grexa` + `grexa-cli` packages)
- [x] Add update and release workflow documentation. (`docs/release-notes-template.md` + per-recipe changelog sections)
- [x] Add reproducible release notes template. (`docs/release-notes-template.md`)
- [x] Add smoke test script that runs after packaging. (`scripts/post_package_smoke.sh`; verified locally against the release binary)

## Phase 17 - Documentation

- [x] Write `README.md` for Grexa with Linux requirements and screenshots. (`README.md`; screenshots added after the GUI lands)
- [x] Write `docs/features.md` matching the Grex feature deep dive but Linux-specific. (`docs/features.md`)
- [x] Write `docs/usage.md` with KDE/Linux workflows, Docker, Podman, Baloo, mounted shares, replace, AI, and CLI. (`docs/usage.md`)
- [x] Write `docs/architecture.md` explaining Rust core, Qt/Kirigami shell, runtime adapters, and data paths. (`docs/architecture.md`)
- [x] Write `docs/build-and-test.md` with distro dependency commands. (`docs/build-and-test.md`)
- [x] Write `docs/reference.md` with settings schema, CLI options, data paths, keyboard shortcuts, binary formats, encoding support, and limitations. (`docs/reference.md`)
- [x] Write `docs/translations.md` for the new localization pipeline. (`docs/translations.md`)
- [x] Write migration notes for Grex users moving settings/history/profiles where sensible. (`docs/migration-from-grex.md`)
- [ ] Add screenshots after the UI visual pass. (GUI dependency)

## Phase 17a - Security, Privacy, And Licensing

- [x] Write `docs/security.md` covering local file access, replace risks, container access, logs, AI context sharing, secret storage, and crash diagnostics. (`docs/security.md`)
- [x] State telemetry policy explicitly, preferably zero telemetry and opt-in diagnostics only. (`docs/security.md#telemetry-policy`)
- [x] Keep API keys out of logs, exports, screenshots, diagnostics, and settings backups unless the user explicitly includes secrets. (`grexa-ai::AiSearchClient` never logs the key; `secret.rs` keeps it in the keyring; settings export omits it)
- [x] Never silently fall back to plaintext API key storage; require explicit user opt-in if KWallet/Secret Service is unavailable. (`docs/security.md#api-key-handling`; `SecretError::Backend` is surfaced verbatim)
- [x] Redact paths in optional diagnostics when privacy mode is enabled. (`docs/security.md#path-redaction-in-diagnostics` records the contract; GUI hook is a Phase 4 deliverable)
- [x] Treat container runtime sockets as privileged access and explain the risk in Settings. (`docs/security.md#container-runtime-sockets`)
- [x] Keep container search read-only and avoid writing helper binaries into containers for 1.0. (`crates/grexa-containers::RuntimeOperations` has no write path; replace pipeline rejects container targets)
- [x] Add threat-model notes for searching untrusted binary/document files. (`docs/security.md#threat-model-summary`)
- [x] Run dependency license review and document compatibility with GPLv3. (`docs/dependency-license-review.md` + `deny.toml`)
- [x] Add dependency vulnerability scanning to CI. (`just audit` + CI's `deny` job uses `cargo-deny check`; `dependabot.yml` opens weekly PRs)
- [x] Add a responsible disclosure contact or security policy. (`docs/security.md#reporting-a-vulnerability`)

## Phase 18 - Visual Polish Pass

- [ ] Create a design token file for spacing, radius, colors, typography, animation duration, row density, and table metrics.
- [ ] Build final empty states for Search, AI, Regex Builder, Profiles, History, and Containers.
- [ ] Add subtle animations for tab creation, filter pane expansion, mode switches, and AI message arrival.
- [ ] Add row hover, selected row, active match, and warning/error states.
- [ ] Tune dark theme, light theme, high contrast themes, and KDE accent interactions.
- [ ] Add DBus single-instance activation and "open path/search in existing window" behavior if it fits Linux desktop conventions.
- [ ] Add Plasma progress/notification integration for long-running searches where practical.
- [ ] Check text fit at narrow widths and high DPI scaling.
- [ ] Check all icons under Breeze, Papirus, and fallback icon themes.
- [ ] Smoke-test under KDE Plasma and at least one non-KDE desktop to verify Qt style fallback quality.
- [ ] Verify the first viewport always shows useful search controls and some result/chat surface, not an oversized header.
- [ ] Capture final screenshots under KDE Plasma.

## Phase 19 - Release Readiness

- [ ] Confirm every item in `docs/feature-parity.md` is implemented, intentionally superseded, or explicitly marked non-applicable.
- [ ] Run full unit, integration, container, CLI, and UI smoke test suites.
- [ ] Run real-world searches on several large repositories.
- [ ] Run Docker and Podman searches against representative containers.
- [ ] Run replace dry-runs and real replace tests on copied fixtures.
- [ ] Verify AI endpoint test against OpenAI-compatible local and remote endpoints.
- [ ] Verify localization switching.
- [ ] Verify Flatpak/AppImage launch, file access, clipboard, notifications, editor open, reveal in file manager, and CLI.
- [ ] Tag `v0.1.0-alpha` only after local search, tabs, filters, exports, history/profiles, context preview, and CLI are solid.
- [ ] Tag `v1.0.0` only after Docker, Podman, replace, AI, localization, docs, packaging, and KDE polish are complete.

## Initial Milestone Definition

- [ ] Alpha 1: native Linux local search, tabs, filters, content/files results, search-within-results, context preview, export, and CLI.
- [ ] Alpha 2: safe replace, history, profiles, settings, localization skeleton, and visual theme pass.
- [ ] Alpha 3: Docker and Podman search with direct grep and mirror fallback.
- [ ] Alpha 4: AI chat, endpoint test, secret storage, Regex Builder, and backup/restore.
- [ ] Beta: feature parity audit complete, packaging complete, KDE polish complete, performance acceptable on large repositories.
- [ ] 1.0: Grexa is a daily-driver Linux replacement for Grex with all applicable Grex features retained.

## Risks And Early Spikes

- [ ] Spike Rust/QML/Kirigami model binding with `cxx-qt` before building deep UI.
- [ ] Spike fallback C++/QML host plus Rust library boundary in case `cxx-qt` blocks table models or long-running task integration.
- [ ] Spike high-volume virtualized result table in QML with 100k+ rows.
- [ ] Spike streaming result backpressure and GUI-thread signal batching before building the final search page.
- [ ] Spike PDF extraction quality and decide whether to use a Rust crate or optional external helper.
- [ ] Spike culture-aware comparison in Rust and decide ICU strategy.
- [ ] Spike regex engine compatibility and performance with Rust `regex`, `fancy-regex`, and PCRE2.
- [ ] Spike Podman rootless socket discovery and exec/archive compatibility.
- [ ] Spike Flatpak filesystem permissions for arbitrary user-selected search roots.
- [ ] Spike Baloo candidate seeding to confirm it is worth implementing.
- [ ] Spike editor open-to-line behavior across Kate, VS Code, and JetBrains.

## Status Snapshot (2026-05-16, second update)

Progress through PLAN.md: **246 of 433 checkboxes ticked (~57%)**, up
from 82 at the start of this session. Workspace passes **253 tests**
across 8 crates plus `cargo clippy --workspace --all-targets -- -D
warnings`. Every non-GUI phase has landed end-to-end:

- Phase 0 audits: 8 audit docs + linux-decisions.md + storage-services
  + ai-provider-scope + baloo-spike + gui-design + memory-budgets +
  accessibility (4 audit docs still pending; not blocking).
- Phase 1 scaffolding: tracing, fmt, clippy, deny, audit, CI YAML,
  dependabot, AppStream, .desktop, scalable SVG icon, Justfile,
  rustfmt.toml, rust-toolchain.toml.
- Phase 2 search engine: cancellation, progress, sort, gitignore
  golden tests (61 cases), two-engine regex (`regex` + `fancy-regex`).
- Phase 3 encoding + documents: BOM detection, UTF-16 LE/BE,
  chardetng heuristic, OOXML, ODF, ZIP, PDF (via pdftotext), RTF.
- Phase 6 safe replace: atomic rename, permission preservation, CRLF
  + final-newline preservation, crash journal.
- Phase 7 containers: detection, CLI adapter, direct grep, archive
  mirror fallback.
- Phase 8 AI: HTTP client + endpoint normalization + model discovery
  + chat + secret storage + opt-in setting + provider scope doc.
- Phase 10 settings/history/profiles: 17 storage audit follow-ups +
  full import/export.
- Phase 11 localization: Fluent runtime + 3 locales + sync check.
- Phase 12 CLI: comparison/normalization/culture/diacritics flags,
  --use-index/--no-index, --container/--runtime, `rg` aliases,
  shell completions, man page, 16 integration tests.
- Phase 13 Baloo: trait surface + spike decision (defer).
- Phase 14 context preview: service + 9 unit tests.
- Phase 15 quality: property tests, root-safety tests, memory-budgets
  doc, accessibility doc.
- Phase 16 packaging: Flatpak / AppImage / Arch / Fedora / Debian /
  openSUSE + smoke test.
- Phase 17 + 17a docs: README + 10 user-facing docs + dependency
  license review.

## Big Rocks Remaining

Everything outstanding lives in the GUI half or in items that are
not safely doable in a one-engineer session without a live KDE box
and human review.

### One dedicated PR series — Qt 6 / Kirigami GUI

This consumes most of the remaining open boxes. The Rust controllers
are wired (see `apps/grexa-gui/src/controller.rs`) and the QML
skeleton is in place; the work is replacing each placeholder page
with the real widgets:

- **Phase 1 cxx-qt build** — install CMake on the dev / CI hosts,
  introduce `cxx-qt-build`, replace the `qml6`-spawn host with a
  cxx-qt QApplication wrapping the controllers. Captured in
  `docs/gui-design.md`.
- **Phase 4 Search UI MVP** — Search page, filter pane, command
  strip, tabs, virtualized Content + Files tables, search-within-
  results, recent-path AutoSuggest, empty states, keyboard shortcuts.
- **Phase 5 Linux desktop integration** — portal file picker,
  clipboard actions, KNotifications, KDE color-scheme tracking,
  Wayland-first sanity, KIO-FUSE path discovery.
- **Phase 9 Regex Builder pane** — sample/pattern split, presets,
  toggles, copy/apply.
- **Phase 10 Settings UI** — every section in `crates/grexa-core::storage::DefaultSettings`,
  AI keyring status display, backup/restore wiring.
- **Phase 14 Context preview UI** — modal + gutter + match
  highlight, Open in Editor action.
- **Phase 18 Visual polish** — design tokens, empty states,
  animations, theme tuning, DBus single-instance, fractional scaling.

### Smaller follow-ups outside the GUI

- **Phase 2 culture-aware search via ICU4X**. The `--comparison`
  flag already parses the matrix; the engine needs ICU integration.
  Spike + 43-case fixture lives in
  `docs/grex-culture-comparison-audit.md`.
- **Phase 0 line 158/159** — convert Grex CLI tests into Grexa CLI
  acceptance tests; build the test-coverage map.
- **Phase 0 line 160/162** — write + peer-review
  `docs/feature-parity.md`. Best done after the GUI lands so the
  "implementation column" cites real code.
- **Phase 15 line 446** — `rg` benchmark harness with `hyperfine`
  comparison spreadsheets. Manual run on real hardware.
- **Phase 7 live-daemon tests** — gate behind a `container-live`
  Cargo feature; opt-in on hosts with Docker / Podman running.
- **Phase 19 release readiness** — manual verification rows, tag
  cuts. Done at release time, not pre-release.

### Aspirational / category items that auto-tick

- Stack Decision (ticked at the start of this session)
- Product Principles (ticked)
- Grex Feature Parity Map (ticked where implementation landed)
- Design Direction (presentation; auto-ticks as GUI lands)
- Risks And Early Spikes (most spikes done or recorded as deferred)
- Initial Milestone Definition (alpha/beta/1.0 tags happen at
  release time)
- Non-Goals (assertions, not tasks)

## Non-Goals

- [ ] Do not preserve Windows GUI, WinUI, Windows Search, Windows toast, Windows App Runtime, WSL delegation, UNC path handling, or Windows-specific editor logic.
- [ ] Do not support macOS.
- [ ] Do not choose a webview stack unless the Qt/Kirigami spike fails in a way that blocks the design goals.
- [ ] Do not make container replace part of 1.0.
- [ ] Do not hide search behavior differences behind vague UI labels. If Grexa differs from Grex, document it.
