# Migrating from Grex

Grexa imports Grex's settings.json, search history, profiles, and
recent paths. This doc records what gets translated, what gets
dropped, and how to perform the import.

## What's on disk in Grex

A typical Grex install on Windows persists four files under
`%LocalAppData%\Grex\`:

```
%LocalAppData%\Grex\
├── settings.json
├── search_path_history.json
├── search_history.json
└── search_profiles.json
```

Copy that directory to your Linux box (any path you can read; the
importer accepts a directory argument).

## What ports verbatim

| Grex setting | Grexa setting |
| ------------ | ------------- |
| `IsRegexSearch` | `regex_search` |
| `IsFilesSearch` | `files_search` |
| `RespectGitignore` | `respect_gitignore` |
| `SearchCaseSensitive` | `search_case_sensitive` |
| `IncludeSystemFiles` | `include_system_files` |
| `IncludeSubfolders` | `include_subfolders` |
| `IncludeHiddenItems` | `include_hidden_items` |
| `IncludeBinaryFiles` | `include_binary_files` |
| `IncludeSymbolicLinks` | `include_symbolic_links` |
| `SizeUnit` | `size_unit` |
| `UILanguage` | `ui_language` (if non-empty) |
| `StringComparisonMode` | `string_comparison_mode` |
| `UnicodeNormalizationMode` | `unicode_normalization_mode` |
| `DiacriticSensitive` | `diacritic_sensitive` |
| `Culture` | `culture` (if non-empty) |
| `DefaultMatchFiles` | `default_match_files` |
| `DefaultExcludeDirs` | `default_exclude_dirs` |
| `Content*ColumnVisible` | `content_*_column_visible` |
| `Files*ColumnVisible` | `files_*_column_visible` |
| `ContextPreviewLines*` | `context_preview_lines_*` (clamped to 1–20) |
| `WindowWidth`/`WindowHeight` | `window_width`/`window_height` (when ≥ 400px each) |
| `AiSearchEndpoint`/`AiSearchModel` | trimmed and saved |
| `ThemePreference` | `theme_preference` (integer round-trip preserved) |

## What gets translated

| Grex setting | Translation |
| ------------ | ----------- |
| `UseWindowsSearchIndex` | → `use_file_index` (Linux Baloo equivalent; defaults are off) |
| `EnableDockerSearch` | → `enable_container_search` (covers both Docker and Podman) |
| `AiSearchApiKey` | **routed through the keyring**, never stored in `settings.json`; if the keyring is unavailable, the importer flags the key for the user instead of falling back to plaintext |
| `WindowX`/`WindowY` | dropped (window position is left to the window manager on Linux) |

## What gets dropped

These fields are Windows-only or don't translate. They're ignored
silently:

- Anything WSL-related: `\\wsl$`, `\\wsl.localhost`, `/mnt/<drive>/…`
  paths in recent-paths or profiles. Marked unavailable rather than
  deleted, so the user can curate.
- UNC paths (`\\server\share\…`) — same treatment.
- Windows drive-letter paths (`C:\Users\…`) — the importer offers a
  one-time `C:\Users\<u>` → `$HOME` remap.
- Windows toast / system tray / WinUI-specific settings.
- `Properties.AssemblyInfo` style metadata.

See [linux-decisions.md](linux-decisions.md) for the full Windows-vs-
Linux divergence table.

## How history and profiles port

- **Recent paths**: cap 20, deduped case-sensitively, newest first.
  Drive-letter and UNC entries are kept but tagged unavailable for
  the user's review.
- **Search history**: cap 20. Grex's 7-field dedupe key
  (`SearchTerm | SearchPath | IsRegexSearch | IsFilesSearch | SearchCaseSensitive | MatchFileNames | ExcludeDirs`)
  is preserved byte-for-byte (including the C# `True`/`False`
  capitalization). This means a history row imported from Grex and a
  fresh search performed in Grexa with the same parameters collapse
  into one entry, matching the Grex behavior.
- **Search profiles**: case-insensitive name comparison preserved.
  Insert-at-top (Grex's `AddOrUpdateProfile` behavior) replaces
  Grexa's earlier alphabetical sort.

## Running the import

The library-level importer (`SettingsStore::import_json` and the
matching history / profile / recent-path stores) is wired today;
manual import is straightforward and works with both the GUI and
CLI consumers:

1. Drop `settings.json` at `$XDG_CONFIG_HOME/grexa/settings.json`.
   Grexa's `import_json` API will merge it on next launch.
2. Drop the three history files into `$XDG_DATA_HOME/grexa/`,
   renaming `search_path_history.json` → `recent_paths.json`.
3. Launch Grexa once; it will:
   - Translate `UseWindowsSearchIndex` and `EnableDockerSearch`.
   - Strip `WindowX/Y`, clamp `WindowWidth/Height` if too small.
   - Route `AiSearchApiKey` to the keyring and clear it from the
     file.
   - Mark imported Windows-style paths as unavailable.

## Verifying the import

After the first launch, check:

```bash
cat $XDG_CONFIG_HOME/grexa/settings.json | jq .ai_search_endpoint   # your URL
cat $XDG_DATA_HOME/grexa/search_history.json | jq 'length'          # ≤ 20
secret-tool lookup service io.visorcraft.Grexa.ai account \
    https://api.openai.com/v1                                        # your key
```

If `secret-tool` finds the API key, the import succeeded. If it
returns nothing and you previously had an API key configured in
Grex, the keyring backend was unavailable — see
[security.md](security.md#api-key-handling).

## Bringing translations forward

Grex's `.resw` resource catalogs do not port automatically. The
mapping is documented in
[grex-strings-migration-matrix.md](grex-strings-migration-matrix.md);
the new Fluent catalogs live in
`crates/grexa-i18n/locales/<tag>/grexa.ftl`. Translators interested in
bringing forward translations from Grex should follow
[translations.md](translations.md).
