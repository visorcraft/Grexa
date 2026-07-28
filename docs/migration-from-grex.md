# Migrating from Grex

Grexa preserves Grex concepts and many serialized values, but the current
release does not ship a one-click Windows Grex backup importer in the GUI or
CLI. Do not copy Grex JSON files directly into Grexa's XDG directories and
expect them to be translated.

This guide separates:

1. manual migration from the Windows Grex application;
2. automatic migration from older Grexa JSON storage.

## Before moving data

Back up the Grex directory on Windows:

```text
%LocalAppData%\Grex\
├── settings.json
├── search_path_history.json
├── search_history.json
└── search_profiles.json
```

Keep the original files unchanged. They are useful as a reference even when
paths cannot map directly to Linux.

## Current migration status

| Data | Grex to Grexa status |
| ---- | --------------------- |
| Settings concepts | Supported by Grexa, but must be recreated in the GUI or translated to Grexa's snake_case schema by a developer tool. |
| Recent paths | No automatic Windows path translator; revisit valid Linux paths in Grexa. |
| Search history | No end-user importer; rerun important searches or recreate them as profiles. |
| Search profiles | No end-user importer; recreate named searches with Linux paths. |
| AI endpoint/model | Enter again in Settings. |
| AI API key | Enter again so it is stored in the Linux Secret Service. Never copy plaintext into `settings.json`. |
| Window position | Intentionally not migrated; the Linux window manager owns placement. |
| Window size | Grexa supports width/height, but manual GUI configuration is simplest. |

`SettingsStore::import_json` is a library API for Grexa's current snake_case
`DefaultSettings` JSON. It is not a PascalCase Grex backup converter.

## Concept mapping

Use this table while recreating settings:

| Grex | Grexa |
| ---- | ----- |
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
| `UILanguage` | `ui_language` |
| `StringComparisonMode` | `string_comparison_mode` |
| `UnicodeNormalizationMode` | `unicode_normalization_mode` |
| `DiacriticSensitive` | `diacritic_sensitive` |
| `Culture` | `culture` |
| `DefaultMatchFiles` | `default_match_files` |
| `DefaultExcludeDirs` | `default_exclude_dirs` |
| `Content*ColumnVisible` | `content_*_column_visible` |
| `Files*ColumnVisible` | `files_*_column_visible` |
| `ContextPreviewLines*` | `context_preview_lines_*` |
| `WindowWidth` / `WindowHeight` | `window_width` / `window_height` |
| `AiSearchEndpoint` | `ai_search_endpoint` |
| `AiSearchModel` | `ai_search_model` |
| `ThemePreference` | `theme_preference` |
| `UseWindowsSearchIndex` | `use_file_index`, currently reserved because Baloo seeding is deferred |
| `EnableDockerSearch` | `enable_container_search`, covering Docker and Podman |

The exact current types/defaults are in
[Reference: settings schema](reference.md#settings-schema).

## Path translation

Windows paths need a Linux filesystem location:

| Windows source | Linux approach |
| -------------- | -------------- |
| `C:\Users\<name>\...` | Copy to a Linux directory, then select that directory in Grexa. |
| Another drive letter | Mount the volume, commonly under `/mnt`, `/media`, or a user-selected mount point. |
| UNC share | Mount through the desktop/filesystem, then use the resulting local path. |
| `\\wsl$\...` or `\\wsl.localhost\...` | Use the native Linux path when running Grexa inside that Linux environment. |
| WSL `/mnt/<drive>/...` | Choose the corresponding native/mounted path on the Grexa host. |

Grexa does not store "unavailable Windows path" placeholders. A search/profile
must point at a directory visible to the Linux filesystem.

## Recommended manual migration

1. Install and start Grexa.
2. Open **Settings** and reproduce search defaults, filters, appearance,
   context, editor, container, accessibility, and privacy choices.
3. Select each important Linux search root once. Grexa records it in recent
   paths.
4. Recreate important Grex searches.
5. Use **Save Profile** for searches that should persist.
6. Configure the AI endpoint/model only if needed.
7. Enable AI explicitly and enter the API key into the keyring.
8. Run representative searches and compare expected files, match counts, and
   ignore behavior.

Important Linux differences:

- path comparison and recent-path dedupe are case-sensitive;
- `.gitignore` behavior follows the Rust `ignore` crate on Linux;
- Windows Search is not used;
- Docker and Podman replace Grex's Docker-only target model;
- window position is not persisted;
- desktop notifications, editor launch, Trash, and file-manager reveal use
  freedesktop/Linux services.

See [Linux decisions](linux-decisions.md) for the complete rationale.

## Advanced settings translation

Developers may create a Grexa-format `settings.json` manually. Start Grexa
once, close it, edit the generated file, then relaunch. Use only keys and enum
spellings from the [settings reference](reference.md#settings-schema).

Do not:

- place `AiSearchApiKey` in the file;
- copy PascalCase Grex keys unchanged;
- copy `WindowX` or `WindowY`;
- assume Windows path strings are valid Linux roots;
- replace current settings without a backup.

Grexa ignores unknown fields and supplies defaults for missing fields.

## Verifying a manual migration

```bash
jq . "$XDG_CONFIG_HOME/grexa/settings.json"

find "$XDG_DATA_HOME/grexa/db/recent_paths" \
  -maxdepth 1 -name 'entry-*.md' -print

find "$XDG_DATA_HOME/grexa/db/search_profiles" \
  -maxdepth 1 -name 'entry-*.md' -print
```

When the corresponding XDG variables are unset, use:

```text
~/.config/grexa/settings.json
~/.local/share/grexa/db/
```

Check an AI key, if `secret-tool` is installed:

```bash
secret-tool lookup \
  service com.visorcraft.Grexa.ai \
  account https://api.openai.com
```

The account is the canonical base URL, with `/v1` removed.

## Upgrading from older Grexa storage

Older Grexa releases stored application lists in:

```text
$XDG_DATA_HOME/grexa/recent_paths.json
$XDG_DATA_HOME/grexa/search_history.json
$XDG_DATA_HOME/grexa/search_profiles.json
```

Current Grexa migrates each file automatically when:

- the legacy file exists;
- its corresponding grexa-db collection is empty;
- the JSON parses as the expected legacy Grexa type.

Destination collections:

```text
$XDG_DATA_HOME/grexa/db/recent_paths/
$XDG_DATA_HOME/grexa/db/search_history/
$XDG_DATA_HOME/grexa/db/search_profiles/
```

After successful migration, the source becomes:

```text
recent_paths.json.bak
search_history.json.bak
search_profiles.json.bak
```

Grexa does not delete these backups. Compare the new records, then archive or
remove the `.bak` files yourself.

If the destination collection already has records, Grexa leaves the legacy
file untouched to avoid combining stores silently.

## Translation catalogs

Grex `.resw` catalogs do not load in Grexa. Grexa uses Fluent under
`crates/grexa-i18n/locales/`.

The upstream string mapping is recorded in
[grex-strings-migration-matrix.md](grex-strings-migration-matrix.md).
Contributors adding a locale should follow [Translating Grexa](translations.md).
