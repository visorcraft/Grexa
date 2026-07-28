# Grexa Architecture

Grexa is a synchronous Rust workspace with two front ends: a Clap CLI and a
Qt 6/Kirigami GUI connected through cxx-qt. Both call the same search,
encoding, replacement, storage, document, and container APIs.

## Dependency direction

```text
grexa-i18n
     │
     └──────────────────────────────┐
                                    ▼
grexa-db ──▶ grexa-core ──▶ grexa-ai
                 │  │
                 │  └────▶ grexa-containers
                 │              │
                 ├───────────────┴────▶ grexa-cli
                 │
                 └────────────────────▶ grexa GUI
                         grexa-ai ─────▶ ▲
                  grexa-containers ───▶ │
                       grexa-i18n ─────▶ │
                         grexa-db ─────▶ │
```

The arrows mean "is used by." There are no GUI or CLI dependencies in the
engine crates.

`grexa-db` is an external pinned Git dependency, not a workspace member. It is
Apache-2.0 so it can remain usable outside GPL applications. The rest of Grexa
is GPL-3.0-only. `grexa-db` must never depend back on a Grexa GPL crate.

## Repository layout

```text
grexa/
├── apps/
│   └── grexa-gui/
│       ├── build.rs          cxx-qt module and QML resource list
│       ├── qml/              Kirigami pages and shared controls
│       └── src/
│           ├── main.rs       process, logging, Qt engine, integration setup
│           ├── qobjects/     Rust/QML controller bridges
│           └── workspace.rs  process-wide stores and Fluent bundle
├── crates/
│   ├── grexa-core/           search, replace, documents, encoding, storage
│   ├── grexa-containers/     Docker/Podman process and search adapters
│   ├── grexa-ai/             OpenAI-compatible client and Secret Service
│   ├── grexa-cli/            Clap binary
│   └── grexa-i18n/           Fluent runtime and embedded catalogs
├── docs/                     public docs and upstream behavior audits
├── packaging/                distro, Flatpak, AppImage, desktop metadata
├── scripts/                  verification, benchmarks, locale checks
├── Cargo.toml                workspace and shared dependency versions
└── Justfile                  developer and packaging entry points
```

## Crate responsibilities

### `grexa-core`

Owns all local-file semantics:

- `models.rs`: search options, results, summaries, sort/output enums;
- `search.rs`: traversal, filtering, matching, progress, aggregation;
- `pattern.rs`: fast and extended regex engines;
- `encoding.rs`: BOM, UTF-8, `chardetng`, and write-back encoding;
- `documents.rs`: OOXML, ODF, ZIP, RTF, and PDF extraction;
- `replace.rs`: substitution, journaling, atomic persistence;
- `preview.rs`: context extraction;
- `storage.rs`: XDG paths, JSON settings, legacy JSON stores and types;
- `db.rs`: grexa-db wrappers for recent paths, history, and profiles;
- `desktop.rs`: editor argv, `xdg-open`, and Trash helpers;
- `baloo.rs`: deferred candidate-source adapter;
- `cancel.rs`: cooperative cancellation;
- `constants.rs`: shared hard caps.

It is synchronous and has no Qt, CLI, container-daemon, or async-runtime
dependency.

### `grexa-containers`

Owns the process boundary to Docker and Podman:

- runtime discovery;
- container listing;
- bounded command execution;
- direct grep;
- archive copy/mirror fallback;
- container/local path translation;
- stale mirror cleanup;
- an internal copy-out/replace/copy-back API.

`CommandRunner` is the mock boundary. Production uses
`SystemCommandRunner`; default tests use a fake.

### `grexa-ai`

Owns:

- OpenAI-compatible endpoint construction;
- model discovery and chat requests;
- response/error parsing;
- evidence packing;
- HTTP timeout, redirect, response-size, and credential rules;
- API-key storage in `org.freedesktop.secrets`.

The client is synchronous and uses `ureq`. `HttpTransport` is the test boundary.
No API key reaches QML or disk.

### `grexa-i18n`

Embeds Fluent catalogs and provides:

- locale parsing;
- English fallback;
- keyed formatting;
- plural formatting;
- exact locale key-set tests.

The GUI accesses the bundle through controller invokables. QML does not use
`qsTr()` for shipped strings.

### `grexa-cli`

Owns only:

- Clap parsing and help;
- conversion from shared flags to `SearchOptions`;
- process exit codes;
- text, JSON, and CSV rendering;
- Ctrl-C setup;
- tracing subscriber setup;
- completion and man-page generation;
- container runtime selection.

Search and replace behavior stays in engine crates.

### `grexa` GUI

Owns presentation and desktop orchestration:

- application/window lifecycle;
- cxx-qt QObjects and Qt model roles;
- GUI worker threads and queued Qt-thread updates;
- search tabs and result snapshots;
- QML pages, controls, themes, and accessibility metadata;
- file picker, clipboard, editor, notification, and file-manager actions;
- user desktop-entry/icon installation for local runs;
- single-instance activation;
- GUI logging.

Algorithmic search, replace, encoding, AI transport, and container logic remain
outside QML.

## Local search flow

```text
SearchOptions
     │
     ├── validate root and term
     ├── build normalization context once
     ├── compile file and directory filters once
     ├── compile PatternEngine once
     ▼
ignore::WalkBuilder(require_git(false), max_depth=64)
     │
     ├── cancel check
     ├── root-relative directory exclusion
     ├── hidden/system/gitignore traversal policy
     ├── metadata, size, file-name, and binary classification
     ▼
content source
     ├── document extractor when enabled
     └── encoding::read_text otherwise
     ▼
line scanner
     ├── 2 MiB compare slice
     ├── original-offset mapping
     ├── whole-word validation
     ├── match and time caps
     └── ProgressEvent
     ▼
Vec<SearchResult>
     ├── aggregate_file_results
     └── SearchSummary
```

The engine buffers the summary because Files mode and replace need complete
per-file information. A caller-supplied progress sink can stream rows while
that buffer is built.

## Pattern and Unicode rules

One `PatternEngine` is built per search:

- `Fast` wraps the `regex` crate;
- `Extended` wraps `fancy-regex`;
- `Auto` tries `Fast` and falls back only when the pattern needs unsupported
  constructs.

Literal case-insensitive/normalized search uses
`normalize_with_mapping`. It normalizes per grapheme segment and records a map
to original byte offsets. Match ends inside a normalized segment round up to
the original segment boundary so preview and replace never split a grapheme.

Whole-word behavior has one implementation:
`is_whole_word_match`. A match is whole only when the adjacent original
characters are not alphanumeric and not `_`.

## Replace flow

```text
ReplaceOptions
     │
     ├── create journal
     ├── search_with using the exact SearchOptions
     ├── deduplicate matching file paths
     ├── cap at 100,000 files
     ▼
for each file
     ├── reject searchable archive/document formats
     ├── capture metadata and permissions
     ├── decode content
     ├── precompute SubstitutionContext
     ├── apply literal or regex substitution
     ├── encode in the original supported encoding
     ├── NamedTempFile in destination directory
     ├── persist atomically
     ├── restore permissions
     └── append modified/failed path to journal
     ▼
clean completion clears journal
     │
     └── ReplaceSummary
```

The journal records scope and paths, not old file bytes. It supports diagnosis
after interruption, not undo.

## Container search flow

```text
LiveProbe
     └── detect Docker / Podman runtime descriptors
              │
              ▼
CliRuntime<SystemCommandRunner>
     ├── list_containers_timeout
     ├── exec_capture_timeout
     ├── archive_path
     └── copy_into_container
              │
              ▼
search_container
     ├── direct grep when matching semantics can be expressed
     │      ├── NUL-delimited output attempt
     │      ├── BusyBox retry
     │      └── parse byte columns into character columns
     └── mirror search otherwise
            ├── copy path to XDG cache
            ├── call grexa-core search
            └── rewrite results to container paths
```

Normalization, diacritic, and culture modes force the mirror path because
portable `grep` cannot reproduce them. Every runtime process has a timeout,
output cap, and process-group cleanup.

## AI request flow

```text
DefaultSettings + Secret Service
     └── AiSearchConfig { endpoint, Option<key>, Option<model> }
              │
              ▼
AiController worker thread
     ├── test endpoint: GET /v1/models
     ├── chat: POST /v1/chat/completions
     └── summary
           ├── visible SearchResult rows
           ├── context preview snippets
           ├── pack_evidence(character budget)
           └── POST /v1/chat/completions
              │
              ▼
AiSearchResponse queued to Qt thread
```

The opt-in setting is checked again in the controller before every request.
Only one request may be active. The transport does not follow redirects and
does not attach a bearer token to remote plaintext HTTP.

## Persistence

```text
$XDG_CONFIG_HOME/grexa/
└── settings.json

$XDG_DATA_HOME/grexa/db/
├── recent_paths/
│   ├── schema.md
│   └── entry-*.md
├── search_history/
│   ├── schema.md
│   └── entry-*.md
├── search_profiles/
│   ├── schema.md
│   └── entry-*.md
└── .grexa-index/

$XDG_STATE_HOME/grexa/
├── grexa.log
├── grexa-gui.<date>.log
└── replace-journal.json

$XDG_CACHE_HOME/grexa/
└── container-mirrors/
```

`SettingsStore` and record wrappers use same-directory temporary files for
atomic replacement. Tests use `AppPaths::under(tempdir)` and never touch the
real user configuration.

## GUI threading and state

Qt objects live on the GUI thread. Blocking work runs on Rust threads:

```text
QML action
   ▼
cxx-qt invokable on GUI thread
   ▼
std::thread::spawn
   ├── search / replace / HTTP / container / database work
   └── cxx_qt::Threading::queue
                          ▼
                 GUI-thread property/model update
```

Search rows flush at most about once per frame, with a bounded worker batch.
Counters update less often and are finalized at completion.

The process-wide `Workspace` owns settings, recent paths, history, profiles,
and the Fluent bundle. It is installed before the QML engine starts. One
`SearchController` model backs the active tab; inactive tab snapshots store
rows and status under stable tab IDs.

## Build boundary

`apps/grexa-gui/build.rs` declares the QML module URI
`com.visorcraft.Grexa`, registers each QObject bridge, lists every QML file,
and embeds QML/resources into the binary.

The cxx-qt toolchain:

1. generates QObject C++;
2. invokes Qt build utilities for the QML module;
3. compiles generated C++ through Cargo;
4. links Qt 6 libraries;
5. embeds QML at `qrc:/qt/qml/com/visorcraft/Grexa/qml/`.

No host CMake project is used.

## Cross-cutting controls

- **Cancellation**: one atomic `CancelToken`, polled at traversal and matching
  boundaries.
- **Errors**: typed errors in engine crates; user-facing summaries at front-end
  boundaries; production failures are logged rather than silently discarded.
- **Logging**: `tracing` everywhere; CLI and GUI install separate subscribers.
- **Localization**: user-visible QML/Rust strings route through Fluent.
- **Security**: no shell command construction for runtime/user inputs; helper
  programs receive argv arrays.
- **Bounds**: file, line, result, regex, process, response, and snapshot caps
  are part of public behavior.
- **Licensing**: `cargo-deny`, `cargo-audit`, generated credits, and bundled
  runtime license texts.

## Testing strategy

- Unit tests sit beside pure Rust modules.
- Filesystem tests use real `tempfile::TempDir` trees.
- Search parity matrices live under `crates/grexa-core/tests/`.
- CLI tests spawn the built binary with `assert_cmd`.
- Container tests mock only `CommandRunner`; opt-in live tests use a daemon.
- AI tests inject `HttpTransport`.
- QObject tests exercise Rust backing structs without a Qt display server.
- A separate headless launch gate proves that the QML root object
  instantiates.
- Locale key parity is enforced in Rust and by the dedicated Python checker.

Run [the documented gates](build-and-test.md#local-quality-gates) before a
pull request.
