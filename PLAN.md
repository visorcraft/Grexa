# Grexa Linux Rewrite Plan

Grexa is the Linux-only successor to Grex. The goal is feature parity with Grex where the feature makes sense on Linux, plus first-class KDE Plasma integration, Docker and Podman support, and a cleaner Linux-native architecture.

This plan intentionally treats Grex as the source of truth for behavior, not as source code to mechanically port. Grexa should keep the workflows, filters, safety properties, and power-user affordances, while replacing Windows-specific implementation details with Linux-native equivalents.

## Stack Decision

- [ ] Build Grexa as a Linux-only desktop app with a Rust core and a Qt 6/QML/Kirigami shell.
- [ ] Use Rust for search, replace, container runtime integration, AI HTTP, settings serialization, CLI, tests, and all non-UI business logic.
- [ ] Use Qt 6 + QML + Kirigami for the GUI so the app feels at home on KDE Plasma while still looking polished on other Linux desktops.
- [ ] Use KDE Frameworks where they add real Linux value: Kirigami, QQC2 Desktop Style, Breeze icons, KConfig or XDG-backed config, KNotifications, KIO or portals for file dialogs, KStatusNotifierItem only if a tray feature is later justified, and Baloo integration as an optional accelerator.
- [ ] Start with KDE's Rust-with-Kirigami project shape using CMake to install desktop assets and Cargo to build Rust.
- [ ] Use `cxx-qt` as the preferred Rust/Qt bridge because KDE's Rust-with-Kirigami guidance now points at it for Rust + QML applications.
- [ ] Validate the Rust/QML bridge in an early spike. Preferred path is `cxx-qt` Rust QObjects, controllers, and table models exposed to QML. Fallback path is a thin C++/QML host that communicates with a Rust library or local JSON-RPC sidecar.
- [ ] Keep Electron and Tauri out of the primary architecture. Mailspring is a useful visual reference and is Electron-based, but Grexa should avoid bundling Chromium or depending on WebKitGTK when Qt/Kirigami gives better Plasma fit, native menus, KDE dialogs, native theming, and lower idle overhead.
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

- [ ] Preserve Grex's core promise: fast, precise grep-style search with tabs, previews, filters, export, safe replace, history, profiles, and optional AI assistance.
- [ ] Treat Linux as the only supported platform. Do not carry Windows abstractions, WSL branches, WinUI patterns, Windows Search code, Windows toast behavior, or Windows path assumptions into Grexa.
- [ ] Make KDE Plasma the first-class desktop target without hard-breaking other Linux desktop environments.
- [ ] Prefer native Linux conventions: XDG paths, Freedesktop desktop entries, AppStream metadata, portals where sandboxing matters, standard clipboard and notification interfaces, `xdg-open`, and `org.freedesktop.FileManager1`.
- [ ] Keep the UI dense, calm, and tool-like. Grexa is a daily developer utility, not a landing page.
- [ ] Favor streaming and cancellation over large in-memory batches.
- [ ] Make every expensive operation cancellable.
- [ ] Define backpressure between the Rust core and QML UI so search can stream large result sets without unbounded memory growth.
- [ ] Require search/replace cancellation latency to remain perceptibly immediate under full result-stream load.
- [ ] Keep user data inspectable and portable.
- [ ] Keep container search read-only unless a future feature deliberately designs writable container replace.
- [ ] Make feature parity measurable through golden fixtures copied or adapted from Grex tests.

## Grex Feature Parity Map

- [ ] Preserve tabbed searches with one isolated state object per tab.
- [ ] Preserve Text and Regex search modes.
- [ ] Preserve Content mode with per-line hits: name, line, column, snippet, relative path, full path, match count, preview segments.
- [ ] Preserve Files mode with per-file aggregation: name, size, match count, first match, preview matches, full path, relative path, extension, detected encoding, modified time.
- [ ] Preserve filters: respect `.gitignore`, case sensitivity, include system files, include subfolders, include hidden items, include binary/searchable documents, include symbolic links, match file names, exclude dirs, size limit type, size value, and size unit.
- [ ] Preserve text comparison settings: ordinal/current culture/invariant culture, Unicode normalization, diacritic sensitivity, and selected culture.
- [ ] Preserve match file syntax: glob patterns separated by `|` or `;`, with `-pattern` exclusions.
- [ ] Preserve exclude dirs syntax: comma/semicolon separated names and regex mode when regex markers are used.
- [ ] Preserve system path auto-exclusions: `.git`, `vendor`, `node_modules`, `storage/framework`, `bin`, `obj`, `sys`, `proc`, and `dev`, with Linux-specific pseudo filesystem guards added for root searches.
- [ ] Replace Windows Search integration with optional Baloo candidate seeding on KDE. Always verify candidate files with Grexa's own search engine before showing results.
- [ ] Drop WSL-specific paths and behavior. Native Linux paths are the default. Mounted Windows drives, SMB, NFS, SSHFS, KIO FUSE, and external drives are just Linux paths after mounting.
- [ ] Preserve Docker container search and add Podman parity.
- [ ] Preserve direct in-container grep as the preferred container strategy.
- [ ] Preserve container mirror fallback when grep is unavailable.
- [ ] Preserve container path display regardless of direct or mirrored search method.
- [ ] Preserve replace disabled for container targets.
- [ ] Preserve context preview with configurable before/after line counts.
- [ ] Preserve search-within-results with text and regex filters.
- [ ] Preserve safe replace workflow with confirmation, cancellation, and Files mode results.
- [ ] Preserve export to CSV, JSON, and clipboard.
- [x] Preserve search history with a cap of 20 by default.
- [ ] Preserve recent path suggestions with a cap of 20 by default.
- [ ] Preserve recent path type-ahead filtering and per-entry removal.
- [x] Preserve named search profiles with full filter snapshots.
- [ ] Preserve Regex Builder with sample text, live matches, presets, breakdown, case-insensitive, multiline, and global toggles.
- [ ] Preserve AI Search chat with OpenAI-compatible endpoint, optional API key, optional model, `/v1/models` discovery/test, context-rich prompt, follow-up conversation, and empty/error states.
- [ ] Preserve settings backup, import, restore defaults, and instant-save behavior.
- [ ] Preserve localization coverage and runtime language switching.
- [ ] Preserve localized tooltips and accessible names for command buttons, filters, settings, AI controls, and result actions.
- [ ] Preserve keyboard shortcuts: Enter to search, Enter to replace from replacement input, Space for preview, Escape to close preview/dialogs, F1 for About, double-click to open result.
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
- [ ] Add `crates/grexa-core` for search, replace, filters, encodings, gitignore, exports, settings models, and shared DTOs.
- [ ] Add `crates/grexa-containers` for Docker and Podman runtime abstraction.
- [ ] Add `crates/grexa-ai` for OpenAI-compatible endpoints.
- [ ] Add `crates/grexa-cli` for the headless CLI.
- [ ] Add `apps/grexa-gui` for Qt/Kirigami app bootstrap, QML, resources, desktop file, icons, and UI controllers.
- [ ] Add `tests/fixtures` for search behavior fixtures ported from Grex tests.
- [ ] Add `docs/` for user docs, feature docs, architecture, packaging, and migration notes.
- [ ] Add `packaging/flatpak`, `packaging/appimage`, and distro packaging notes.
- [ ] Add `scripts/` for localization extraction, icon generation, fixture generation, and smoke tests.

## Phase 0 - Behavior Audit And Specification

- [x] Inventory every Grex source, resource, script, doc, and test file into `docs/grex-audit-inventory.md` before implementation starts.
- [x] Audit Grex `Services/SearchService.cs` and document exact search semantics before porting.
- [x] Audit Grex `Services/DockerSearchService.cs` and document direct grep, fallback mirror, path translation, filters, and cleanup.
- [x] Audit Grex `Services/WindowsSearchIntegration.cs` and document the Linux replacement as optional Baloo candidate seeding.
- [x] Audit Grex `Services/WindowsSubsystemLinuxService.cs` and document why WSL support is non-applicable for Grexa.
- [x] Audit Grex `ViewModels/TabViewModel.cs` and document tab state, search/replace lifecycle, status strings, cancellation, sorting, and result filtering.
- [ ] Audit Grex `ViewModels/MainViewModel.cs` and document tab lifecycle behavior.
- [ ] Audit Grex `Controls/SearchTabContent.xaml.cs` and document UI workflows, shortcuts, context menus, export, profiles, history, and AI mode transitions.
- [ ] Audit Grex `Controls/SearchTabContent.xaml` and document layout, controls, result columns, and visual states to preserve or replace.
- [ ] Audit Grex `Controls/SettingsView.xaml` and `Controls/SettingsView.xaml.cs` and document every settings section and Linux replacement.
- [ ] Audit Grex `Controls/RegexBuilderView.xaml` and `Controls/RegexBuilderView.xaml.cs` and document Regex Builder behavior.
- [ ] Audit Grex `Controls/ContextPreviewDialog.xaml` and `Services/ContextPreviewService.cs` and document preview behavior.
- [ ] Audit Grex `Controls/AboutView.xaml` and `Controls/AboutView.xaml.cs` for About page content and localization behavior.
- [ ] Audit Grex `Services/AiSearchService.cs` and document endpoint normalization, model discovery, response parsing, and error extraction.
- [ ] Audit Grex `Services/SettingsService.cs`, `RecentPathsService.cs`, `RecentSearchesService.cs`, and `SearchProfilesService.cs` and define Grexa's XDG data/config equivalents.
- [ ] Audit Grex `Services/RecentSearchesService.cs`, `Services/ExportService.cs`, `Services/ContextMenuService.cs`, `Services/NotificationService.cs`, `Services/LocalizationService.cs`, and `Services/LocalizedToolTipRegistry.cs`.
- [ ] Audit Grex `EncodingDetectionService.cs` and list required encodings and confidence behavior.
- [ ] Audit Grex `GitIgnoreService.cs` tests and codify edge cases for root-relative patterns, directory-only patterns, negations, `**`, brackets, and case behavior.
- [ ] Audit Grex culture-aware comparison behavior and codify string comparison, Unicode normalization, diacritic, and culture cases in fixtures.
- [ ] Audit Grex status text behavior, including filtered result summaries and elapsed-time pluralization.
- [ ] Audit Grex model classes and ensure every field is mapped, renamed, removed as non-applicable, or replaced by a Linux-specific field.
- [ ] Audit Grex `Strings/*/Resources.resw` and build a migration matrix for kept, removed, renamed, and new Linux strings.
- [ ] Audit Grex scripts and decide which localization or asset scripts should be ported.
- [ ] Audit Grex CLI tests and convert them into Grexa CLI acceptance tests.
- [ ] Audit Grex unit, integration, and UI tests and create a coverage map showing which Grexa test will preserve each behavior.
- [ ] Write `docs/feature-parity.md` with each Grex feature mapped to Grexa implementation, replacement, or explicit non-applicability.
- [ ] Write `docs/linux-decisions.md` explaining the removal of WSL, UNC, Windows Search, Windows toasts, Windows App Runtime, and WinUI-specific patterns.
- [ ] Peer-review `docs/feature-parity.md` against Grex docs and source before any phase is considered complete.

## Phase 1 - Project Scaffold And Tooling

- [ ] Initialize Cargo workspace and CMake install project.
- [ ] Create the Qt/Kirigami GUI skeleton with `ApplicationWindow`, desktop file, app id, icon resources, and a placeholder Search page.
- [ ] Add a minimal `cxx-qt` QObject and QML binding smoke test before any large UI work.
- [ ] Add a `cxx-qt` table-model spike for result rows before committing to the final table implementation.
- [ ] Pin the async model early: Rust worker tasks may use `tokio` or scoped threads, but all QObject/QML mutations and model signals must marshal onto the GUI thread.
- [ ] Implement a streaming append-only result model spike with batched row insertion signals, measured rows/sec throughput, bounded channel backpressure, and cancellation-latency metrics.
- [x] Add `justfile` or `Makefile` commands for build, test, lint, format, run GUI, run CLI, and package.
- [ ] Add contributor workflow support: `cargo watch`, QML live reload where feasible, and one-command bootstrap notes for Arch, Fedora, Debian/Ubuntu, and openSUSE.
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

- [ ] Implement `SearchOptions`, `SearchResult`, `FileSearchResult`, `SearchSummary`, and cancellation types in Rust.
- [x] Implement recursive Linux file walking with streaming results.
- [ ] Use the `ignore` crate or equivalent to handle `.gitignore`, `.ignore`, global git excludes if desired, hidden files, symlinks, and parallel traversal.
- [ ] Decide and test bind-mount, overlayfs, btrfs subvolume, and same-filesystem traversal behavior.
- [ ] Preserve Grex's `.gitignore` behavior through golden tests, especially root-relative and negated patterns.
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
- [ ] Decide whether Rust `regex` is sufficient or whether a PCRE2/fancy-regex mode is needed to preserve .NET regex features that users may expect.
- [ ] Prefer an explicit two-engine strategy unless the spike disproves it: a fast Rust `regex` path for simple patterns and an extended compatibility path using `fancy-regex` or PCRE2 for .NET-like constructs.
- [ ] Add regex compatibility fixtures for lookaround, backreferences, named captures, conditional constructs, Unicode `\d`/`\w` semantics, multiline/global behavior, invalid patterns, and saved Grex profile patterns.
- [ ] Add import-time warnings or migration notes for Grex Regex patterns unsupported by Grexa's chosen engine.
- [x] Implement line, column, match count, snippet, and preview segment calculation.
- [ ] Implement result sorting fields equivalent to Grex.
- [x] Implement result aggregation into Files mode without rerunning search.
- [ ] Add progress events: files scanned, bytes scanned, matches found, skipped files, elapsed time.
- [ ] Add cancellation checkpoints throughout traversal and file scanning.
- [ ] Define cancellation result policy: whether partial results remain visible, how memory is released, and how cancelled/partial status is reported.
- [ ] Add performance baselines against `ripgrep` on large fixture trees.

## Phase 3 - Encoding And Searchable Document Support

- [ ] Implement BOM detection for UTF-8, UTF-16 LE/BE, UTF-32 LE/BE.
- [ ] Implement heuristic/statistical detection for the 30+ encodings listed in Grex docs.
- [ ] Evaluate `encoding_rs`, `chardetng`, and ICU-backed alternatives for coverage gaps.
- [ ] Preserve Grex labels for detected encodings where practical.
- [ ] Implement plain text decoding with replacement/error policy documented.
- [ ] Optimize encoding detection: fast-path UTF-8/BOM files, avoid expensive heuristic detection on every file when unnecessary, and consider user/default encoding overrides.
- [ ] Implement searchable Office Open XML extraction for `.docx`, `.xlsx`, and `.pptx`.
- [ ] Implement searchable OpenDocument extraction for `.odt`, `.ods`, and `.odp`.
- [ ] Implement ZIP search for file names and text/XML contents.
- [ ] Implement PDF text extraction with a Rust library or a controlled optional helper, and document limitations.
- [ ] Evaluate optional Poppler/`pdftotext` integration for better PDF quality, with a pure-Rust fallback if available.
- [ ] Explicitly document that scanned/OCR-only PDFs, encrypted PDFs, malformed PDFs, and complex font/CMap cases may be unsupported unless an OCR/helper feature is later added.
- [ ] Implement RTF text extraction.
- [ ] Preserve binary skip lists for images, audio, video, executables, legacy Office, archives, caches, locks, packs, indexes, and unsupported binaries.
- [ ] Add fixtures for every supported searchable binary/document type.
- [ ] Add tests for files with invalid bytes, mixed encodings, huge lines, and null bytes.

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
- [ ] Add support for mounted SMB, NFS, SSHFS, external drives, and KIO FUSE paths as normal Linux paths.
- [ ] Document that Grexa searches actual mounted paths, not abstract KIO URLs, unless a future KIO worker bridge is deliberately implemented.
- [ ] Detect unsupported `smb://`, `fish://`, `mtp://`, and other abstract KIO/GVFS URLs and show a clear mount-or-browse-real-path message.
- [ ] Count and surface skipped files/directories for disappearing mounts, permission errors, stale network filesystems, and transient I/O failures.
- [ ] Avoid aggressive canonicalization that breaks KIO-FUSE, GVFS, bind mounts, symlinks, or user-intended path display.
- [ ] Add a "Reveal in File Manager" action using `org.freedesktop.FileManager1.ShowItems`, with `xdg-open` fallback.
- [ ] Add "Open in Editor" using configured editor templates.
- [ ] Ship editor presets for Kate/KWrite, VS Code, VSCodium, JetBrains IDEs, Sublime Text, GNOME Text Editor, Neovim terminal wrapper, and default `xdg-open`.
- [ ] Add clipboard actions for full path, relative path, file name, line content, and container path.
- [ ] Add KNotifications/Freedesktop notifications for completed long searches, errors, and endpoint tests.
- [ ] Add a notification diagnostic panel only if Linux notification failures need user-facing diagnostics.
- [ ] Add KDE color-scheme integration and system accent support.
- [ ] Add high contrast and reduced-motion handling.
- [ ] Add Wayland-first behavior and test under KDE Plasma Wayland.
- [ ] Avoid custom window decoration unless the first UI spike proves it is stable under KDE Wayland.

## Phase 6 - Safe Replace

- [ ] Implement replace preview/search pass that reuses the exact search filters.
- [ ] Implement confirmation dialog with file count, match count, irreversible warning, and cancellation.
- [ ] Switch to Files mode after replace results, matching Grex behavior.
- [ ] Implement text replacement.
- [ ] Implement regex replacement with capture group support.
- [ ] Preserve file permissions, ownership where possible, modified timestamps policy, and line endings.
- [ ] Use safe temporary file writes and atomic rename where the filesystem supports it.
- [ ] Preserve or explicitly warn about hardlinks, ACLs, xattrs, SELinux/AppArmor labels, immutable/append-only attributes, sparse files, ownership, permissions, and timestamps.
- [ ] Ensure temporary files are created on the same filesystem as the target so atomic rename remains valid.
- [ ] Add a crash-recovery/journal design for replace operations so users can understand which files were already modified after a crash or cancellation.
- [ ] Preserve mixed line endings and final-newline behavior where possible.
- [ ] Explicitly disallow replace inside ZIP/docx/xlsx/pptx/odt/ods/odp/pdf/rtf extracted document contents for 1.0 unless a separate archive-edit design is added.
- [ ] Add cancellation support before each file write and between large chunks.
- [ ] Add clear partial-replace reporting if cancellation occurs after some files were written.
- [ ] Keep no-undo behavior explicit. Consider optional backup files only as a later enhancement, not as a hidden behavior change.
- [ ] Disable replace for container targets.
- [ ] Add replace tests for permissions, symlinks, encodings, regex groups, binary skip rules, and cancellation.

## Phase 7 - Docker And Podman Support

- [ ] Design a `ContainerRuntime` trait with Docker and Podman implementations.
- [ ] Detect Docker via `$DOCKER_HOST`, `/var/run/docker.sock`, and docker CLI fallback.
- [ ] Detect Docker Desktop for Linux socket variants and document unsupported daemon setups.
- [ ] Detect rootless Podman via `$XDG_RUNTIME_DIR/podman/podman.sock`.
- [ ] Detect rootful Podman via `/run/podman/podman.sock` when accessible.
- [ ] Detect Podman CLI fallback when the socket service is not running.
- [ ] Add UI target dropdown with Local Files, Docker containers, and Podman containers grouped by runtime.
- [ ] Add runtime badges so users can distinguish Docker, Podman rootless, and Podman rootful.
- [ ] Implement container listing for both Docker and Podman.
- [ ] Implement grep availability probing per container and cache results by runtime/container id.
- [ ] Implement direct grep search using container exec with `find -print0 | xargs -0 -P <n> grep`.
- [ ] Build container exec commands with argv arrays where APIs permit, avoiding shell quoting bugs for paths/patterns containing spaces, quotes, colons, glob characters, or newlines.
- [ ] Add BusyBox/Alpine grep/find fallbacks where GNU `grep`, GNU `find`, or `xargs -P` are unavailable.
- [ ] Handle scratch/distroless containers by going directly to mirror/archive fallback with clear UI status.
- [ ] Account for Docker vs Podman exec output framing differences and stderr/stdout multiplexing behavior.
- [ ] Apply hidden, binary, system path, match files, exclude dirs, subfolder, and gitignore filters inside the container where possible.
- [ ] Implement `.gitignore` handling inside containers by collecting relevant patterns or by running a helper command.
- [ ] Parse grep output robustly when file names or content contain colons.
- [ ] Count multiple matches per line and compute column numbers.
- [ ] Add mirror fallback using Docker/Podman archive APIs or CLI `cp`/`tar` fallback.
- [ ] Test archive fallback with rootless UID/GID mappings, read-only containers, sparse files, special files, broken symlinks, and permission-denied files.
- [ ] Use Linux temporary/cache path `$XDG_CACHE_HOME/grexa/container-mirrors`.
- [ ] Preserve symlink handling policy during mirror fallback.
- [ ] Prune expired mirrors after search and on startup.
- [ ] Display container paths in results even when the mirror fallback is used.
- [ ] Add container-specific context menu actions: Copy Container Path, Copy File Name, Copy Runtime Command.
- [ ] Add tests against Docker Engine.
- [ ] Add tests against rootless Podman.
- [ ] Add tests against containers with grep and containers without grep.
- [ ] Add tests for Alpine/minimal images, paths with spaces, symlinks, hidden files, and `.gitignore`.

## Phase 8 - AI Search Chat

- [ ] Implement AI settings: endpoint URL, optional API key, optional model.
- [ ] Store API keys in KWallet or Secret Service where available, with a documented fallback if secret storage is unavailable.
- [ ] Make AI chat explicitly opt-in and show what local context is sent before the first request.
- [ ] Gate AI code behind a Cargo feature or build option if feasible so privacy-sensitive distributions can build without AI integration.
- [ ] Implement endpoint normalization for bare hosts, `/v1`, `/v1/chat/completions`, and trailing slash variants.
- [ ] Implement `/v1/models` discovery and Settings "Test Endpoint".
- [ ] Implement OpenAI-compatible chat completions requests.
- [ ] Preserve Grex response parsing: `choices[].message.content`, `choices[].text`, `output_text`, and structured error messages.
- [ ] Build context from path, query, search mode, result mode, and active filters.
- [ ] Add Linux-specific context suggestions for hidden files, symlinks, mounted paths, containers, Baloo, and pseudo filesystems.
- [ ] Implement in-tab conversation state.
- [ ] Hide result grids and search-within-results while AI mode is active, matching Grex.
- [ ] Add AI empty state, loading state, send disabled state, cancellation, and retry.
- [ ] Add tests with mock HTTP endpoints for models, chat completions, errors, malformed JSON, empty responses, and auth headers.
- [ ] State provider scope clearly: OpenAI-compatible APIs only; Ollama or other local providers are supported through their OpenAI-compatible shim when available.

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

- [ ] Choose localization pipeline: KDE KI18n/gettext if the GUI bridge supports it cleanly, otherwise Qt `.ts` plus Rust Fluent for core/CLI.
- [ ] Convert Grex `Strings/*/Resources.resw` into Grexa's selected localization format.
- [ ] Convert placeholders and pluralization rules explicitly, including `.resw` `{0}` style placeholders to the selected target format.
- [ ] Flag strings whose semantics changed from Windows to Linux for human retranslation instead of blindly reusing stale translations.
- [ ] Decide whether 1.0 ships all inherited locales or a smaller high-quality locale set with the remaining catalogs marked incomplete.
- [ ] Add CI checks that source strings, translation catalogs, placeholders, plural forms, and fallback keys remain in sync.
- [ ] Preserve all existing user-facing strings that still apply.
- [ ] Remove Windows-specific strings and add Linux/KDE/Podman replacements.
- [ ] Add extraction/update scripts so new strings cannot bypass localization.
- [ ] Add runtime language switching.
- [ ] Add fallback-to-English behavior.
- [ ] Add tests for missing keys, formatted strings, language switch propagation, and RTL layout where feasible.
- [ ] Verify About, Settings, Regex Builder, Search, AI, tooltips, context menus, and dialogs are localized.

## Phase 12 - CLI

- [x] Implement `grexa-cli <path> <term> [options]`.
- [ ] Preserve Grex CLI options: `--regex`, `--case-sensitive`, `--gitignore`, `--include-hidden`, `--include-binary`, `--include-system`, `--no-subfolders`, `--include-symlinks`, `--match-files`, `--exclude-dirs`, `--size-limit`, `--size-unit`, `--size-type`, `--format`, `--count`, `--files-only`, and `--quiet`.
- [ ] Decide whether to expose Grex's advanced CLI option model fields for string comparison, Unicode normalization, diacritic-insensitive search, and culture; document the decision and test whichever behavior is chosen.
- [ ] Add Linux-specific CLI options for Baloo seeding only if useful: `--use-index` and `--no-index`.
- [ ] Add container CLI options: `--runtime docker|podman|auto`, `--container <name-or-id>`, and `--container-path <path>` if this can be done without making the CLI confusing.
- [ ] Decide which familiar `grep`/`rg` aliases to support, such as `-n`, `--hidden`, `--no-ignore`, `-g`, and explicit line-number controls, while avoiding conflicts with Grex's existing option meanings.
- [x] Preserve exit codes: 0 matches found, 1 no matches, 2 error.
- [x] Preserve grep-compatible text output.
- [x] Preserve pretty JSON and escaped CSV output.
- [ ] Add shell completion generation for Bash, Zsh, and Fish.
- [ ] Add man page generation.
- [ ] Add CLI integration tests for local search, errors, output formats, count, files-only, quiet, and container search.

## Phase 13 - Baloo Index Acceleration Spike

- [ ] Treat Baloo as an optional candidate source, never the source of truth.
- [ ] Timebox Baloo to a short spike with an explicit keep/defer/drop decision before implementation proceeds.
- [ ] Detect whether Baloo is available and indexing is enabled.
- [ ] Detect whether the path appears indexed before using Baloo.
- [ ] Query Baloo for candidate files for plain-text searches.
- [ ] Fall back silently to Grexa's walker when Baloo is unavailable, disabled, stale, or unsupported for the path.
- [ ] Verify every candidate file with Grexa's own matching pipeline.
- [ ] Disable Baloo for regex searches unless a future implementation proves it can safely prefilter.
- [ ] Add UI text that explains "Use KDE file index" without promising completeness.
- [ ] Add diagnostics showing whether a search used the index or the custom walker.
- [ ] Measure whether Baloo accelerates real source-code searches, not only home-folder document searches; defer from 1.0 if the benefit is weak.
- [ ] Add tests with a mocked Baloo adapter so CI does not depend on a live indexer.

## Phase 14 - Context Preview And File Actions

- [ ] Implement context preview service for local Linux files.
- [ ] Implement context preview for mirrored container results.
- [ ] Implement direct container context preview if mirror is unavailable and runtime exec can read the file.
- [ ] Preserve before/after line count settings from 1 to 20.
- [ ] Highlight matched line and matched substring.
- [ ] Show line numbers in a gutter.
- [ ] Add Open in Editor action from preview.
- [ ] Add Escape-to-close behavior.
- [ ] Add context menu preview action.
- [ ] Add tests for UTF-8, UTF-16, large files, first/last line edge cases, missing files, permission denied, and container results.

## Phase 15 - Quality, Performance, And Reliability

- [ ] Build a Grex-compatible golden fixture suite for local search.
- [ ] Add a peer-review checklist requiring another pass over every phase before each milestone can close.
- [ ] Add property tests for path normalization, glob matching, exclude dirs, size limits, and snippet boundaries.
- [ ] Add stress tests for large directories, large files, huge lines, many matches, many tabs, and cancellation.
- [ ] Add benchmarks against `rg` for common cases.
- [ ] Add memory usage budgets for million-result scans and document expected behavior.
- [ ] Add UI responsiveness tests using QML test tools or screenshot-driven smoke tests.
- [ ] Add Qt/QML smoke tests under `QT_QPA_PLATFORM=offscreen` for CI.
- [ ] Keep most controller/search behavior testable as pure Rust without QML.
- [ ] Add accessibility pass: keyboard navigation, focus order, screen reader labels, contrast, high contrast themes, and reduced motion.
- [ ] Add AT-SPI/accessibility roles and names for custom result tables, row actions, command buttons, filter controls, and dialogs.
- [ ] Add Wayland and X11 smoke tests under KDE where CI/container support allows.
- [ ] Add fractional scaling visual checks at 125%, 150%, 200%, and mixed-DPI monitor setups.
- [ ] Add root search safety tests for `/proc`, `/sys`, `/dev`, `/run`, and permission-denied directories.
- [ ] Add container runtime matrix tests for Docker, Podman rootless, and Podman rootful where available.
- [ ] Add crash-safe log capture and error reports in `$XDG_STATE_HOME/grexa`.

## Phase 16 - Packaging And Distribution

- [ ] Ship Flatpak as the primary binary distribution path.
- [ ] Define Flatpak permissions narrowly: filesystem access should be explicit and user-driven where possible, with clear guidance for searching arbitrary paths.
- [ ] Document Flatpak limitations for Docker/Podman socket access, host filesystem access, editor launching, and file-manager reveal actions.
- [ ] Provide non-Flatpak packages for users who need unrestricted container sockets and filesystem traversal.
- [ ] Provide AppImage as a secondary portable option if Qt/KDE dependencies package cleanly.
- [ ] Provide distro packaging recipes for Arch PKGBUILD, Fedora RPM, Debian package, and openSUSE spec after the app stabilizes.
- [ ] Install `.desktop`, AppStream metadata, icons, MIME actions if any, and man pages.
- [ ] Include `grexa-cli` in packages.
- [ ] Add update and release workflow documentation.
- [ ] Add reproducible release notes template.
- [ ] Add smoke test script that runs after packaging.

## Phase 17 - Documentation

- [ ] Write `README.md` for Grexa with Linux requirements and screenshots.
- [ ] Write `docs/features.md` matching the Grex feature deep dive but Linux-specific.
- [ ] Write `docs/usage.md` with KDE/Linux workflows, Docker, Podman, Baloo, mounted shares, replace, AI, and CLI.
- [ ] Write `docs/architecture.md` explaining Rust core, Qt/Kirigami shell, runtime adapters, and data paths.
- [ ] Write `docs/build-and-test.md` with distro dependency commands.
- [ ] Write `docs/reference.md` with settings schema, CLI options, data paths, keyboard shortcuts, binary formats, encoding support, and limitations.
- [ ] Write `docs/translations.md` for the new localization pipeline.
- [ ] Write migration notes for Grex users moving settings/history/profiles where sensible.
- [ ] Add screenshots after the UI visual pass.

## Phase 17a - Security, Privacy, And Licensing

- [ ] Write `docs/security.md` covering local file access, replace risks, container access, logs, AI context sharing, secret storage, and crash diagnostics.
- [ ] State telemetry policy explicitly, preferably zero telemetry and opt-in diagnostics only.
- [ ] Keep API keys out of logs, exports, screenshots, diagnostics, and settings backups unless the user explicitly includes secrets.
- [ ] Never silently fall back to plaintext API key storage; require explicit user opt-in if KWallet/Secret Service is unavailable.
- [ ] Redact paths in optional diagnostics when privacy mode is enabled.
- [ ] Treat container runtime sockets as privileged access and explain the risk in Settings.
- [ ] Keep container search read-only and avoid writing helper binaries into containers for 1.0.
- [ ] Add threat-model notes for searching untrusted binary/document files.
- [ ] Run dependency license review and document compatibility with GPLv3.
- [ ] Add dependency vulnerability scanning to CI.
- [ ] Add a responsible disclosure contact or security policy.

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

## Non-Goals

- [ ] Do not preserve Windows GUI, WinUI, Windows Search, Windows toast, Windows App Runtime, WSL delegation, UNC path handling, or Windows-specific editor logic.
- [ ] Do not support macOS.
- [ ] Do not choose a webview stack unless the Qt/Kirigami spike fails in a way that blocks the design goals.
- [ ] Do not make container replace part of 1.0.
- [ ] Do not hide search behavior differences behind vague UI labels. If Grexa differs from Grex, document it.
