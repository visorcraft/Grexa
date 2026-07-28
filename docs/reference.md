# Grexa Reference

This is the authoritative public reference for CLI syntax, output formats,
settings, storage, keyboard shortcuts, environment variables, integrations,
and enforced resource limits.

## CLI

### Syntax

```text
grexa-cli [OPTIONS] <PATH> <TERM>
grexa-cli replace [OPTIONS] <PATH> <TERM> <REPLACEMENT>
grexa-cli completions <SHELL>
grexa-cli manpage
```

`<PATH>` is a local directory. With `--container`, it is a path inside the
selected running container. `<TERM>` is literal text unless `--regex` is set.

Run the generated help for the installed version:

```bash
grexa-cli --help
grexa-cli replace --help
```

### Matching and traversal flags

| Flag | Default | Effect |
| ---- | ------- | ------ |
| `-E`, `--regex` | off | Treat `<TERM>` as a regular expression. |
| `--regex-engine <auto|fast|extended>` | `auto` | Select the regex engine. `auto` tries the fast engine first and falls back to the extended engine. |
| `-i`, `--case-sensitive` | off | Use case-sensitive matching. The short flag follows Grex compatibility, not grep's `-i` meaning. |
| `-w`, `--whole-word` | off | Require both adjacent characters to be neither alphanumeric nor `_`. Grexa does not use regex `\b` for this option. |
| `-g`, `--gitignore` | off | Apply `.gitignore`, `.ignore`, and global Git excludes. A `.git/` directory is not required. |
| `-H`, `--include-hidden`, `--hidden` | off | Include dotfiles and dot-directories. |
| `-b`, `--include-binary` | off | Include supported searchable documents and allowed binary extensions. |
| `-s`, `--include-system`, `--no-ignore` | off | Include Grexa's built-in dependency/system exclusions. This does not negate `--gitignore`. |
| `-d`, `--no-subfolders` | off | Search only the root directory. |
| `-L`, `--include-symlinks` | off | Follow symbolic links. |
| `-m`, `--match-files <PATTERNS>` | empty | Include/exclude file-name globs separated by `|` or `;`; prefix exclusions with `-`. |
| `-x`, `--exclude-dirs <VALUE>` | empty | Exclude directory names separated by comma/semicolon, or use one regex. |
| `--size-limit <N>` | none | Set a file-size threshold. |
| `--size-unit <kb|mb|gb>` | `kb` | Unit for `--size-limit`. |
| `--size-type <less|equal|greater|none>` | `less` | Comparison for `--size-limit`; no effect when no limit is supplied. |
| `--max-results <N>` | none | Stop after N result rows. A supplied value is normal truncation, not an internal-cap warning. |

File-pattern example:

```text
*.rs|*.toml|*.md|-generated*
```

Directory exclusions containing `^`, `$`, or `|`, and no comma or semicolon,
are interpreted as a regex against the path after the search root has been
removed. Other values are exact directory-name lists.

### Text comparison flags

| Flag | Default | Effect |
| ---- | ------- | ------ |
| `--comparison <ordinal|current-culture|invariant-culture>` | `ordinal` | Literal string-comparison mode. |
| `--normalization <none|form-c|form-d|form-kc|form-kd>` | `none` | Normalize the haystack and needle before literal comparison. |
| `--ignore-diacritics` | off | Remove combining marks before literal comparison. |
| `--culture <BCP-47>` | none | ICU locale tag used by `current-culture`; ignored by other modes. |

Case-insensitive and normalized matches are mapped back to original UTF-8
offsets by grapheme segment. Reported columns and replacement spans always
refer to the original content.

### Local-search output flags

| Flag | Default | Effect |
| ---- | ------- | ------ |
| `-f`, `--format <text|json|csv>` | `text` | Select output encoding. |
| `-c`, `--count` | off | Print total matching-line count only. |
| `-l`, `--files-only` | off | Print sorted, deduplicated full paths. |
| `-q`, `--quiet` | off | Print nothing; use the exit status. |

`--count`, `--files-only`, and `--quiet` take precedence over `--format` in
that order.

### Container flags

| Flag | Default | Effect |
| ---- | ------- | ------ |
| `--container <ID-OR-NAME>` | none | Search inside a running container. Accepts exact ID, exact name, or ID prefix. |
| `--runtime <auto|docker|podman>` | `auto` | Select the runtime; requires `--container`. |

Container search currently honors regex, regex mode, case sensitivity,
whole-word, maximum results, normalization, comparison, diacritic, and culture
settings. Direct container search does not apply local walker flags such as
`.gitignore`, hidden paths, symlinks, globs, excluded directories, or size
limits.

Container output is text. `--count` and `--files-only` work. `--format` and
`--quiet` are accepted by the shared command surface but currently affect
local search only.

### Baloo compatibility flags

| Flag | Current behavior |
| ---- | ---------------- |
| `--use-index` | Sets `SearchOptions.use_file_index`; conflicts with `--no-index`. |
| `--no-index` | Clears `SearchOptions.use_file_index`. |

The Baloo adapter exists, but candidate seeding is not wired into
`search_with`. Both flags therefore use the ordinary filesystem walker in the
current release. See [Baloo spike](baloo-spike.md).

### Replace

`replace` accepts every matching, traversal, text-comparison, and Baloo flag
listed above. It does not accept output or container flags.

| Replace-only flag | Default | Effect |
| ----------------- | ------- | ------ |
| `--dry-run` | off | Run the search and print affected lines without writing files. |

Regex replacements expand `$1`, `$name`, and `${name}` against captures
re-queried from the full haystack. A literal dollar can be handled according
to the selected regex engine's replacement syntax.

Dry-run stdout is:

```text
path:line:content
```

Applied replacement stdout is:

```text
path: N replacements
```

The summary and failures go to stderr.

### Utility subcommands

| Subcommand | Effect |
| ---------- | ------ |
| `completions <bash|elvish|fish|powershell|zsh>` | Write a completion script to stdout. |
| `manpage` | Write a roff man page to stdout. |

Repository helpers place the common generated files under `target/`:

```bash
just completions
just manpage
```

### Exit codes

Search:

| Code | Meaning |
| ---- | ------- |
| `0` | At least one match. |
| `1` | No matches. |
| `2` | Invalid input, I/O failure, invalid regex, unavailable runtime, or another error. |

Replace:

| Code | Meaning |
| ---- | ------- |
| `0` | At least one file modified and no failures. |
| `1` | No file modified. |
| `2` | Validation failure, I/O failure, or one or more per-file failures. |

## Output formats

### Text

Local search emits one row per matching line:

```text
<full-path>:<line>:<column>:<content>
```

Line and column are 1-based. Container output omits the column:

```text
<container-path>:<line>:<content>
```

Control characters are replaced with U+FFFD when stdout is a terminal to
prevent terminal escape injection. Redirected and piped output is left
unchanged.

### JSON

JSON output is a pretty-printed array. Each item has:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `file_name` | string | Base file name. |
| `line_number` | integer | 1-based line. |
| `column_number` | integer | 1-based Unicode character column. |
| `line_content` | string | Truncated source-line preview. |
| `match_preview_before` | string | Text before the highlighted span. |
| `match_preview_match` | string | Highlighted span. |
| `match_preview_after` | string | Text after the span. |
| `full_path` | string | Full filesystem path. |
| `relative_path` | string | Path relative to the search root. |
| `match_count` | integer | Matches collected on that line. |

### CSV

The header is:

```text
File,Line,Column,Content,FullPath,MatchCount
```

Fields follow CSV escaping rules. Values beginning with `=`, `+`, `-`, `@`,
tab, CR, or LF are prefixed with `'` to prevent spreadsheet formula
execution.

## Resource bounds

| Scope | Limit | Behavior |
| ----- | ----- | -------- |
| CLI search term | 4,096 characters | Rejected before search. |
| File read by search, preview, or replace | 512 MiB | File is skipped or rejected. |
| Result rows without `--max-results` | 1,000,000 | Search stops collecting; `SearchSummary.capped` is true. |
| Matches on one source line | 10,000 | Later matches are dropped and a warning is logged. |
| Bytes compared on one line | 2 MiB | Later bytes on that line are not matched; warned once per file. |
| Extended-regex work per line | 100 ms | That line stops scanning. |
| Extended regex backtrack limit | 100,000 | Pattern operation fails when exhausted. |
| Filesystem recursion depth | 64 | Deeper directories are not descended. |
| Searchable ZIP entries | 1,024 | Document is skipped when exceeded. |
| One textual archive entry | 4 MiB | Document is skipped when exceeded. |
| Extracted document text | 16 MiB | Document is skipped when exceeded. |
| PDF extraction | 30 s | `pdftotext` is killed. |
| Files in one replace | 100,000 | File list is truncated; `ReplaceSummary.capped` is true. |
| Matches collected in one replaced file | 1,000,000 | Later spans are not collected. |
| Extended-regex replace work per file | 5 s | That file fails without being rewritten. |
| Container command | 30 s | Process group is killed and a timeout is returned. |
| Container stdout or stderr | 10 MiB per stream | Process group is killed and an output-limit error is returned. |
| GUI search tabs | 8 | New tabs are refused at the cap. |
| GUI active result rows | 1,000,000 | Additional rows are not retained. |
| GUI inactive snapshot rows | 2,000,000 total | Old inactive snapshots are evicted to make room. |
| AI evidence source rows | 400 | Later visible rows are omitted with a disclosure. |
| AI evidence budget | 2,000 to 40,000 characters | Excerpts are packed breadth-first within the configured value. |
| AI response body | 4 MiB | Request fails. |
| AI request | 90 s | Transport times out. |

Document and search limits are safety ceilings, not promises that every input
below a ceiling is cheap. Use file globs and `--max-results` for broad roots.

## Settings schema

Settings live at:

```text
$XDG_CONFIG_HOME/grexa/settings.json
```

If `XDG_CONFIG_HOME` is unset, the default is
`$HOME/.config/grexa/settings.json`. Missing and unknown fields are accepted;
missing fields take their defaults. Writes use a same-directory temporary file
and atomic persist.

### Search and appearance

| Key | Type | Default | Meaning |
| --- | ---- | ------- | ------- |
| `regex_search` | bool | `false` | New-tab regex mode. |
| `files_search` | bool | `false` | New-tab Files mode. |
| `respect_gitignore` | bool | `false` | Apply ignore files. |
| `search_case_sensitive` | bool | `false` | Case-sensitive default. |
| `include_system_files` | bool | `false` | Include built-in dependency/system exclusions. |
| `include_subfolders` | bool | `true` | Recursive search. |
| `include_hidden_items` | bool | `false` | Include dot paths. |
| `include_binary_files` | bool | `false` | Search documents/binary allowlist. |
| `include_symbolic_links` | bool | `false` | Follow symlinks. |
| `use_file_index` | bool | `false` | Reserved Baloo setting; currently no search effect. |
| `enable_container_search` | bool | `false` | Show and probe container targets. |
| `size_unit` | enum | `"KB"` | Persisted display unit: `KB`, `MB`, or `GB`. |
| `theme_preference` | integer | `0` | Theme ID; see the table below. |
| `ui_language` | string | `"en-US"` | Locale tag; `en`, `de`, and `ja` are recognized. |
| `string_comparison_mode` | enum | `"Ordinal"` | `Ordinal`, `CurrentCulture`, or `InvariantCulture`. |
| `unicode_normalization_mode` | enum | `"None"` | `None`, `FormC`, `FormD`, `FormKC`, or `FormKD`. |
| `diacritic_sensitive` | bool | `true` | Preserve diacritics during literal comparison. |
| `culture` | string | `"en-US"` | ICU/BCP-47 culture tag. |
| `default_match_files` | string | `""` | Default file patterns. |
| `default_exclude_dirs` | string | `""` | Default directory exclusions. |

Theme IDs:

| ID | Theme | ID | Theme |
| -- | ----- | -- | ----- |
| `0` | System | `7` | Paranoid |
| `1` | Light | `8` | Red Velvet |
| `2` | Dark | `9` | Subspace |
| `3` | Gentle Gecko | `10` | Tiefling |
| `4` | Black Knight | `11` | Vibes |
| `5` | Diamond | `12` | OLED Black |
| `6` | Dreams | | |

### Result columns and window

| Key | Type | Default | Meaning |
| --- | ---- | ------- | ------- |
| `content_line_column_visible` | bool | `true` | Show Content line column. |
| `content_column_column_visible` | bool | `true` | Show Content character column. |
| `content_path_column_visible` | bool | `true` | Show Content path column. |
| `files_size_column_visible` | bool | `true` | Show Files size column. |
| `files_matches_column_visible` | bool | `true` | Show Files match-count column. |
| `files_path_column_visible` | bool | `true` | Show Files path column. |
| `files_ext_column_visible` | bool | `true` | Show Files extension column. |
| `files_encoding_column_visible` | bool | `true` | Show Files encoding column. |
| `files_date_modified_column_visible` | bool | `true` | Show Files modified-time column. |
| `window_width` | integer or null | `1100` | Saved width; imported values below 400 are ignored. |
| `window_height` | integer or null | `700` | Saved height; imported values below 400 are ignored. |
| `context_preview_lines_before` | integer | `5` | Context lines before, clamped to 1 through 20. |
| `context_preview_lines_after` | integer | `5` | Context lines after, clamped to 1 through 20. |

### AI, editor, replace, privacy, and accessibility

| Key | Type | Default | Meaning |
| --- | ---- | ------- | ------- |
| `ai_search_endpoint` | string | `"https://api.openai.com/v1"` | OpenAI-compatible base URL; trimmed on save/import. |
| `ai_search_model` | string | `"gpt-4o-mini"` | Model ID; blank enables discovery. |
| `ai_summary_budget_chars` | integer | `12000` | Evidence budget, clamped to 2,000 through 40,000. |
| `ai_search_enabled` | bool | `false` | Mandatory opt-in gate for AI requests. |
| `editor_preset` | integer | `8` | Editor ID; see below. |
| `editor_custom_command` | string | `""` | Overrides the preset. Supports `{path}`, `{file}`, and `{line}` argv placeholders. |
| `replace_confirm` | bool | `true` | Show replace confirmation. |
| `replace_show_journal_on_startup` | bool | `true` | Surface a residual journal after interrupted replace. |
| `privacy_redact_paths` | bool | `false` | Replace the GUI log's `$HOME` prefix with `~`; stderr and CLI logs are unchanged. |
| `accessibility_reduced_motion` | bool | `false` | Collapse configured animation durations and stop decorative busy motion. |
| `accessibility_high_contrast` | bool | `false` | Increase token contrast for the selected theme. |

Editor IDs:

| ID | Program |
| -- | ------- |
| `0` | Kate |
| `1` | KWrite |
| `2` | Visual Studio Code |
| `3` | VSCodium |
| `4` | Sublime Text |
| `5` | JetBrains `idea` launcher |
| `6` | GNOME Text Editor |
| `7` | Neovim |
| `8` | `xdg-open` |

The custom command is split into an argv vector without a shell. Single and
double quoted tokens are recognized; command substitution, variable
expansion, and shell operators are not.

API keys are never settings fields. They live in the Secret Service under:

```text
service = com.visorcraft.Grexa.ai
account = <canonical endpoint base URL>
```

## Data paths

All paths honor XDG overrides:

| Path | Owner and lifecycle |
| ---- | ------------------- |
| `$XDG_CONFIG_HOME/grexa/settings.json` | Atomic GUI settings. |
| `$XDG_DATA_HOME/grexa/db/recent_paths/` | Up to 20 grexa-db records, exact case-sensitive dedupe. |
| `$XDG_DATA_HOME/grexa/db/search_history/` | Up to 20 grexa-db records, newest first. |
| `$XDG_DATA_HOME/grexa/db/search_profiles/` | Named grexa-db profile records. |
| `$XDG_DATA_HOME/grexa/db/<collection>/schema.md` | Collection schema. |
| `$XDG_DATA_HOME/grexa/db/.grexa-index/` | Rebuildable grexa-db secondary indexes. |
| `$XDG_DATA_HOME/grexa/*.json.bak` | Legacy Grexa JSON stores renamed after successful first-use migration. |
| `$XDG_STATE_HOME/grexa/grexa.log` | Appended CLI log. |
| `$XDG_STATE_HOME/grexa/grexa-gui.<date>.log` | Daily GUI log, at most today plus one archive. |
| `$XDG_STATE_HOME/grexa/replace-journal.json` | Modified/failed paths for an in-progress replace; cleared after clean completion. |
| `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-time>/` | Temporary container filesystem mirrors. |
| `$XDG_RUNTIME_DIR/grexa/grexa.lock` | Best-effort GUI single-instance lock; falls back to `$XDG_CACHE_HOME/grexa/grexa.lock`. |

Defaults when an XDG variable is unset:

| Variable | Default |
| -------- | ------- |
| `XDG_CONFIG_HOME` | `$HOME/.config` |
| `XDG_DATA_HOME` | `$HOME/.local/share` |
| `XDG_CACHE_HOME` | `$HOME/.cache` |
| `XDG_STATE_HOME` | `$HOME/.local/state` |
| `XDG_RUNTIME_DIR` | No general default; the GUI lock falls back to the cache directory. |

## Encoding labels

`FileSearchResult.encoding` can report:

- `UTF-8`
- `UTF-8 BOM`
- `UTF-16 LE`
- `UTF-16 BE`
- `UTF-32 LE`
- `UTF-32 BE`
- an `encoding_rs` canonical name such as `windows-1252`, `Shift_JIS`,
  `EUC-KR`, or `ISO-8859-1`

UTF-32 is detected but not decoded as UTF-32; reading falls back to lossy
UTF-8. Other malformed input uses replacement characters rather than
panicking.

## Desktop keyboard and pointer actions

| Input | Action |
| ----- | ------ |
| F1 | Open About. |
| Ctrl+, or Ctrl+3 | Open Settings. |
| Ctrl+1 | Open Search. |
| Ctrl+2 | Open Regex Builder. |
| Ctrl+4 | Open History. |
| Ctrl+5 | Open Profiles. |
| Ctrl+T | Open a search tab, or return to Search and open one. |
| Ctrl+W | Close the active search tab. |
| Ctrl+L | Open Search and focus the search-term field. |
| Ctrl+F | Open Search and focus the loaded-result filter. |
| Ctrl+Q | Quit. |
| Escape | Cancel a running search; otherwise use Qt's dialog/drawer close behavior. |
| Enter in path/term | Start search. |
| Enter in replacement | Start Replace All. |
| Space on selected result | Open context preview. |
| Enter on selected result | Open in configured editor. |
| Click result | Open context preview. |
| Right-click result | Open result action menu. |

Ctrl-C requests cooperative cancellation in the CLI.

## Environment variables

| Variable | Use |
| -------- | --- |
| `GREXA_LOG` | `tracing-subscriber` filter, for example `info,grexa_core=debug`. |
| `HOME` | Fallback root for XDG paths, `~` expansion in Database, and privacy redaction. |
| `XDG_CONFIG_HOME` | Settings root. |
| `XDG_DATA_HOME` | grexa-db root. |
| `XDG_CACHE_HOME` | Container mirror root. |
| `XDG_STATE_HOME` | Logs and replace journal root. |
| `XDG_RUNTIME_DIR` | Single-instance lock and rootless Podman detection. |
| `DOCKER_HOST` | Docker runtime detection. |
| `WAYLAND_DISPLAY` | Select `wl-copy`; otherwise clipboard falls back to `xclip`. |
| `PATH` | External helper, runtime, editor, and desktop-integration lookup. |
| `QMAKE` | Build-time qmake selection; use `qmake6` in mixed Qt 5/6 environments. |
| `QML2_IMPORT_PATH` | AppImage troubleshooting only; a healthy bundle does not require it. |
| `QT_QPA_PLATFORM` | Qt platform selection, commonly `offscreen` for GUI smoke tests. |

## External programs

Grexa starts programs directly with argv arrays, never through a shell:

| Program | Purpose | Required? |
| ------- | ------- | --------- |
| `pdftotext` | PDF extraction | Optional |
| `docker`, `podman` | Container search | Optional |
| `baloosearch6`, `baloosearch`, or `baloo-search` | Deferred Baloo adapter | Optional, currently unused by search |
| configured editor or `xdg-open` | Open a result | Optional |
| `gdbus` | File-manager reveal and existing-instance activation | Optional, fallback available |
| `gio` | Move a result to Trash | Optional |
| `wl-copy` or `xclip` | Result clipboard actions | Optional |
| `notify-send` | Search completion notifications | Optional |
| `wmctrl` | Existing-instance activation fallback | Optional |
| `kbuildsycoca6`, `update-desktop-database`, `gtk-update-icon-cache` | Best-effort user desktop integration refresh | Optional |

## Package-specific constraints

| Package | Constraint |
| ------- | ---------- |
| AppImage | Bundles Qt/QML/Kirigami pieces, but optional helpers such as `pdftotext`, container CLIs, editors, and desktop tools still come from the host `PATH`. |
| Flatpak | Filesystem access is home, `/run/media`, and portal grants. Docker/Podman sockets are not exposed, so container search is unavailable. |
| Native Arch/Debian/RPM | Use the package built for the matching distribution release. cxx-qt QML AOT output links against Qt private ABI for that Qt minor version. |
| Release tarball | Built on the Arch release job and dynamically linked. The host needs a compatible Qt/Kirigami/glibc stack; use AppImage for a self-contained GUI. |

## Container mirror lifecycle

- Created only when the selected container lacks usable `grep`.
- Stored under
  `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-time>/`.
- Display paths are rewritten back to the original container path.
- CLI container search prunes mirrors older than one hour after each run.
- The GUI prunes on startup and periodically.
- Manual/library cleanup:
  `grexa_containers::prune_mirrors(max_age_secs)`.
