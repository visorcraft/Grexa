# Grexa Features

A user-facing inventory of what Grexa does today (v1.0). The
source-of-truth audits live in `docs/grex-*-audit.md`; this doc is
the consumer-shaped view. The Grex ↔ Grexa parity matrix lives in
[feature-parity.md](feature-parity.md).

## Search

### Modes

- **Text**: literal substring matching, case-sensitive or
  case-insensitive (default). Configurable string-comparison mode
  (`ordinal`, `current-culture`, `invariant-culture`), Unicode
  normalization (`form-c`, `form-d`, `form-kc`, `form-kd`), and
  optional diacritic stripping (`café` → `cafe`).
- **Regex**: PCRE-style with a two-engine cascade. Simple patterns use
  the fast Rust `regex` crate; lookaround / backreference / conditional
  groups automatically fall through to `fancy-regex`. The engine choice
  is visible in `tracing` logs.

### Result modes

- **Content** — one row per matching line: file name, line number,
  column number, snippet, full path, match count.
- **Files** — one row per matching file with aggregated counts plus
  first-match preview, encoding label, size, modified time.

### Filters

- Respect `.gitignore` (and `.ignore`, global git excludes) without
  requiring a real git repo.
- Include / exclude hidden files
- Include / exclude system directories (`.git`, `vendor`,
  `node_modules`, etc., plus Linux pseudo filesystems `/proc`, `/sys`,
  `/dev`, `/run`)
- Include / exclude binary files (with extension allowlist)
- Include / exclude symbolic links
- Recurse subdirectories
- File-name globs (`*.rs|*.toml|-target*`, with `-` prefix excluding)
- Directory excludes (`bin,obj,target,node_modules`, or regex when the
  string contains regex metacharacters)
- Size limit (`Less` / `Equal` / `Greater` than N KB / MB / GB)

### Search execution

- **Cooperative cancellation** via `CancelToken`. Every walker entry
  and every 64th line in a file checks the flag. Partial results are
  preserved on cancel; `SearchSummary.cancelled = true` flags the
  truncation.
- **Streaming progress** via `ProgressEvent` (`FileScanned`,
  `FileSkipped`, `Match`). The GUI uses these to populate the table
  model batched.
- **Stable, deterministic results**: sort defaults (`Name asc` for
  content, `Matches desc` for files) plus stable tie-breakers across
  parallel runs.

## Searchable document support

When `--include-binary` is set, Grexa transparently extracts text
from:

- **OOXML**: `.docx`, `.xlsx`, `.pptx`
- **ODF**: `.odt`, `.ods`, `.odp`
- **ZIP**: file names + every textual entry inside
- **PDF**: via `pdftotext` (Poppler)
- **RTF**: control-word stripping in pure Rust

Search results from these formats list the container file path so
"open in editor" opens the original document, not the extracted text.

## Encoding detection

Three-tier cascade:

1. **BOM**: UTF-8, UTF-8 BOM, UTF-16 LE/BE, UTF-32 LE/BE.
2. **Strict UTF-8 fast path** for BOM-less files.
3. **`chardetng` heuristic** for legacy encodings (Windows-1252,
   Shift-JIS, etc.).

Each match row reports the detected encoding label. Invalid bytes
fall back to lossy UTF-8 with U+FFFD replacements so search never
crashes on malformed input.

## Safe replace

- **Atomic same-filesystem temp-file-then-rename** via
  `tempfile::NamedTempFile::persist`.
- **Capture-group regex replace** with `$1`, `$name` references.
- **Permission preservation**.
- **CRLF / no-final-newline preservation** through full-buffer
  substitution.
- **Encoding round-trip**: UTF-16 LE/BE files stay UTF-16; UTF-8
  files stay UTF-8; legacy encodings detected via `chardetng` are
  re-encoded through `encoding_rs`.
- **Crash-recovery journal** at `$XDG_STATE_HOME/grexa/replace-journal.json`
  — every replaced file is logged before the operation completes, so
  a SIGKILL leaves an accurate "modified-so-far" list behind.
- **No silent backup fallback**. If the user wants undo, they snapshot
  the tree before launching the replace.
- **Disabled for containers and for ZIP / OOXML / ODF / PDF / RTF
  archives** in v1.0.

## Container search

- **Docker** detected via `$DOCKER_HOST` and `/var/run/docker.sock`,
  with CLI fallback when only the binary is on `$PATH`.
- **Rootless Podman** detected via `$XDG_RUNTIME_DIR/podman/podman.sock`.
- **Rootful Podman** via `/run/podman/podman.sock`, with CLI fallback.
- **Direct grep** inside the container via argv-array `exec` —
  immune to shell-quoting bugs on paths / patterns containing spaces,
  colons, globs, or newlines.
- **Archive mirror fallback** when the container has no `grep` —
  `docker cp` / `podman cp` to
  `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-ts>`
  and run the local search engine over the mirrored tree.
- **Container path display** even when mirroring is used; the
  `used_mirror` flag on the summary lets the UI badge the result.
- **Writable container replace** is implemented at the library level
  (`grexa_containers::replace_container`): files are copied out, replaced
  locally, and copied back. GUI and CLI integration are still pending, so
  the feature is not yet exposed to end users.

## AI Search Chat

- OpenAI-compatible HTTP shape only (`POST /v1/chat/completions`,
  `GET /v1/models`).
- Endpoint normalization handles bare hosts, `/v1`, `/v1/chat/completions`,
  and trailing slashes uniformly.
- Model discovery via `/v1/models` when the user doesn't specify a model.
- **API keys stored in the system keyring** (`org.freedesktop.secrets`
  on Linux) keyed by endpoint, so multiple endpoints don't share a key.
- **Opt-in**: `DefaultSettings.ai_search_enabled` defaults to `false`.
- **No telemetry**; see [SECURITY.md](SECURITY.md).
- See [ai-provider-scope.md](ai-provider-scope.md) for the full
  in-scope / out-of-scope matrix.

## Context preview

- Configurable before/after line counts (1-20 each, clamped at the
  service boundary).
- Returns line numbers + content + match-line index for the UI.
- Encoding-aware reading (UTF-8 / UTF-8 BOM / UTF-16 / chardetng
  fallback).
- Tolerates empty files, out-of-range line numbers, and missing files.

## Settings, history, profiles, recent paths

- Settings JSON at `$XDG_CONFIG_HOME/grexa/settings.json`.
- Recent paths at `$XDG_DATA_HOME/grexa/recent_paths.json` (cap 20,
  case-sensitive dedupe, type-ahead filter).
- Search history at `$XDG_DATA_HOME/grexa/search_history.json` (cap
  20, 7-field dedupe key matching Grex byte-for-byte).
- Search profiles at `$XDG_DATA_HOME/grexa/search_profiles.json`
  (case-insensitive name, move-to-top on upsert).
- Atomic write + Grex JSON import compatibility.
- Theme preference, comparison mode, normalization, diacritic
  sensitivity, AI endpoint/model, column visibility, window
  dimensions, default match/exclude globs, context preview line
  counts.

## Localization

- Fluent (`.ftl`) catalog format. Three locales today: `en`, `de`,
  `ja`. Translation key parity enforced by
  `scripts/check_locale_sync.py` and a `cargo test`.
- Plural-aware (ICU `select` ranges) — Grex's English-only
  `string.Format` plural failures don't survive the port.
- Runtime locale switching via `Bundle::for_locale(Locale::from_tag(tag))`.

## CLI

- Positional `<path> <term>` plus every flag Grex's CLI exposed:
  - Search behavior: `--regex` `--case-sensitive` `--gitignore`
    `--include-hidden` `--include-binary` `--include-system`
    `--no-subfolders` `--include-symlinks` `--match-files`
    `--exclude-dirs` `--size-limit` `--size-unit` `--size-type`
    `--whole-word` / `-w` `--max-results <N>`
    `--regex-engine <auto|fast|extended>`
  - Output: `--format text|json|csv` `--count` `--files-only`
    `--quiet`
  - Advanced: `--comparison` `--normalization` `--ignore-diacritics`
    `--culture` `--use-index` `--no-index`
  - Container: `--container <id>` `--runtime auto|docker|podman`
  - `rg`-style aliases: `--hidden`, `--no-ignore`
- Replace subcommand: `grexa-cli replace <path> <term> <replacement>`
  with flags `--regex`, `--case-sensitive`, `--gitignore`,
  `--include-hidden`, `--include-binary`, `--include-system`,
  `--no-subfolders`, `--include-symlinks`, `--match-files`,
  `--exclude-dirs`, `--dry-run`.
- Exit codes: `0` matches, `1` no matches, `2` error.
- `completions <shell>` and `manpage` subcommands.
- Ctrl-C cancels the in-flight search.
- Structured tracing logs to `$XDG_STATE_HOME/grexa/grexa.log`
  (configure verbosity with `GREXA_LOG=debug`).

## Linux desktop integration

- Editor presets with correct open-at-line flags for Kate, KWrite,
  VS Code, VSCodium, Sublime Text, JetBrains IDEs, GNOME Text
  Editor, Neovim, and `xdg-open`.
- "Reveal in file manager" via `org.freedesktop.FileManager1.ShowItems`
  with `xdg-open` fallback.
- `file://` URI percent-encoding for the FileManager1 D-Bus call.

## Optional integrations

- **Baloo** candidate seeding (KDE file index). The
  [Baloo spike](baloo-spike.md) recommended **defer**; the trait
  surface exists for a future enable.
- **PDF text** via `pdftotext` (Poppler). Falls back gracefully when
  the binary is missing.
