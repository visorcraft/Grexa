# Reference

Authoritative reference for the Grexa settings schema, CLI flags,
data paths, encoding labels, and keyboard shortcuts.

## CLI

```text
grexa-cli <path> <term> [OPTIONS]
grexa-cli completions <shell>
grexa-cli manpage
```

### Positional arguments

| Name | Meaning |
| ---- | ------- |
| `<path>` | Local directory or, with `--container`, an in-container path. |
| `<term>` | Plain text or regex; depends on `--regex`. |

### Flags

| Flag                       | Default     | Effect |
| -------------------------- | ----------- | ------ |
| `-E`, `--regex`            | off         | Treat `<term>` as a regex. |
| `-i`, `--case-sensitive`   | off         | Case-sensitive comparison. |
| `-g`, `--gitignore`        | off         | Respect `.gitignore`, `.ignore`, and global git excludes. |
| `-H`, `--include-hidden` (`--hidden`) | off | Include dotfiles and dot-directories. |
| `-b`, `--include-binary`   | off         | Include searchable binary/document formats (DOCX/PDF/RTF/…). |
| `-s`, `--include-system` (`--no-ignore`) | off | Include `.git`, `node_modules`, `/proc`, `/sys`, `/dev`, etc. |
| `-d`, `--no-subfolders`    | off         | Do not recurse. |
| `-L`, `--include-symlinks` | off         | Follow symbolic links. |
| `-m`, `--match-files <glob>` | empty     | Pipe/semicolon-separated globs, `-` prefix for exclusion. |
| `-x`, `--exclude-dirs <names-or-regex>` | empty | Names like `bin,obj`; or regex when `^`/`$`/`|` present without `,`/`;`. |
| `--size-limit <N>`         | none        | Size threshold value. |
| `--size-unit <kb|mb|gb>`   | `kb`        | Size unit. |
| `--size-type <less|equal|greater|none>` | `less` | Comparison. |
| `-f`, `--format <text|json|csv>` | `text` | Output format. |
| `-c`, `--count`            | off         | Print only the total match count. |
| `-l`, `--files-only`       | off         | Print matching file names (deduped, sorted). |
| `-q`, `--quiet`            | off         | No output; exit code carries the answer. |
| `--comparison <mode>`      | `ordinal`   | `ordinal` / `current-culture` / `invariant-culture`. |
| `--normalization <form>`   | `none`      | `none` / `form-c` / `form-d` / `form-kc` / `form-kd`. |
| `--ignore-diacritics`      | off         | Strip combining marks before comparison. |
| `--culture <locale>`       | none        | BCP-47 tag (e.g. `tr-TR`); active when `--comparison current-culture`. |
| `--use-index`              | off         | Allow Baloo candidate seeding (Phase 13 deferred). Conflicts with `--no-index`. |
| `--no-index`               | off         | Force the walker even when the setting enables Baloo. |
| `--container <id-or-name>` | none        | Search inside a container; positional `<path>` is interpreted in-container. |
| `--runtime <auto|docker|podman>` | `auto` | Used only with `--container`. |

### Exit codes

| Code | Meaning |
| ---- | ------- |
| `0`  | Matches found. |
| `1`  | No matches found. |
| `2`  | Error (missing path, malformed regex, runtime not available, …). |

### Subcommands

| Subcommand | Purpose |
| ---------- | ------- |
| `completions <bash|zsh|fish|elvish|powershell>` | Emit a shell completion script to stdout. |
| `manpage` | Emit a roff(7) man page to stdout. |

## Settings schema (`$XDG_CONFIG_HOME/grexa/settings.json`)

All fields below are persisted by `crates/grexa-core::DefaultSettings`.
Defaults match the `Default` impl.

| Key | Type | Default | Notes |
| --- | ---- | ------- | ----- |
| `regex_search` | bool | `false` | Default regex mode. |
| `files_search` | bool | `false` | Default Files-mode (vs. Content). |
| `respect_gitignore` | bool | `false` | Default `.gitignore` honoring. |
| `search_case_sensitive` | bool | `false` | Default case-sensitivity. |
| `include_system_files` | bool | `false` | Default include-system. |
| `include_subfolders` | bool | `true` | Default recursion. |
| `include_hidden_items` | bool | `false` | Default include-hidden. |
| `include_binary_files` | bool | `false` | Default include-binary. |
| `include_symbolic_links` | bool | `false` | Default follow-symlinks. |
| `use_file_index` | bool | `false` | Default Baloo seed flag. |
| `enable_container_search` | bool | `false` | Show container target dropdown. |
| `size_unit` | enum | `KB` | `KB` / `MB` / `GB`. |
| `theme_preference` | u8 | `0` | Grex integer encoding (0…11), plus Grexa-only `12` = OLED Black. |
| `ui_language` | string | `"en-US"` | BCP-47 tag fed to `Locale::from_tag`. |
| `string_comparison_mode` | enum | `Ordinal` | `Ordinal` / `CurrentCulture` / `InvariantCulture`. |
| `unicode_normalization_mode` | enum | `None` | `None` / `FormC` / `FormD` / `FormKC` / `FormKD`. |
| `diacritic_sensitive` | bool | `true` | Inverted by `--ignore-diacritics`. |
| `culture` | string | `"en-US"` | ICU/BCP-47 culture tag. |
| `default_match_files` | string | `""` | Default `--match-files`. |
| `default_exclude_dirs` | string | `""` | Default `--exclude-dirs`. |
| `content_*_column_visible` | bool | `true` | Content table column visibility (Line / Column / Path). |
| `files_*_column_visible` | bool | `true` | Files table column visibility (Size / Matches / Path / Ext / Encoding / Modified). |
| `window_width` | u32? | `1100` | GUI dimension; window position is intentionally not persisted. |
| `window_height` | u32? | `700` | |
| `context_preview_lines_before` | u8 | `5` | Clamp 1–20. |
| `context_preview_lines_after` | u8 | `5` | Clamp 1–20. |
| `ai_search_endpoint` | string | `"https://api.openai.com/v1"` | Trim on write. |
| `ai_search_model` | string | `"gpt-4o-mini"` | Trim on write. |
| `ai_search_enabled` | bool | `false` | Explicit opt-in gate. |

Import semantics from a Grex backup are documented in
[grex-storage-services-audit.md](grex-storage-services-audit.md).

## Data paths

| Path | Owner |
| ---- | ----- |
| `$XDG_CONFIG_HOME/grexa/settings.json` | settings |
| `$XDG_DATA_HOME/grexa/recent_paths.json` | recent paths (cap 20) |
| `$XDG_DATA_HOME/grexa/search_history.json` | search history (cap 20, 7-field dedupe) |
| `$XDG_DATA_HOME/grexa/search_profiles.json` | saved profiles |
| `$XDG_STATE_HOME/grexa/grexa.log` | tracing log |
| `$XDG_STATE_HOME/grexa/replace-journal.json` | crash-recovery journal (cleared on success) |
| `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-ts>/` | mirrored container paths |

API keys (any AI endpoint) live in the keyring under service id
`io.visorcraft.Grexa.ai`, account = canonical endpoint URL.

## Encoding labels

Grexa's `FileSearchResult.encoding` reports one of:

- `UTF-8`
- `UTF-8 BOM`
- `UTF-16 LE` / `UTF-16 BE`
- `UTF-32 LE` / `UTF-32 BE` (detect-only; decode falls back to lossy UTF-8)
- Any canonical `encoding_rs::Encoding::name()`, e.g. `windows-1252`,
  `Shift_JIS`, `EUC-KR`, `ISO-8859-1`, …

See [grex-encoding-detection-audit.md](grex-encoding-detection-audit.md).

## Keyboard shortcuts (GUI, Phase 4)

| Key | Action |
| --- | ------ |
| Enter | Trigger search (when the search input is focused). |
| Enter | Trigger replace (when the replacement input is focused). |
| Space | Open context preview for the selected result. |
| Escape | Close preview / cancel modal dialog. |
| F1 | Open About dialog. |
| Double-click | Open the result in the configured editor. |
| Ctrl-C (CLI) | Cancel the in-flight search. |

## Container CLI argv samples

What Grexa actually invokes (for reference / audit):

| Operation | argv |
| --------- | ---- |
| list | `<cli> ps --all --format=json` |
| has_grep | `<cli> exec <id> which grep` |
| direct grep | `<cli> exec <id> grep -rnH -F -i -- <pattern> <path>` |
| archive | `<cli> cp <id>:<path> <dest>` |

All flags are passed as argv arrays, never via a shell, so paths /
patterns with spaces / quotes / colons / globs / newlines are safe.

## Localization

- Catalog format: Fluent (`.ftl`).
- Default locale: `en` (canonical, source of truth).
- Bundled locales: `en`, `de`, `ja` (32 keys each at v1.0).
- Sync gate: `scripts/check_locale_sync.py` + a `cargo test`.

## Tracing log levels

| Level | What it surfaces |
| ----- | ---------------- |
| `error` | Fatal mismatches (e.g. AI HTTP transport error). |
| `warn` | Non-fatal anomalies (e.g. > 500k results). |
| `info` | Search start / complete + summary counts. |
| `debug` | Extractor failures, BOM detection details. |
| `trace` | Per-line scan decisions; only useful with `GREXA_LOG=trace`. |

Set via `GREXA_LOG=...`. Defaults to `warn` when unset.

## Container mirror lifecycle

- Created: per-search, under
  `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<id>/<unix-ts>/`.
- Pruned: `prune_mirrors(max_age_secs)` removes snapshots older than
  the threshold. The GUI calls this on startup; integrators should
  call it after each container search.
