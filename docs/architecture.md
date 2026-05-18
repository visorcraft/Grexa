# Architecture

Grexa is a Cargo workspace. Every crate has a single responsibility
and a stable public surface; the GUI and CLI both consume the same
underlying APIs.

## Crate layout

```
grexa/
├── crates/
│   ├── grexa-core/          # search, replace, encoding, settings, …
│   ├── grexa-containers/    # docker/podman adapter
│   ├── grexa-ai/            # OpenAI-compatible HTTP client + secrets
│   ├── grexa-cli/           # `grexa-cli` headless binary
│   └── grexa-i18n/          # Fluent locales + Bundle runtime
├── apps/
│   └── grexa-gui/           # Qt 6 / Kirigami shell (crate: grexa)
├── docs/                    # this directory
├── packaging/               # Flatpak / AppImage / distro recipes
└── scripts/                 # locale sync, future fixture generators
```

## Data flow

### Local search

```
SearchOptions
   │
   ▼
search_with(options, cancel, progress)
   ├── BOM detection + chardetng fallback (encoding.rs)
   ├── ignore::WalkBuilder with .require_git(false)
   ├── classify_skip → SkipReason
   ├── document extractor (when --include-binary) → text
   ├── PatternEngine::Fast | PatternEngine::Extended
   ├── line scan → SearchResult batch
   ├── progress: FileScanned, FileSkipped, Match
   └── aggregate_file_results → FileSearchResult
                                                  │
                                                  ▼
                                          SearchSummary
```

### Replace

```
ReplaceOptions { search: SearchOptions, replacement }
   │
   ▼
replace_with(options, cancel, progress)
   ├── search_with (drives the same filter set)
   ├── dedupe file paths
   ├── for each file:
   │    ├── symlink_metadata (capture original permissions)
   │    ├── read_text (encoding-aware)
   │    ├── apply_substitution (text / regex / regex-extended)
   │    ├── encode_for_writeback (round-trip the encoding)
   │    ├── atomic_write (NamedTempFile::new_in.persist)
   │    └── restore_permissions
   ├── journal append → $XDG_STATE_HOME/grexa/replace-journal.json
   └── on clean exit: clear_journal
                                                  │
                                                  ▼
                                          ReplaceSummary
```

### Container search

```
detect_runtimes(LiveProbe)
   │
   ▼
CliRuntime<SystemCommandRunner> (Docker | Podman, rootless or not)
   │
   ├── list_containers       → Vec<ContainerInfo>
   ├── has_grep              → bool (which grep)
   ├── exec_capture          → CommandResult (argv array)
   └── archive_path          → PathBuf in $XDG_CACHE_HOME mirror
                                                  │
                                                  ▼
                            search_container(runtime, container, options)
                                  ├── has_grep ? direct_grep : mirror_search
                                  ├── parse_grep_output    (colon-tolerant)
                                  └── rewrite_path         (mirror → container path)
                                                  │
                                                  ▼
                                          ContainerSearchSummary
```

### AI Search

```
AiSearchConfig { endpoint, api_key (via keyring), model }
   │
   ▼
AiSearchClient<UreqTransport>
   ├── test_endpoint  → GET /v1/models
   ├── discover_model → first id from /v1/models (fallback gpt-4o-mini)
   └── send_chat      → POST /v1/chat/completions
                          ├── build_messages: system + context + history
                          ├── extract_assistant_content (choices/message/content, choices/text, output_text)
                          └── extract_error_message (.error.message, .message, raw)
                                                  │
                                                  ▼
                                          AiSearchResponse
```

## Cross-cutting concerns

- **Cancellation**: `CancelToken` (`Arc<AtomicBool>`) is consumed by
  the search and replace pipelines. The CLI installs a Ctrl-C handler;
  the GUI exposes a Stop button.
- **Progress streaming**: `ProgressEvent` is the canonical channel
  between the worker and the UI. The CLI ignores it for one-shot
  searches; the GUI batches into the table model.
- **Logging**: every crate uses the `tracing` crate. The CLI wires the
  global subscriber with both stderr and a non-blocking file appender
  rooted at `$XDG_STATE_HOME/grexa/grexa.log`.
- **Localization**: `grexa-i18n::Bundle` is shared between CLI text
  output (when localized) and the GUI. The English catalog is the
  source of truth; sync is enforced in CI.
- **Configuration & data**: every persistent artifact respects the
  XDG Base Directory spec. Test code overrides paths via
  `AppPaths::under(temp_dir)`.

## Testing strategy

- **Unit tests** live in each module's `#[cfg(test)]` block. 291
  tests passing across the workspace as of v0.3.
- **Integration tests** are in `crates/<crate>/tests/*.rs`:
  - `grexa-core/tests/gitignore_parity.rs` — 61 cases mirroring
    `docs/grex-gitignore-audit.md`.
  - `grexa-core/tests/property.rs` — proptest properties for globs,
    exclude dirs, determinism, snippet caps.
  - `grexa-core/tests/root_safety.rs` — auto-exclusion of pseudo
    filesystems.
  - `grexa-cli/tests/cli.rs` — 16 spawned-process integration tests
    via `assert_cmd`.
- **Mocked HTTP / process** via `MockTransport` and `MockCommandRunner`
  so neither AI nor container tests require a live daemon.
- **Clippy `-D warnings`** is the merge gate. `just lint` runs the
  same command CI uses.

## What the GUI adds

The GUI is intentionally a thin presentation layer. The Rust core
exposes:

- Streaming search/replace primitives + cancellation
- Typed `SearchResult` and `FileSearchResult` records
- Localized strings via `grexa-i18n::Bundle`
- File-manager / editor argv builders (`grexa_core::desktop`)
- File-URI percent-encoding for FileManager1
- Theme-preference enum with Grex integer round-trip

The QML shell binds these to the user-visible widgets via `cxx-qt`
QObjects. No business logic lives in the GUI.

## Future architectural extensions

- **`grexa-core` async path**: today every search/replace call is
  blocking. A `tokio`-backed variant would let the GUI run multiple
  tabs concurrently without thread sprawl. Per-tab state isolation
  already lives in the controller (see
  `apps/grexa-gui/src/qobjects/search.rs::TabSnapshot`); the
  scheduler is the remaining piece.
- **Baloo seeding**: trait surface ships in `grexa-core::baloo`;
  wiring through the search engine is deferred per
  [docs/baloo-spike.md](baloo-spike.md).
- **Custom regex engine option**: today the cascade picks `regex`
  vs `fancy-regex` automatically. A future `SearchOptions.regex_engine`
  field would let advanced users pin the slower-but-richer engine.
