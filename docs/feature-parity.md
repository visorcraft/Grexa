# Grex → Grexa Feature Parity

The canonical answer to "is feature X from Grex present in Grexa?"
Every row is one Grex behavior; the *Implementation* column points at
the code, doc, or audit that establishes parity, an intentional
divergence, or an explicit non-applicability.

Last refreshed: v1.0.0.

Status legend:

- ✅ **Done** — shipped in Grexa.
- 🟡 **Partial** — present at the contract level; some refinement
  scheduled for a later release.
- ⏸ **Deferred** — explicit deferral with a recorded reason.
- 🟥 **N/A** — Windows-only or otherwise out of scope for Linux.

## Search

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Tabbed searches, per-tab state | ✅ | Per-tab snapshots in `qobjects/search.rs::TabSnapshot`; QML tab strip in `SearchPage.qml` with horizontal-scroll overflow (v0.3). |
| Text + Regex modes | ✅ | `crates/grexa-core/src/search.rs` + `pattern.rs` (two-engine cascade). |
| Content mode with line / column / snippet | ✅ | `SearchResult`. |
| Files mode aggregation | ✅ | `FileSearchResult` + `aggregate_file_results`. |
| Filter set (gitignore, case, hidden, system, …) | ✅ | Every `SearchOptions` field; `gitignore_parity.rs` pins 61 cases. |
| Match-files glob syntax (`|` / `;` / `-prefix`) | ✅ | `FileNameFilter` (`crates/grexa-core/src/search.rs`). |
| Exclude-dirs name list or regex | ✅ | `ExcludeDirFilter`. |
| System-path auto-exclusions | ✅ | `SYSTEM_DIRS` + `is_system_path`; `tests/root_safety.rs`. |
| Pseudo-FS guards (`/proc`, `/sys`, `/dev`, `/run`) | ✅ | Same auto-exclusions; user can override with `--include-system`. |
| Culture-aware comparison (ordinal / culture / invariant / normalization / diacritic) | 🟡 | Modes wired through `SearchOptions`; ICU4X-backed casing ships v1.1 (`docs/grex-culture-comparison-audit.md`). |
| Streaming + cancellation | ✅ | `CancelToken` + `ProgressEvent`. |
| Sort + stable tie-breakers | ✅ | `crates/grexa-core/src/sort.rs`. |
| Search-within-results | ✅ | `SearchController` within-filter state + QML tab snapshots. |
| Result export (CSV / JSON / clipboard) | ✅ | CLI emits CSV/JSON; GUI Export menu writes CSV/JSON/Markdown; result-row context menu copies path / filename / line content / path:line. |

## Replace

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Safe atomic replace | ✅ | `crates/grexa-core/src/replace.rs::replace_with` + `tempfile::persist`. |
| Regex captures (`$1`, `$name`) | ✅ | `PatternEngine::replace_all`. |
| Permission preservation | ✅ | `restore_permissions`. |
| CRLF + final-newline preservation | ✅ | Whole-buffer substitution; two unit tests. |
| Encoding round-trip | ✅ | `encode_for_writeback` covers UTF-8 / UTF-8 BOM / UTF-16 / chardetng-detected encodings. |
| Confirmation dialog | ✅ | `SearchPage.qml::replaceDialog` (gated on `replace_confirm` setting); Enter commits, Escape cancels. |
| Switch to Files mode after replace | ✅ | `Workspace::run_replace_blocking` flips `result_mode` on completion. |
| Crash-recovery journal | ✅ | `replace-journal.json` under `$XDG_STATE_HOME/grexa/`. |
| No-undo, no backup files | ✅ | Documented in `docs/features.md`. |
| Replace disabled for containers / archives | ✅ | `RuntimeOperations` has no write path; document extractor only emits text. |

## Encoding

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| BOM detection (UTF-8 / 16 / 32) | ✅ | `crates/grexa-core/src/encoding.rs::detect_from_bytes`. |
| Strict UTF-8 fast path | ✅ | `read_text`. |
| Legacy 8-bit + multibyte heuristic | ✅ | `chardetng`. |
| Grex-compatible labels | ✅ | `DetectedEncoding::label`. |
| Lossy fallback for malformed bytes | ✅ | `encoding_rs::Encoding::decode` plus `String::from_utf8_lossy`. |

## Searchable documents

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| OOXML extraction | ✅ | `documents.rs::extract_ooxml` for docx/xlsx/pptx. |
| ODF extraction | ✅ | Same path for odt/ods/odp. |
| ZIP search | ✅ | `extract_zip` (names + textual entries). |
| RTF text extraction | ✅ | `extract_rtf`. |
| PDF extraction via Poppler | ✅ | `extract_pdf` shells `pdftotext`. |
| Binary skip list | ✅ | `BINARY_EXTENSIONS` in `search.rs`. |

## Container search

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Docker runtime detection | ✅ | `detect_runtimes::detect_docker`. |
| Podman rootless detection | ✅ | `detect_podman_rootless`. |
| Podman rootful detection | ✅ | `detect_podman_rootful`. |
| List containers (`ps --format json`) | ✅ | `CliRuntime::list_containers` (Docker + Podman shapes). |
| `grep` availability probe | ✅ | `has_grep`. |
| Direct exec grep with argv array | ✅ | `direct_grep` — quoting-safe. |
| BusyBox / distroless fallback | ✅ | `grep -rnH` is portable across distros; missing-grep falls through to mirror. |
| Archive mirror fallback | ✅ | `archive_path` + `search_container` mirror branch. |
| Container path display | ✅ | `rewrite_path` keeps the container path intact. |
| Container context preview | ✅ | `grexa_containers::container_context_preview`. |
| Replace disabled for containers | ✅ | No write path on `RuntimeOperations`. |
| Live Docker test suite | ⏸ | Gated behind the `container-live` Cargo feature; runs in CI when a daemon is present. |
| Live Podman test suite | ⏸ | Same. |

## AI Search Chat

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| OpenAI-compatible endpoint | ✅ | `crates/grexa-ai/src/lib.rs::AiSearchClient`. |
| Endpoint URL normalization | ✅ | `normalize_endpoint_base`. |
| Model auto-discovery (`/v1/models`) | ✅ | `AiSearchClient::discover_model`. |
| Chat completions request | ✅ | `send_chat`. |
| Response parsing (choices/message/content, choices/text, output_text) | ✅ | `extract_assistant_content`. |
| Error extraction | ✅ | `extract_error_message`. |
| Context prompt builder | ✅ | `build_context_prompt`. |
| Linux-aware filter hints | ✅ | `linux_suggestions_for`. |
| Secret-Service-backed API key | ✅ | `secret.rs` via `keyring`. |
| Opt-in setting | ✅ | `DefaultSettings.ai_search_enabled`. |
| Provider scope doc | ✅ | `docs/ai-provider-scope.md`. |
| In-tab conversation state | ✅ | `AiChatPanel.qml` with turn-count header + Clear button (v0.3). |

## Context preview

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Configurable before/after (1–20) | ✅ | `crates/grexa-core/src/preview.rs::context_preview`. |
| 1-based line numbers | ✅ | Same module. |
| Encoding-aware reader | ✅ | Inherits `encoding::read_text`. |
| Match-line index | ✅ | `match_line_index` field. |
| Edge cases (empty / OOR / missing / perms) | ✅ | 9 unit tests. |
| Container-backed preview | ✅ | `grexa_containers::container_context_preview`. |
| Gutter + match highlight | ✅ | `ContextPreviewDialog.qml` renders both. |

## Settings, history, profiles, recent paths

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| `DefaultSettings` round-trip | ✅ | `crates/grexa-core/src/storage.rs`. |
| Recent paths cap 20 + dedupe + filter | ✅ | `RecentPathStore`. |
| Search history cap 20 + 7-field dedupe key (matching Grex byte-for-byte) | ✅ | `SearchHistoryStore` + `RecentSearch::key`. |
| Search profiles upsert + move-to-top | ✅ | `SearchProfileStore`. |
| JSON import/export with merge rules | ✅ | `SettingsStore::import_json`/`export_json`. |
| Theme preference (13 variants) | ✅ | `ThemePreference` enum; Grex integer encoding preserved for `0…11`, with Grexa-only OLED Black at `12`. |
| Settings UI sections | ✅ | `apps/grexa-gui/qml/SettingsPage.qml`; auto-save on change as of v0.3 (no Apply button). |
| API key in keyring (never plaintext) | ✅ | `grexa_ai::secret`. |
| Restore defaults | ✅ | `SettingsStore::delete`. |

## CLI

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Positional `<path> <term>` | ✅ | `grexa-cli`. |
| All Grex CLI flags (regex, case, gitignore, hidden, binary, system, no-subfolders, symlinks, match-files, exclude-dirs, size-limit, size-unit, size-type) | ✅ | Mirror of `SearchArgs`. |
| Output formats (text / JSON / CSV) | ✅ | `--format`. |
| `--count`, `--files-only`, `--quiet` | ✅ | All wired. |
| Exit codes 0 / 1 / 2 | ✅ | Confirmed by `crates/grexa-cli/tests/cli.rs`. |
| Comparison / normalization / diacritic / culture | ✅ | `--comparison`, `--normalization`, `--ignore-diacritics`, `--culture`. |
| Baloo seed override | ✅ | `--use-index` / `--no-index`. |
| Container mode | ✅ | `--container`, `--runtime`. |
| `rg`-style aliases | ✅ | `--hidden`, `--no-ignore`. |
| Shell completion | ✅ | `grexa-cli completions <shell>`. |
| Man page | ✅ | `grexa-cli manpage`. |
| Ctrl-C cancellation | ✅ | Cooperative `CancelToken`. |

## Linux desktop integration

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Editor argv presets (Kate / VSCode / JetBrains / …) | ✅ | `grexa_core::desktop::open_in_editor_command`. |
| FileManager1 reveal | ✅ | `grexa_core::desktop::file_manager_show_items_uris` + `reveal_with_xdg_open`. |
| User path classifier (abstract URLs) | ✅ | `classify_user_path`. |
| KNotifications | ✅ | `notify_desktop` in `qobjects/search.rs` shells `notify-send` (which routes via `org.freedesktop.Notifications` / KNotifications). |
| Portal file picker | ✅ | `QtQuick.Dialogs.FolderDialog` — Breeze on KDE, XDG desktop portal under Wayland / Flatpak. Recent-path store integrated. |
| KDE color scheme + accent | 🟡 | Kirigami picks up the user's accent + theme today; full Qt palette swap via `KColorSchemeManager` still needs a cxx-qt-lib binding (v0.4 target — see release notes "Known limits"). |

## Localization

| Grex feature | Status | Implementation |
| ------------ | ------ | -------------- |
| Fluent catalog format | ✅ | `crates/grexa-i18n`. |
| Three locales (en/de/ja) | ✅ | `locales/<tag>/grexa.ftl`. |
| Plural-aware status text | ✅ | `Bundle::plural_count` for bare-count fragments (`count-matches`, `count-files`, `count-files-modified`, `count-matches-replaced`, `count-failures` in en/de/ja). QML side uses Qt's `qsTr("%n …(s)", "", n)` plural overload. |
| Runtime locale switching | ✅ | `Bundle::for_locale(Locale::from_tag(tag))`. |
| English fallback | ✅ | `Bundle` always carries a fallback bundle. |
| Locale sync gate | ✅ | `scripts/check_locale_sync.py` + a unit test. |
| RTL layout | 🟡 | Documented contract in `docs/accessibility.md`; Kirigami flips layout direction automatically when `LayoutMirroring.enabled` is set, but live verification with a translated RTL locale is pending a future translation contribution. |

## Non-applicable (Linux replacements)

| Grex behavior | Replacement |
| ------------- | ----------- |
| `%LocalAppData%\Grex\...` | `$XDG_*_HOME/grexa/...` |
| Windows Search index | Baloo (optional, deferred) |
| WSL / `\\wsl$\` / `\\wsl.localhost\` paths | None — Linux is native |
| UNC paths (`\\server\share`) | Mounted shares via gvfs / kio-fuse |
| Windows toasts | KNotifications / `org.freedesktop.Notifications` |
| Windows clipboard | QClipboard / wl-copy / xclip |
| Windows shell verbs | `org.freedesktop.FileManager1` + `xdg-open` |
| Recycle Bin | `gio trash` (future) |
| `Microsoft.Search` provider | Removed |
| `Microsoft.Toolkit.Uwp.Notifications` | Removed |
| Windows window position persistence | Window manager handles placement; only width/height persisted |

## Coverage map (Grex test files → Grexa equivalents)

| Grex test | Grexa equivalent |
| --------- | ---------------- |
| `Tests/Services/SearchServiceTests.cs` | `crates/grexa-core/src/search.rs` tests + `tests/gitignore_parity.rs` + `tests/property.rs` + `tests/root_safety.rs` |
| `Tests/Services/SettingsServiceTests.cs` | `crates/grexa-core/src/storage.rs::tests` |
| `Tests/Services/RecentSearchesServiceTests.cs` | `storage.rs::tests::search_history_*` |
| `Tests/Services/SearchProfilesServiceTests.cs` | `storage.rs::tests::profile_*` |
| `Tests/Services/RecentPathsServiceTests.cs` | `storage.rs::tests::recent_paths_*` |
| `Tests/Services/GitIgnoreServiceTests.cs` | `crates/grexa-core/tests/gitignore_parity.rs` (61 cases) |
| `Tests/Services/EncodingDetectionServiceTests.cs` | `crates/grexa-core/src/encoding.rs::tests` |
| `Tests/Services/AiSearchServiceTests.cs` | `crates/grexa-ai/src/lib.rs::tests` + `secret.rs::tests` |
| `Tests/Services/DockerSearchServiceTests.cs` | `crates/grexa-containers/src/{runtime,search}.rs::tests` |
| `Tests/Services/ContextPreviewServiceTests.cs` | `crates/grexa-core/src/preview.rs::tests` |
| `Tests/Services/ExportServiceTests.cs` | `crates/grexa-cli/tests/cli.rs` (CSV/JSON output) |
| `Tests/Services/LocalizationServiceTests.cs` | `crates/grexa-i18n/src/lib.rs::tests` |
| `Tests/Controls/SearchTabContentTests.cs` | `apps/grexa-gui/src/{tab,workspace,status}.rs::tests` |
| Grex.UITests | QML side; lands with cxx-qt PR + `QT_QPA_PLATFORM=offscreen` |

## Peer review

This document is the parity contract. Any new Grex feature shipped in
upstream releases must:

1. Add a row above with the Grex spec link.
2. Implement, defer, or non-applicable-mark in Grexa with a status icon.
3. Link to the audit doc that records the decision.

Every release walks this matrix once and confirms the status flag is
still accurate before the version is tagged.
