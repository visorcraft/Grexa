<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Grex to Grexa Feature Parity

This matrix answers whether behavior from the Windows/WinUI
[Grex](https://github.com/visorcraft/grex) application is present in Grexa.
It describes Grexa 1.11.1.

Status:

- ✅ Shipped
- 🟡 Partial or intentionally narrower
- ⏸ Deferred
- 🟥 Not applicable on Linux

For current Grexa behavior, this document and the
[feature inventory](features.md) take precedence over implementation plans in
the historical `grex-*-audit.md` files.

## Search

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Multiple search tabs with independent state | ✅ | `SearchController` snapshots path, term, filters, sort, within-filter, results, and scroll state. The GUI allows eight tabs and bounds inactive snapshots. |
| Text and regex modes | ✅ | `grexa-core` uses the Rust `regex` engine first and the bounded `fancy-regex` engine when extended constructs require it. |
| Content and files result modes | ✅ | Content rows expose path, line, column, text, and match previews. Files mode aggregates matching rows by path. |
| Gitignore-aware traversal | ✅ | The `ignore` walker honors gitignore, hidden-file, symlink, recursion, system-path, glob, directory, and size options. |
| File globs and excluded directories | ✅ | `FileNameFilter` supports `|` or `;` terms and `-` exclusions. `ExcludeDirFilter` supports names or regex. |
| Whole-word matching | ✅ | One Unicode-aware adjacent-character rule is shared by text, regex, local, and container mirror search. |
| Case, culture, normalization, and diacritics | ✅ | ICU4X casing, ordinal/culture/invariant comparison, NFC/NFD/NFKC/NFKD normalization, and grapheme-aware source-offset mapping. |
| Searchable documents | ✅ | DOCX, XLSX, PPTX, ODT, ODS, ODP, ZIP, RTF, and PDF text extraction. PDF requires `pdftotext`. |
| Binary-file controls | ✅ | Known binary formats are skipped unless allowed. Searchable document containers are extracted rather than treated as raw text. |
| Progress and cancellation | ✅ | `ProgressEvent` and `CancelToken` connect the synchronous engine to CLI Ctrl-C and GUI cancellation. Partial results remain visible. |
| Stable sorting | ✅ | GUI content and file modes sort by their visible columns with stable path/line tie-breakers. |
| Search within current results | ✅ | Plain or regex within-filter, cached and fail-closed when invalid. |
| Context preview | ✅ | Local and container previews include configurable surrounding lines, line numbers, and match highlighting. |
| Result export and copy actions | ✅ | GUI exports visible rows as CSV, JSON, or Markdown and provides path/text copy actions. CLI emits text, JSON, or CSV for local search. |
| Windows Search acceleration | ⏸ | `use_file_index`, `--use-index`, and the Baloo adapter remain compatibility surfaces. They do not change current searches. |

## Replace

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Previewed local-file replace | ✅ | The GUI reuses the exact `SearchOptions` from the last search. CLI defaults to dry-run and requires `--apply` to write. |
| Regex capture expansion | ✅ | Numbered and named captures are expanded against the full source text so lookaround keeps working. |
| Atomic write | ✅ | Encoded output is written to a temporary file and persisted over the original. |
| Encoding and permissions preserved | ✅ | UTF BOM/encoding and filesystem permissions are retained when writeback is possible. |
| Confirmation | ✅ | Controlled by `replace_confirm`; CLI uses explicit `--apply`. |
| Bounded work | ✅ | Files, matches per file, and extended-regex time are capped. Searchable archive/document formats are never rewritten as raw bytes. |
| Crash/interruption journal | ✅ | `$XDG_STATE_HOME/grexa/replace-journal.json` records interrupted operations and is removed after a clean completion. |
| Undo or backup files | 🟡 | Neither is provided. Atomic replacement prevents partial writes, but users need version control or external backups for rollback. |
| Container replace | ⏸ | A crate-level copy-out/copy-back implementation exists, but neither public GUI nor CLI invokes it. |
| Archive/document replace | ⏸ | Search-only. |

## Encodings and documents

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| UTF BOM detection | ✅ | UTF-8, UTF-16, and UTF-32 BOMs are recognized. |
| Strict UTF-8 fast path | ✅ | Valid UTF-8 avoids heuristic detection. |
| Legacy encoding detection | ✅ | `chardetng` and `encoding_rs`. |
| Malformed input fallback | ✅ | Lossy decoding keeps searches available and reports the detected label. |
| OOXML and ODF extraction | ✅ | Bounded ZIP entry parsing with XML text extraction. |
| ZIP and RTF extraction | ✅ | Textual ZIP entries and RTF text are searchable. |
| PDF extraction | ✅ | Bounded `pdftotext` subprocess with timeout. |

## Containers

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Docker discovery and search | ✅ | Linux Docker CLI runtime. |
| Rootless and rootful Podman | ✅ | Added Linux runtime choices with auto-detection or `--runtime`. |
| Direct in-container grep | ✅ | NUL-delimited parsing when supported, BusyBox retry, char-based columns, whole-word filtering, and result caps. |
| Missing-grep fallback | ✅ | The target path is copied to a bounded local mirror and searched by `grexa-core`. |
| Container context preview | ✅ | Reads context through the selected runtime. |
| Command safety bounds | ✅ | Runtime commands have a 30-second timeout and bounded stdout/stderr capture. Mirror paths are constrained and cleaned up. |
| Local walker flags on direct grep | 🟡 | Local-only traversal flags have no meaning in the direct path. Comparison modes that grep cannot express force the mirror path. |
| Live daemon coverage | ✅ | `cargo test -p grexa-containers --features container-live -- live::`; tests self-skip when no supported daemon exists. |

## AI Search

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| OpenAI-compatible endpoint | ✅ | Model discovery and chat-completions requests through synchronous `ureq`. |
| Endpoint normalization | ✅ | Accepts a base URL, `/v1`, or `/v1/chat/completions` form. |
| Common response/error shapes | ✅ | Chat choice, legacy text choice, `output_text`, and structured provider errors. |
| Linux filter suggestions | ✅ | Optional prompt hints use Grexa terminology. |
| Search-result summary | ✅ | Up to 400 visible rows are packed into a configured 2,000 to 40,000 character evidence budget with truncation disclosure. |
| Keyring-only API key | ✅ | One credential per canonical endpoint in the Linux Secret Service. No plaintext fallback. |
| Opt-in and transport guardrails | ✅ | Disabled by default. Credentials require HTTPS or loopback HTTP. Redirects are disabled; request and response bounds apply. |
| Multi-turn provider conversation | 🟡 | The tab shows local chat bubbles, but each provider request is standalone and does not resend previous turns. |
| Tool calls, file upload, streaming, embeddings | ⏸ | Outside the current provider contract. |

## Settings and persisted lists

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Settings defaults and round-trip | ✅ | Atomic `$XDG_CONFIG_HOME/grexa/settings.json` through `SettingsStore`. |
| Recent paths | ✅ | grexa-db records under `db/recent_paths`, exact case-sensitive dedupe, cap 20. |
| Search history | ✅ | grexa-db records under `db/search_history`, current full search identity key, cap 20. |
| Search profiles | ✅ | grexa-db records under `db/search_profiles`, name-based upsert and move-to-top behavior. |
| Older Grexa JSON migration | ✅ | Empty database collections import the three legacy Grexa JSON files once and rename them to `.bak`. |
| Grex Windows backup import | ⏸ | No public converter. Recreate settings and profiles against Linux paths. See [Migration from Grex](migration-from-grex.md). |
| Grexa-format settings import/export | 🟡 | `SettingsStore` exposes library APIs, but the current GUI and CLI do not provide a backup command. |
| Theme preference | ✅ | System, light, dark, Grex-compatible named values, and Grexa OLED Black. |
| Secret persistence | ✅ | AI keys never enter settings JSON or QML. |

## CLI

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Positional path and term | ✅ | `grexa-cli <path> <term>`. |
| Shared search/replace flags | ✅ | `SharedSearchArgs` and one `build_search_options` path. |
| Text, JSON, and CSV | ✅ | Local search supports all three. Container output has a narrower text/count/files surface. |
| Count, files-only, and quiet | ✅ | Local search supports all. Container search supports count/files-only; accepted format/quiet flags do not alter its renderer. |
| Comparison, normalization, culture, and diacritics | ✅ | Same core semantics as the GUI. |
| Grep-style exit codes | ✅ | Search: 0 matches, 1 none, 2 failure. Replace: 0 modified, 1 none, 2 failure. |
| Ctrl-C cancellation | ✅ | Cooperative cancellation token. |
| Completion and man page generation | ✅ | Bash, Zsh, Fish, Elvish, PowerShell, and roff. |
| Baloo flags | ⏸ | Parsed for compatibility but currently have no search effect. |

## Linux desktop integration

| Grex/Windows behavior | Status | Grexa replacement |
| --------------------- | ------ | ----------------- |
| Open match in editor | ✅ | Built-in editor argv presets and a shell-free custom template using `{path}`, `{file}`, and `{line}`. |
| Reveal file | ✅ | `org.freedesktop.FileManager1.ShowItems`, then `xdg-open` fallback. |
| Recycle Bin | ✅ | `gio trash` when available. |
| Completion notification | ✅ | `notify-send`, with optional activation through `wmctrl`. |
| Clipboard helpers | ✅ | Qt clipboard plus `wl-copy` or `xclip` where an external helper is needed. |
| Native folder picker | ✅ | Qt Quick folder dialog and desktop portal integration. |
| Mounted network paths | ✅ | Local paths exposed by GVFS, KIO FUSE, CIFS, NFS, or another mount work normally. |
| Abstract `smb://` or `sftp://` URLs | 🟡 | Not search roots. Mount or browse the resource first and select its local path. |
| Window placement persistence | 🟥 | The Linux window manager owns placement. Grexa persists size only. |
| Windows toast activation and shell verbs | 🟥 | Replaced by freedesktop services and normal subprocess argv. |
| WSL and UNC path translation | 🟥 | Grexa searches native Linux-visible paths. |

## Localization and accessibility

| Grex behavior | Status | Grexa implementation |
| ------------- | ------ | -------------------- |
| Localized interface | ✅ | Embedded Fluent catalogs for English, German, and Japanese with English fallback. |
| Plural-aware counts | ✅ | Fluent plural selectors through `i18nPlural(...)`. |
| Runtime locale selection | ✅ | BCP-47 and common POSIX locale tags. |
| Locale parity gate | ✅ | Rust key-set test plus `scripts/check_locale_sync.py` for QML/source usage. |
| Keyboard operation | ✅ | Global navigation/search/tab shortcuts and keyboard-operable result actions. |
| Reduced motion and high contrast | ✅ | User settings and theme tokens. |
| Screen-reader metadata | 🟡 | Main controls, results, and chat expose accessible roles/names. Full AT-SPI regression automation is not present. |
| RTL locale verification | 🟡 | Layout mirroring support exists, but no RTL translation ships today. |

## Verification map

| Surface | Main verification |
| ------- | ----------------- |
| Core search/replace | Unit tests plus `gitignore_parity`, property, and root-safety integration suites |
| CLI behavior | `crates/grexa-cli/tests/cli.rs` |
| Containers | Mock-runner unit tests plus opt-in `container-live` tests |
| AI | Client, prompt, transport, and secret-store tests |
| Localization | `grexa-i18n` tests plus `scripts/check_locale_sync.py` |
| GUI bridge | Rust backing-struct tests and QML compilation |
| GUI launch | `just verify-gui` and the release/package smoke gates |
| Full repository | `just ci`; `just preflight` adds deny, audit, and credits |

Update this matrix when a Grex behavior changes, Grexa intentionally diverges,
or a deferred surface becomes user-visible.
