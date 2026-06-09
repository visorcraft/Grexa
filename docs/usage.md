# Using Grexa

Workflow walk-throughs for the cases the test suite exercises and the
GUI surfaces. Every command in this doc is verified by either an
integration test or a manual smoke run.

## Local search

### Find every `TODO` in your project

```bash
grexa-cli ~/code/grexa TODO
```

Output is `file:line:column:content`, one match per line. Exit code 0
when matches were found, 1 when none, 2 on error.

### Constrain by file extension

```bash
grexa-cli ~/code/grexa TODO --match-files '*.rs|*.md|-target*'
```

The `|` separates patterns; a `-` prefix turns a pattern into an
exclusion. `;` is also accepted as a separator.

### Exclude directories by name or regex

```bash
grexa-cli ~ secret --exclude-dirs '.git,node_modules,target'
```

If the value contains regex metacharacters (`^`, `$`, `|`) without
a `,` or `;`, it's treated as a regex applied to every path
component.

### Regex with capture-group preview

```bash
grexa-cli ~/code 'fn\s+(\w+)\s*\(' --regex
```

The two-engine cascade tries the fast `regex` crate first and falls
through to `fancy-regex` automatically for lookaround / backreferences.

### Search with culture-aware comparison

```bash
grexa-cli ~/notes "café" --comparison invariant-culture --ignore-diacritics
```

Matches `café`, `Café`, and `cafe` interchangeably. See
[grex-culture-comparison-audit.md](grex-culture-comparison-audit.md)
for the full matrix.

### Searchable documents

```bash
grexa-cli ~/Documents "Q3 forecast" --include-binary
```

Extracts text from `.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`,
`.zip`, `.rtf`, and `.pdf` (via `pdftotext`) before line-scanning.

## Replace

```bash
grexa-cli replace <path> <term> <replacement> [flags]
```

Replaces every match of `<term>` with `<replacement>` across all
matching files. Supports every search-behavior flag with identical
semantics (matching, filtering, size limits, Unicode comparison,
indexing, `--max-results`) — see the
[flag reference](reference.md#flags) — so the file set it rewrites is
exactly the set the equivalent search previews. Add `--dry-run` to
preview which files and matches would be affected without writing
anything.

In the GUI: Replace button → review the term + replacement + filter
snapshot → Replace All → auto-flip to Files mode. If the process was
killed mid-run, the residual journal dialog surfaces on next launch
so the half-written work is reviewable or dismissable. The library
API is documented in [features.md](features.md#safe-replace).

## Container search

### List runtimes Grexa detected

The runtime auto-detector probes Docker (`$DOCKER_HOST` +
`/var/run/docker.sock` + CLI), rootless Podman
(`$XDG_RUNTIME_DIR/podman/podman.sock`), and rootful Podman
(`/run/podman/podman.sock`).

### Search inside a running container

```bash
grexa-cli /etc/nginx TODO --container web --runtime podman
```

- Positional `path` is interpreted as the *in-container* path.
- `--runtime auto` (the default) picks whatever's available.
- When the container has no `grep`, Grexa transparently archives the
  path to `$XDG_CACHE_HOME/grexa/container-mirrors/...` and runs the
  local search engine against the mirror. Stderr surfaces
  `grexa-cli: used mirror fallback (no grep in container)`.

### Mirror cleanup

```bash
# Drop snapshots older than one hour (3600 s).
python3 - <<'PY'
import grexa_containers
# Or use the library entry point directly:
PY
```

Programmatic: `grexa_containers::prune_mirrors(3600)` from a script /
the GUI cleanup hook. The GUI calls this on startup and after every
container search.

## AI Search Chat

The GUI ships an in-tab AI chat panel
(`apps/grexa-gui/qml/AiChatPanel.qml`) wired to `app.aiController`.
Enable it in Settings → AI Search; pick an endpoint and (optionally)
a model. API keys are stored in the system keyring, never on disk.

The same client is also available as a library — from a script:

```rust
use grexa_ai::{AiSearchClient, AiSearchConfig, AiSearchContext, AiConversationTurn, AiRole, store_api_key};

// 1. Store the key once — uses the system keyring.
store_api_key("https://api.openai.com/v1", "sk-…")?;

// 2. Build the config + client.
let config = AiSearchConfig {
    endpoint: "https://api.openai.com/v1".into(),
    api_key: grexa_ai::load_api_key("https://api.openai.com/v1")?,
    model: Some("gpt-4o-mini".into()),
};
let client = AiSearchClient::new();

// 3. Build the context (path + query + filters).
let context = AiSearchContext {
    search_path: "/home/me/code/grexa".into(),
    search_query: "TODO ai conversation".into(),
    filter_suggestions: grexa_ai::linux_suggestions_for(&search_options),
    regex_search: false,
    files_search: false,
};

// 4. Send a turn.
let response = client.send_chat(&config, &context, &[AiConversationTurn {
    role: AiRole::User,
    content: "Where should I look for AI conversation state?".into(),
}]);

if response.success {
    println!("{}", response.message);
}
```

See [ai-provider-scope.md](ai-provider-scope.md) for which servers
Grexa can talk to (any OpenAI-compatible endpoint).

## CLI output formats

| Format | Use case |
| ------ | -------- |
| `text` (default) | grep-compatible `path:line:col:content` |
| `json` | pretty-printed array of `SearchResult` objects (consume with `jq`) |
| `csv` | machine-readable; quoted/escaped per RFC 4180 |
| `--count` | total match count only |
| `--files-only` | one full-path per line, deduped + sorted |
| `--quiet` | no output; exit code carries the answer |

## Cancellation

- **CLI**: `Ctrl-C` triggers `CancelToken::cancel()`. The walker
  stops at the next entry; the in-progress file finishes its current
  64-line batch. Partial results print with a stderr notice.
- **GUI**: the Stop button drives the same token. The result table
  freezes at its current contents; the row count badge shifts to a
  "cancelled" state.

## History, profiles, recent paths

These are persisted under XDG paths and consumed by the GUI:

- `$XDG_CONFIG_HOME/grexa/settings.json`
- `$XDG_DATA_HOME/grexa/recent_paths.json`
- `$XDG_DATA_HOME/grexa/search_history.json`
- `$XDG_DATA_HOME/grexa/search_profiles.json`

The CLI doesn't read or write these — every CLI invocation is a
one-shot. Use the GUI to manage stored entries.

## Logging

`grexa-cli` writes structured logs to
`$XDG_STATE_HOME/grexa/grexa.log`. Override verbosity:

```bash
GREXA_LOG=debug grexa-cli ~/code TODO
GREXA_LOG=grexa_core::search=trace grexa-cli ~/code TODO
```

The GUI writes the same logs to the same file
(`$XDG_STATE_HOME/grexa/grexa-gui.log`); `GREXA_LOG` is honored
either way.

## Shell completions

```bash
# Bash
grexa-cli completions bash > ~/.local/share/bash-completion/completions/grexa-cli

# Zsh (writes the `_grexa-cli` file expected by compinit)
grexa-cli completions zsh > "${fpath[1]}/_grexa-cli"

# Fish
grexa-cli completions fish > ~/.config/fish/completions/grexa-cli.fish
```

## Man page

```bash
grexa-cli manpage | gzip -c > /usr/share/man/man1/grexa-cli.1.gz
```

Or run `just manpage` to land the file under `target/man/`.
