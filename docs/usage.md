# Using Grexa

This guide covers the desktop workflow, safe replacement, container and
document search, optional AI features, and common CLI automation.

For exact flags, settings keys, paths, output fields, shortcuts, and safety
limits, use the [reference](reference.md).

## Desktop workflow

### Run a local search

1. Open `grexa`.
2. Enter a local directory in **Search path**, or use the folder picker.
3. Enter a literal term.
4. Optionally toggle **Regex**, **Case Sensitive**, or **Whole Word**.
5. Select **Content** or **Files** mode.
6. Select **Search** or press Enter in either search field.

Content mode shows one row per matching line. Files mode deduplicates the
visible rows by file and shows aggregate match counts.

The default search is recursive, case-insensitive, ordinal, and
diacritic-sensitive. It skips hidden paths, dependency/system directories,
binary files, and symlink traversal. It does not apply `.gitignore` unless
that option is enabled.

### Set filters

Open **Filters** on the Search page. Changes are auto-saved and apply to the
next search:

- respect `.gitignore`, `.ignore`, and global Git excludes;
- include hidden paths;
- include binary files and supported documents;
- include dependency/system paths;
- recurse into subdirectories;
- follow symbolic links;
- include or exclude file-name globs;
- exclude directories by names or regex.

The CLI additionally exposes file-size filtering, Unicode normalization,
culture-aware comparison, diacritic handling, regex-engine selection, and an
explicit result cap.

File patterns use `|` or `;` as separators. Prefix an individual pattern with
`-` to exclude it:

```text
*.rs|*.toml|*.md|-generated*
```

Directory exclusions accept a comma/semicolon name list:

```text
.git,node_modules,target,.venv
```

A value containing `^`, `$`, or `|`, without commas or semicolons, is treated
as a regex against root-relative directory paths.

### Work with results

- Select a row and press Space to open context preview.
- Press Enter on a selected row to open it in the configured editor.
- Click a row for context preview.
- Right-click for open, reveal, trash, and copy actions.
- Select a column header to sort.
- Enter text in **Filter results** to refine loaded results without scanning
  the filesystem again. Toggle its regex button for a regular-expression
  filter. An invalid filter regex fails closed and shows no rows.
- Export the visible rows, after result filtering and Files-mode deduplication,
  as CSV, JSON, or Markdown.

Configure editor presets, a custom editor command, and preview lines under
Settings. A custom editor command may contain `{path}`, `{file}`, and `{line}`
placeholders; see [Reference: editor settings](reference.md#settings-schema).

### Use search tabs

Select `+` or press Ctrl+T to open a tab. Each tab keeps its form, visible
mode, result filter, counters, and result snapshot during the current process.
Ctrl+W closes the active tab.

Grexa permits eight in-session tabs. Inactive snapshots share a bounded result
budget; an old large snapshot may be evicted. Search profiles are the
persistent alternative.

### Use history and profiles

Every completed GUI search is recorded in **History**, with a maximum of 20
entries. Opening a history item restores its fields on the Search page but
does not automatically run the search.

Use **Save Profile** on the Search page to store a named path, term, regex,
case, and result-mode configuration. Opening a profile restores it. Profile
names are matched case-insensitively when updating or deleting.

Recent paths, history, and profiles are plain grexa-db Markdown records under
`$XDG_DATA_HOME/grexa/db/`.

### Test a regex

Open **Regex Builder** to test a pattern against sample text. It shows compile
errors, match count, highlighted ranges, and individual matches. Simple
patterns use the fast engine; lookaround and backreferences require the
extended engine.

## Search from the CLI

### Basic search

```bash
grexa-cli ~/code/grexa TODO
```

Text output is:

```text
path:line:column:content
```

Exit code `0` means matches, `1` means no matches, and `2` means an error.

### Filter file names and directories

```bash
grexa-cli ~/code TODO \
  --match-files '*.rs|*.toml|*.md|-generated*' \
  --exclude-dirs '.git,node_modules,target'
```

### Respect ignore files

```bash
grexa-cli ~/code TODO --gitignore
```

`--include-system` controls Grexa's built-in dependency/system exclusions. It
does not disable `--gitignore`; omit `--gitignore` if ignore-file rules should
not apply.

### Regex and engine selection

```bash
# Auto-select fast or extended engine
grexa-cli ~/code 'fn\s+(\w+)\s*\(' --regex

# Fail unless the fast engine supports the pattern
grexa-cli ~/code 'fn\s+\w+' --regex --regex-engine fast

# Require lookbehind support
grexa-cli ~/code '(?<=prefix)\w+' --regex --regex-engine extended
```

The extended engine is bounded, but complex backtracking patterns are still
slower. Narrow the path and file globs first.

### Unicode-aware comparison

```bash
grexa-cli ~/notes 'café' \
  --comparison invariant-culture \
  --normalization form-c \
  --ignore-diacritics
```

Use `--comparison current-culture --culture tr-TR` when locale-specific case
mapping matters. These options apply to literal search. Regex behavior is
controlled by the selected regex engine.

### Bound broad searches

```bash
grexa-cli ~/code error --max-results 10000
```

A user-supplied limit stops cleanly after that many result rows. Without one,
the engine applies its 1,000,000-row hard ceiling.

## Search documents

Enable binary/document search:

```bash
grexa-cli ~/Documents 'Q3 forecast' --include-binary
```

Supported extraction:

| Format | Behavior |
| ------ | -------- |
| DOCX | Text from `word/document.xml` |
| XLSX | Shared strings and first comments part |
| PPTX | Text from slide XML |
| ODT / ODS / ODP | Text from `content.xml` |
| ZIP | Entry names and recognized textual entries |
| RTF | Best-effort visible-text extraction |
| PDF | `pdftotext` subprocess |

Results point to the original container document. Extracted documents are
search-only and are not rewritten by replace.

PDF search requires `pdftotext` from Poppler. Extraction errors skip the
affected document and appear in debug logs.

## Replace

### Desktop replace

1. Run the exact search that should define the rewrite scope.
2. Review its filters and results.
3. Select **Replace**.
4. Enter the replacement text. Regex searches accept `$1`, `$name`, and
   `${name}` captures.
5. Confirm **Replace All**.

Replace reuses the stored options from the last completed search. Later
settings changes cannot silently widen that rewrite. On success, Grexa switches
to Files mode and shows the summary.

There is no content undo. Use version control, a filesystem snapshot, or a
backup when rollback matters.

### CLI replace

Preview first:

```bash
grexa-cli replace ~/code 'old_(\w+)' 'new_$1' --regex --dry-run
```

Apply:

```bash
grexa-cli replace ~/code 'old_(\w+)' 'new_$1' --regex
```

Replace accepts the same search-behavior flags as local search. It performs
encoding-aware whole-buffer substitution, writes a temporary file in the same
directory, atomically persists it, and restores permissions.

Replace exit codes:

- `0`: at least one file modified and no failures;
- `1`: no files modified;
- `2`: validation, I/O, or partial failure.

If a process is interrupted, the residual journal at
`$XDG_STATE_HOME/grexa/replace-journal.json` lists files already modified. It
does not contain their previous content. The GUI can surface that journal at
the next launch.

## Container search

### Enable the desktop target selector

Enable **Settings → Containers → Enable container search**. The Search page
then probes Docker and Podman off the GUI thread and lists running containers.
Choose a container target before searching.

### Search from the CLI

```bash
grexa-cli /etc/nginx TODO --container web --runtime podman
```

The container value may be an exact ID, exact name, or unambiguous ID prefix
from the running-container list. The positional path is interpreted inside the
container.

Container search honors these matching options:

- literal or regex mode;
- case sensitivity;
- whole-word matching;
- maximum results;
- normalization, comparison, diacritic, and culture controls.

Local-walk flags such as `.gitignore`, hidden paths, symlinks, file globs,
directory exclusions, and size filters do not apply to the direct container
path. Container output is text; `--count` and `--files-only` work, while
`--format` and `--quiet` currently affect local searches only.

Grexa first tries `grep` inside the container. If no `grep` exists, it copies
the requested path to:

```text
$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<container-id>/<unix-time>/
```

and searches the mirror locally. Displayed paths are translated back to
container paths. The CLI prints a fallback notice on stderr.

Mirrors older than one hour are pruned by GUI maintenance and after CLI
container searches. Library users can call
`grexa_containers::prune_mirrors(max_age_secs)`.

Container replacement exists only as an internal library path. The GUI and CLI
do not expose it.

The Flatpak intentionally does not expose Docker or Podman sockets, so
container search is unavailable there. Use a native package, AppImage, or
compatible tar build. Flatpak local-file access is limited to the home
directory, `/run/media`, and locations granted through the desktop portal.

## AI Search

### Configure

1. Open **Settings → AI Search**.
2. Enable the AI panel.
3. Enter the base URL of an OpenAI-compatible endpoint.
4. Optionally set a model. A blank model uses the first ID returned by
   `GET /v1/models`, then falls back to `gpt-4o-mini`.
5. Save an API key if the endpoint needs one.
6. Test the endpoint.

Keys are stored in the Linux Secret Service. There is no plaintext fallback.
Remote credentials are sent only over HTTPS. Plain HTTP credentials are
allowed only for loopback hosts such as `localhost`.

### Chat and summarize

The AI drawer supports two request types:

- A chat message sends that single user message plus the current search
  path/query/modes/filter suggestions. The visible message list is local UI
  history; previous turns are not included in later requests.
- **Summarize results** sends a fixed summarization instruction plus bounded
  excerpts from up to 400 visible result rows, with surrounding lines and
  `path:line` labels. The excerpt budget is configurable from 2,000 to 40,000
  characters.

Grexa discloses when the visible-row cap or excerpt budget omitted matches.
The response is advisory and never changes files.

See [AI provider scope](ai-provider-scope.md) and
[Security and privacy](SECURITY.md#outbound-traffic).

## Database browser

Open **Tools → Database** and enter a grexa-db root directory. For Grexa's own
data, use:

```text
$XDG_DATA_HOME/grexa/db
```

The browser can:

- list collections and schema fields;
- show up to 500 record paths;
- inspect record frontmatter;
- filter typed fields with `eq`, `ne`, `lt`, `le`, `gt`, `ge`, or `contains`;
- validate every record against the collection schema;
- materialize a query as a filesystem view of symlinks, optionally grouped by
  a field;
- list and delete materialized views.

Secondary indexes are derived sidecars. Queries fall back to scans when an
index is absent, and a filesystem watcher reconciles out-of-band edits.

## CLI output and scripting

| Output | Shape |
| ------ | ----- |
| text | `path:line:column:content`, one matching line per row |
| JSON | pretty-printed array of `SearchResult` objects |
| CSV | header plus RFC 4180-style escaped rows |
| `--count` | one integer |
| `--files-only` | sorted, deduplicated full paths |
| `--quiet` | no local-search output; exit code only |

Example:

```bash
grexa-cli ~/code TODO --format json |
  jq -r '.[] | "\(.full_path):\(.line_number)"'
```

CSV values beginning with spreadsheet formula characters are prefixed with a
single quote to prevent formula execution when opened in office software.
Terminal control characters are replaced when output goes directly to a TTY;
piped output remains unchanged.

## Cancellation

- CLI: Ctrl-C requests cooperative cancellation. Partial local results print
  with a notice.
- GUI: **Stop** or Escape cancels the active search. Rows already received
  remain visible.
- Switching tabs while a search runs also cancels that search before restoring
  the next tab.

## Logs

CLI:

```text
$XDG_STATE_HOME/grexa/grexa.log
```

GUI, daily rotation with at most two files:

```text
$XDG_STATE_HOME/grexa/grexa-gui.<date>.log
```

Set `GREXA_LOG` with a `tracing-subscriber` filter:

```bash
GREXA_LOG=debug grexa-cli ~/code TODO
GREXA_LOG=info,grexa_core=trace grexa
```

The CLI defaults to `warn`; the GUI defaults to `info`.

## Shell integration

```bash
# Bash
grexa-cli completions bash \
  > ~/.local/share/bash-completion/completions/grexa-cli

# Zsh
grexa-cli completions zsh > "${fpath[1]}/_grexa-cli"

# Fish
grexa-cli completions fish \
  > ~/.config/fish/completions/grexa-cli.fish

# Man page
grexa-cli manpage | gzip -c | sudo tee /usr/share/man/man1/grexa-cli.1.gz >/dev/null
```

Repository helpers write generated files under `target/`:

```bash
just completions
just manpage
```
