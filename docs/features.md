# Grexa Features

This inventory describes behavior exposed by the current desktop application
and CLI. Lower-level APIs that are not reachable from either user interface
are labeled explicitly.

For workflows, see [Using Grexa](usage.md). For exact flags and limits, see
[Reference](reference.md). Upstream status lives in
[Grex to Grexa feature parity](feature-parity.md).

## Search engine

### Literal search

- Case-insensitive by default, with case-sensitive mode.
- Whole-word matching through one shared adjacent-character rule.
- Ordinal, current-culture, and invariant-culture comparison.
- ICU/BCP-47 culture selection.
- Unicode NFC, NFD, NFKC, and NFKD normalization.
- Optional diacritic removal.
- Grapheme-aware mapping from normalized matches back to original byte and
  character offsets.

The GUI exposes literal, case, and whole-word controls directly. Advanced
comparison, normalization, diacritic, and culture controls are available from
the CLI and persisted settings schema.

### Regex search

- Fast Rust `regex` engine for linear-time patterns.
- `fancy-regex` extended engine for lookaround and backreferences.
- Automatic engine selection.
- CLI engine pinning with `--regex-engine fast|extended`.
- Per-line time, backtrack, match-count, and preview limits.
- Full-haystack capture re-query for correct lookaround replacement.

### Filesystem traversal

- Recursive search by default.
- Optional `.gitignore`, `.ignore`, and global Git exclude handling without
  requiring a Git repository.
- Hidden-path, dependency/system-directory, and symlink controls.
- Root safety for `/proc`, `/sys`, `/dev`, and `/run`.
- File-name include/exclude globs.
- Directory-name lists or root-relative regex exclusions.
- Less/equal/greater file-size filters in KB, MB, or GB.
- Exact, case-sensitive Linux path behavior.

### Results

- Content mode: one row per matching line.
- Files mode: one row per matching file with aggregate count, first match,
  encoding, size, extension, and modification time.
- 1-based line and Unicode character columns.
- Stable sorting with deterministic tie-breakers.
- Loaded-result literal or regex filtering.
- Context preview with 1 through 20 lines before and after.
- CSV, JSON, and text from the CLI.
- CSV, JSON, and Markdown export from the GUI.
- Copy path, file name, relative path, line content, or `path:line`.
- Open at line in common editors.
- Reveal with `org.freedesktop.FileManager1`, with `xdg-open` fallback.
- Move to freedesktop Trash through `gio`.

### Cancellation and progress

- Cooperative `CancelToken` shared by search and replace.
- Typed progress events for scanned, skipped, and matched files.
- Ctrl-C cancellation in the CLI.
- Stop button and Escape cancellation in the GUI.
- Partial search results remain available after cancellation.
- GUI row delivery is coalesced to avoid flooding the Qt event loop.

## Searchable documents

`--include-binary` and the GUI document toggle enable:

| Format | Extraction |
| ------ | ---------- |
| DOCX | Word document XML |
| XLSX | Shared strings and comments |
| PPTX | Slide XML |
| ODT / ODS / ODP | ODF content XML |
| ZIP | Entry names and recognized text entries |
| RTF | Best-effort control-word stripping |
| PDF | Poppler `pdftotext` |

Results retain the original document path. Document extraction is read-only;
replace does not rewrite archive/document formats.

Extraction is bounded by entry-count, entry-size, output-size, and PDF timeout
limits documented in [Reference: resource bounds](reference.md#resource-bounds).

## Encoding

Detection order:

1. UTF-8, UTF-16, or UTF-32 byte-order mark.
2. Strict UTF-8.
3. `chardetng` heuristic with `encoding_rs` decoding.
4. Lossy UTF-8 fallback for malformed input.

Replace round-trips UTF-8, UTF-8 BOM, UTF-16 LE/BE, and supported legacy
encodings. UTF-32 is detected for display but is not decoded or rewritten as
UTF-32.

## Replace

- Reuses the exact options from the completed preview search in the GUI.
- Shares the CLI's search flags through one argument/options builder.
- Supports literal and regex capture replacement.
- Preserves CRLF/LF shape and final-newline state through whole-buffer
  substitution.
- Preserves file permissions.
- Refuses files above the shared 512 MiB read ceiling.
- Writes a temporary file in the destination directory and atomically persists
  it.
- Records modified and failed paths in a replace journal.
- Clears the journal after clean completion.
- Surfaces a residual journal at GUI startup when enabled.
- Caps files, matches per file, and extended-regex CPU time.

There is no backup-content or undo store. Users needing rollback should use
version control or filesystem snapshots.

The library contains a container copy-out/replace/copy-back path, but container
replacement is not exposed in the GUI or CLI. Documents and archives are
search-only.

## Desktop workbench

- Qt 6 and Kirigami on Wayland or X11.
- Search tabs with independent form and result snapshots.
- Eight-tab cap and bounded inactive snapshot memory.
- Search path history with type-ahead selection and removal.
- Search History page with filtering and form restoration.
- Named Profiles page with load and delete actions.
- Regex Builder with presets, errors, highlights, and match list.
- Context preview dialog.
- Filter drawer and optional AI drawer.
- Content/Files mode switching and sortable result headers.
- Appearance themes, including system, light, dark, OLED black, and Grex
  palette variants.
- Auto-saved settings with visible success/failure status.
- Desktop completion notifications.
- Single-instance lock with existing-window activation.
- In-app About, Credits, third-party licenses, and GPL text.

## Plain-file database

Grexa uses the separately maintained,
[Apache-2.0 grexa-db engine](https://github.com/visorcraft/grexa-db) for:

- recent paths;
- completed search history;
- saved search profiles.

Each record is Markdown with YAML frontmatter under
`$XDG_DATA_HOME/grexa/db/`. Schemas are `schema.md` files. Writes are atomic,
and legacy Grexa JSON stores migrate on first use.

The **Tools → Database** page can open any grexa-db root and:

- list collections and schemas;
- inspect record frontmatter;
- filter typed fields;
- validate records;
- materialize filesystem views as symlink directories;
- list and delete views.

Derived secondary indexes accelerate selective queries and can be rebuilt from
the source records.

## Container search

- Docker detection through `DOCKER_HOST`, standard socket, and CLI.
- Rootless Podman detection through `$XDG_RUNTIME_DIR`.
- Rootful Podman detection through `/run/podman/podman.sock`.
- Running-container selection by ID or name.
- Direct `grep` through an argv array, never a shell.
- NUL-delimited output where supported, with BusyBox retry.
- Whole-word and maximum-result handling.
- Unicode/culture options through mirror fallback when direct grep cannot
  express them.
- Local archive mirror fallback when `grep` is absent.
- Container-path rewriting after mirror search.
- 30-second command timeouts and bounded stdout/stderr.
- Automatic stale-mirror cleanup.

Local filesystem traversal flags do not apply to the direct in-container path.
See [Using Grexa: container search](usage.md#container-search).

## AI Search

- Explicit `ai_search_enabled` opt-in, off by default.
- OpenAI-compatible `GET /v1/models` and `POST /v1/chat/completions`.
- Endpoint normalization for a bare base URL, `/v1`, or
  `/v1/chat/completions`.
- Optional model discovery.
- Synchronous `ureq` transport with no redirect following.
- 90-second request and 4 MiB response limits.
- Bearer credentials only over HTTPS or loopback HTTP.
- One key per canonical endpoint in the Linux Secret Service.
- No plaintext fallback.
- One in-flight request at a time.
- Single-turn chat requests.
- Bounded match summaries with context and `path:line` citations.
- Visible truncation disclosure when rows or evidence do not fit.

AI responses never modify files. See [AI provider scope](ai-provider-scope.md)
and [Security and privacy](SECURITY.md).

## CLI

- One-shot local search with positional path and term.
- Search/replace shared behavior flags.
- Container target and runtime selection.
- Text, JSON, and CSV local-search output.
- Count, files-only, and quiet modes.
- Grep-like exit codes.
- Dry-run and applied replace.
- Bash, Zsh, Fish, Elvish, and PowerShell completion generation.
- Generated roff man page.
- TTY control-sequence sanitization.
- Spreadsheet-formula neutralization in CSV.
- Structured file and stderr logging through `tracing`.

## Settings and persistence

- XDG Base Directory compliance.
- Atomic JSON settings.
- Plain Markdown history/profile/recent-path records.
- Daily GUI log rotation.
- Separate appended CLI log.
- Interruption-only replace journal.
- Bounded temporary container mirrors.
- Keyring-only AI secrets.

The complete keys and paths are in
[Reference: settings schema](reference.md#settings-schema) and
[Reference: data paths](reference.md#data-paths).

## Localization

- Fluent catalogs embedded at compile time.
- English, German, and Japanese.
- English fallback.
- Named placeholders and plural selectors.
- Runtime locale selection from BCP-47 or POSIX-style tags.
- Exact key-set parity tests and a source/QML sync script.

See [Translating Grexa](translations.md).

## Accessibility

- Accessible roles and names on shared controls and main result/chat lists.
- Keyboard navigation and global shortcuts.
- Reduced-motion setting.
- High-contrast token setting.
- Terminal-friendly line output and documented exit codes.
- No color-only CLI status contract.

Manual assistive-technology verification remains a release responsibility. See
[Accessibility](accessibility.md).

## Optional integrations

| Integration | Unlocks |
| ----------- | ------- |
| Poppler `pdftotext` | PDF search |
| Docker or Podman | Container search |
| KWallet, GNOME Keyring, or compatible Secret Service | AI API-key storage |
| common editors | Open at matching line |
| `wl-copy` or `xclip` | GUI copy actions |
| `gio` | Move to Trash |
| desktop notification tools | Completion notifications and activation |

The Baloo adapter and compatibility setting ship, but Baloo candidate seeding
remains deferred and does not alter searches.
