# Grex Storage Services Audit

This document records the exact persistence behavior of Grex
`SettingsService`, `RecentPathsService`, `RecentSearchesService`, and
`SearchProfilesService`, and defines Grexa's XDG-based replacement. It is the
behavioral source of truth for PLAN.md phase 0 line 149 and for the existing
`crates/grexa-core/src/storage.rs` skeleton.

Source evidence:

- `Services/SettingsService.cs`
- `Services/RecentPathsService.cs`
- `Services/RecentSearchesService.cs`
- `Services/SearchProfilesService.cs`
- `Models/RecentSearch.cs`
- `Models/SearchProfile.cs`
- `Tests/Services/SettingsServiceTests.cs`
- `Tests/Services/RecentPathsServiceTests.cs`
- `Tests/Services/RecentSearchesServiceTests.cs`
- `Tests/Services/SearchProfilesServiceTests.cs`
- `crates/grexa-core/src/storage.rs` (existing Grexa skeleton)
- `crates/grexa-core/src/models.rs`

The Settings UI surface and Linux replacements are covered in
`grex-settings-view-audit.md`; this audit is intentionally scoped to the
on-disk persistence contract and dedupe/limit/sort/import semantics.

## File Layout

Grex stores all four JSON files under `%LocalAppData%\Grex\`:

| Concept           | Grex filename                  | Grexa filename (XDG)                                 |
| ----------------- | ------------------------------ | ---------------------------------------------------- |
| Default settings  | `settings.json`                | `$XDG_CONFIG_HOME/grexa/settings.json`               |
| Recent paths      | `search_path_history.json`     | `$XDG_DATA_HOME/grexa/recent_paths.json`             |
| Search history    | `search_history.json`          | `$XDG_DATA_HOME/grexa/search_history.json`           |
| Search profiles   | `search_profiles.json`         | `$XDG_DATA_HOME/grexa/search_profiles.json`          |

Notes:

- Grex uses one Windows directory for all four files. Grexa splits them by
  XDG semantics: settings is user configuration, the other three are user
  data. This matches XDG conventions and is consistent with
  `crates/grexa-core/src/storage.rs::AppPaths`.
- Grex's recent-paths file is `search_path_history.json`. Grexa renames the
  file to `recent_paths.json` because the field is the concept and the file
  is new on Linux. Import code that consumes a Grex backup must read either
  name; see [Import From Grex Backups](#import-from-grex-backups).
- All four Grex services create their parent directory on demand. Grexa's
  `save_json` helper already does the same.
- All four Grex services swallow I/O and JSON errors silently and fall back
  to an empty default. Grexa preserves the "missing or unreadable file →
  empty default" contract but surfaces parse errors through
  `JsonStoreError` so the caller can decide whether to log or notify.

## Thread Safety And Caching

- All four Grex services use a single `static readonly object _lock` for
  every public method.
- Only `SettingsService` caches a deserialized instance (`_cachedSettings`),
  exposes `InvalidateCache`, and reloads when the cache is null.
- `SettingsService` also supports `SetSettingsFilePathOverride(string?)` so
  tests can redirect the file path without touching `%LocalAppData%`.

Grexa contract:

- The Rust stores are stateless: each call re-reads from disk. This matches
  every Grex store except `SettingsService`. A future caching layer for
  settings is acceptable but must invalidate on every write the same way
  Grex does, and must support a test-only path override (the `AppPaths`
  helper already provides this via `AppPaths::under`).
- Concurrent access correctness on Linux relies on `fs::write` being atomic
  enough for our use case. A follow-up task should adopt
  `tempfile::persist` for write-then-rename atomicity, especially for
  settings, to match the implicit single-writer expectation Grex enforces
  with the global lock.

## DefaultSettings Schema

Grex `DefaultSettings` (Services/SettingsService.cs:25-79) and Grexa
`DefaultSettings` (crates/grexa-core/src/storage.rs:86-122) both use
PascalCase / snake_case fields. The mapping below names each Grex field, its
default, and the Grexa equivalent or replacement.

| Grex field                          | Default                    | Grexa field                              | Notes |
| ----------------------------------- | -------------------------- | ---------------------------------------- | ----- |
| `IsRegexSearch`                     | `false`                    | `regex_search`                           | identical |
| `IsFilesSearch`                     | `false`                    | `files_search`                           | identical |
| `RespectGitignore`                  | `false`                    | `respect_gitignore`                      | identical |
| `SearchCaseSensitive`               | `false`                    | `search_case_sensitive`                  | identical |
| `IncludeSystemFiles`                | `false`                    | `include_system_files`                   | identical (Linux semantics differ; see linux-decisions) |
| `IncludeSubfolders`                 | `true`                     | `include_subfolders`                     | identical |
| `IncludeHiddenItems`                | `false`                    | `include_hidden_items`                   | identical |
| `IncludeBinaryFiles`                | `false`                    | `include_binary_files`                   | identical |
| `IncludeSymbolicLinks`              | `false`                    | `include_symbolic_links`                 | identical |
| `UseWindowsSearchIndex`             | `false`                    | `use_file_index`                         | Grex name retained as legacy alias on import; semantics map to Baloo on Linux |
| `EnableDockerSearch`                | `false`                    | `enable_container_search`                | covers both Docker and Podman in Grexa |
| `SizeUnit`                          | `KB`                       | `size_unit`                              | identical enum |
| `ThemePreference`                   | `GentleGecko`              | not present yet                          | see [Theme Translation](#theme-translation) |
| `UILanguage`                        | `"en-US"`                  | `ui_language`                            | identical |
| `StringComparisonMode`              | `Ordinal`                  | `string_comparison_mode`                 | identical |
| `UnicodeNormalizationMode`          | `None`                     | `unicode_normalization_mode`             | identical |
| `DiacriticSensitive`                | `true`                     | `diacritic_sensitive`                    | identical |
| `Culture`                           | `CultureInfo.CurrentCulture.Name` | `culture`                         | Grexa hardcodes `"en-US"` default; runtime should still seed from the system locale |
| `DefaultMatchFiles`                 | `""`                       | `default_match_files`                    | identical |
| `DefaultExcludeDirs`                | `""`                       | `default_exclude_dirs`                   | identical |
| `ContentLineColumnVisible`          | `true`                     | `content_line_column_visible`            | identical |
| `ContentColumnColumnVisible`        | `true`                     | `content_column_column_visible`          | identical |
| `ContentPathColumnVisible`          | `true`                     | `content_path_column_visible`            | identical |
| `FilesSizeColumnVisible`            | `true`                     | `files_size_column_visible`              | identical |
| `FilesMatchesColumnVisible`         | `true`                     | `files_matches_column_visible`           | identical |
| `FilesPathColumnVisible`            | `true`                     | `files_path_column_visible`              | identical |
| `FilesExtColumnVisible`             | `true`                     | `files_ext_column_visible`               | identical |
| `FilesEncodingColumnVisible`        | `true`                     | `files_encoding_column_visible`          | identical |
| `FilesDateModifiedColumnVisible`    | `true`                     | `files_date_modified_column_visible`     | identical |
| `WindowX`, `WindowY`                | `null`                     | not stored                               | see [Window Geometry](#window-geometry) |
| `WindowWidth`                       | `1100`                     | `window_width`                           | identical |
| `WindowHeight`                      | `700`                      | `window_height`                          | identical |
| `ContextPreviewLinesBefore`         | `5`                        | `context_preview_lines_before`           | clamp `[1, 20]` on get/set in Grex; Grexa must enforce the same clamp |
| `ContextPreviewLinesAfter`          | `5`                        | `context_preview_lines_after`            | identical |
| `AiSearchEndpoint`                  | `"https://api.openai.com/v1"` | `ai_search_endpoint`                  | trim on write |
| `AiSearchApiKey`                    | `""`                       | not stored in settings.json              | see [API Key Handling](#api-key-handling) |
| `AiSearchModel`                     | `"gpt-4o-mini"`             | `ai_search_model`                       | trim on write |

### Theme Translation

Grex `ThemePreference` is a numeric enum:

| Value | Name           |
| ----- | -------------- |
| 0     | `System`       |
| 1     | `Light`        |
| 2     | `Dark`         |
| 3     | `GentleGecko`  |
| 4     | `BlackKnight`  |
| 5     | `Diamond`      |
| 6     | `Dreams`       |
| 7     | `Paranoid`     |
| 8     | `RedVelvet`    |
| 9     | `Subspace`     |
| 10    | `Tiefling`     |
| 11    | `Vibes`        |

Grex serializes the value as an integer (verified by
`SettingsServiceTests.ExportSettingsAsJson_With*Theme_ExportsCorrectNumericValue`).

Grexa import rules:

- `0` (`System`) maps to Grexa's "Follow KDE color scheme" mode.
- `1` (`Light`) maps to Grexa Light.
- `2` (`Dark`) maps to Grexa Dark.
- `3..=11` map to Grexa's high-contrast / accent theme set; the existing
  Grex names are kept as theme identifiers so user-visible labels can be
  preserved or rebranded per Linux design guidance.
- Imported themes that have no Grexa counterpart fall back to "Follow KDE
  color scheme" and the importer logs a one-time notice.

Grexa's `DefaultSettings` does not yet include a `theme_preference` field.
Adding it is a near-term task; until then, theme import is a no-op.

### Window Geometry

Grex persists `WindowX`, `WindowY`, `WindowWidth`, `WindowHeight`.
`SettingsService.ImportSettingsFromJson` writes these fields (see
`SettingsServiceTests.ImportSettingsFromJson_DoesNotImportWindowPosition`,
which documents an intent the implementation does not actually enforce).

Grexa rules:

- Persist `window_width` / `window_height` only. Window position is left to
  the window manager. KDE Plasma already restores window placement via the
  Window Rules module and the X11/Wayland session manager, and forcing
  coordinates from settings.json is hostile to multi-monitor and
  Wayland-only setups.
- The Grex import path must drop `WindowX` and `WindowY` and use only the
  reported width/height when those values are sane (>= 400 each).

### API Key Handling

`AiSearchApiKey` is stored in plaintext in Grex `settings.json`. Grexa must
not replicate this. PLAN.md phase 8 already requires KWallet / Secret
Service storage with an explicit opt-in if secret storage is unavailable.

Import rules for Grex backups:

- Parse `AiSearchApiKey` if present, route it to the secret store, and
  remove it from the in-memory `DefaultSettings` before persisting.
- If secret storage is unavailable, surface the imported key to the user
  with an explicit prompt and do not silently fall back to plaintext.

### Import Semantics

Grex `ImportSettingsFromJson`:

- Deserializes with `PropertyNameCaseInsensitive = true`,
  `ReadCommentHandling = Skip`, `AllowTrailingCommas = true`.
- Returns `(success, errorMessage)` and never throws.
- Treats `null` (the JSON literal) as an error: "Invalid settings file
  format."
- Treats malformed JSON as an error prefixed with "Invalid JSON format:".
- Treats unknown properties as ignored, not as errors.
- Overwrites every known field with the imported value, except:
  - `UILanguage`: only overwritten when the imported value is non-empty.
  - `Culture`: only overwritten when the imported value is non-empty.
  - `WindowX/Y/Width/Height`: code comment says "intentionally not
    imported," but the current implementation does import them. Grexa
    follows the *comment*, not the bug: width/height in, position out.
- Trims `AiSearchEndpoint` and `AiSearchModel`.
- Coerces null strings to empty.
- Saves the merged result and invalidates the cache.

Grexa must match every observable behavior except:

- Drop `WindowX/Y`.
- Route `AiSearchApiKey` through the secret store.
- Translate `UseWindowsSearchIndex` to `use_file_index`.
- Translate `EnableDockerSearch` to `enable_container_search`.
- Map `ThemePreference` per the table above.

### Export Semantics

Grex `ExportSettingsAsJson` returns indented JSON of the cached settings.
Round-trip tests assert that an export followed by `DeleteSettingsFile`,
then an import of the exported JSON, reproduces every field except window
position (per the comment).

Grexa must:

- Emit pretty-printed JSON with stable key order (`serde_json` default is
  insertion order, which matches the Rust struct field order; this is
  sufficient).
- Always include the key set defined by the current `DefaultSettings`
  struct; do not omit fields with default values.

### DeleteSettingsFile / Restore Defaults

`SettingsService.DeleteSettingsFile()` deletes the on-disk file and
invalidates the cache; the next `LoadSettings` returns a fresh
`new DefaultSettings()`.

Grexa replacement:

- Provide `SettingsStore::delete(&self) -> Result<(), JsonStoreError>` that
  removes the file (ignoring a missing file as success) and forces the next
  `load()` to return `DefaultSettings::default()`.
- The current Rust skeleton lacks this method; add it.

### DockerSearchEnabledChanged Event

Grex raises a static `DockerSearchEnabledChanged` event whenever
`SetEnableDockerSearch` changes the persisted value. Setting the same value
twice does not raise the event.

Grexa replacement:

- The CLI does not need an event. The GUI can subscribe to a settings
  change channel exposed by the controller, but the storage layer should
  remain pure value-in / value-out.
- The semantic guarantee to preserve is "no-op writes do not fire change
  notifications" — owners of the GUI signal must diff old and new values
  before emitting.

## RecentPathsService Contract

Source: `Services/RecentPathsService.cs`, `Tests/Services/RecentPathsServiceTests.cs`.

- File path: `%LocalAppData%\Grex\search_path_history.json`.
- JSON shape: `List<string>` (path strings).
- Cap: 20 entries (`MaxRecentPaths = 20`).
- Locking: global static lock.
- Errors: silently swallowed; missing or invalid file → empty list.

Operations:

| Operation                     | Behavior |
| ----------------------------- | -------- |
| `GetRecentPaths()`            | Returns the list verbatim. Order: newest first. |
| `AddRecentPath(path)`         | Ignores null/empty/whitespace. Removes any existing equal entry, inserts at index 0, truncates to 20. |
| `RemoveRecentPath(path)`      | Ignores null/empty/whitespace. Removes any existing equal entry. |
| `FilterPaths(searchText)`     | Empty/whitespace `searchText` returns all paths. Otherwise filters by `ToLowerInvariant().Contains(searchText.ToLowerInvariant())`. |

Equality:

- Dedupe and removal are exact string match, case-sensitive (NTFS often is
  case-insensitive in practice, but the code does not normalize). Grexa
  inherits the same case-sensitive equality on Linux, which is the correct
  default for case-sensitive filesystems.

Grexa skeleton parity (`crates/grexa-core/src/storage.rs::RecentPathStore`):

- ✅ Limit of 20 (`RECENT_PATH_LIMIT`).
- ✅ Newest-first insertion with dedupe.
- ✅ `remove()` matches Grex behavior.
- ❌ No `add()` whitespace guard. Grex skips empty/whitespace input; Grexa
  currently inserts them. Add a guard.
- ❌ No `filter(query)` helper. Add `RecentPathStore::filter(&self, query: &str) -> Result<Vec<PathBuf>, JsonStoreError>` mirroring Grex
  semantics (case-insensitive substring against the UTF-8 string form of
  the path; treat empty/whitespace as "return all").
- ❌ No `clear()` helper, but Grex does not expose one for recent paths
  either; not required for parity.

Tests to port (all xunit `[Fact]` in
`Tests/Services/RecentPathsServiceTests.cs`):

- empty file → empty list
- add → contains, count 1
- duplicate add → moved to top
- empty/null/whitespace add → no-op
- filter empty → all
- filter substring → matching subset
- filter case-insensitive → matches regardless of casing
- remove existing → removed
- remove empty/null/whitespace → no-throw
- 25 adds → 20 retained, newest first
- corrupted JSON → empty list (handled by error guard)
- long paths, special characters, network paths, relative paths, unicode →
  treated as opaque strings

## RecentSearchesService Contract

Source: `Services/RecentSearchesService.cs`, `Models/RecentSearch.cs`,
`Tests/Services/RecentSearchesServiceTests.cs`.

- File path: `%LocalAppData%\Grex\search_history.json`.
- JSON shape: `List<RecentSearch>` (objects).
- Cap: 20 entries.
- Locking: global static lock.
- Errors: silently swallowed.

Operations:

| Operation                     | Behavior |
| ----------------------------- | -------- |
| `GetRecentSearches()`         | Verbatim list, newest first. |
| `AddRecentSearch(s)`          | Ignores null and entries with null/empty/whitespace `SearchTerm`. Removes any existing entry with the same `GetKey()`, inserts at 0, truncates to 20. |
| `RemoveRecentSearch(s)`       | Removes entries with the same `GetKey()`. Ignores null. |
| `ClearHistory()`              | Deletes the file. |
| `FilterSearches(searchText)`  | Empty/whitespace returns all. Otherwise case-insensitive substring against either `SearchTerm` or `SearchPath`. |

### Dedupe Key

`RecentSearch.GetKey()` returns:

```
{SearchTerm}|{SearchPath}|{IsRegexSearch}|{IsFilesSearch}|{SearchCaseSensitive}|{MatchFileNames}|{ExcludeDirs}
```

Seven fields, separated by `|`. The key is case-sensitive across all
components. Two entries with the same term/path but different
`IsRegexSearch` are treated as distinct (verified by
`RecentSearch_GetKey_CreatesUniqueKey`).

Fields NOT part of the key (so they get overwritten when the same key is
re-added): `RespectGitignore`, `IncludeSubfolders`, `IncludeHiddenItems`,
`IncludeBinaryFiles`, `Timestamp`, `ResultCount`.

### RecentSearch Schema

| Grex field             | Grexa field                | Notes |
| ---------------------- | -------------------------- | ----- |
| `SearchTerm`           | `search_term`              | dedupe key component |
| `SearchPath`           | `search_path` (`PathBuf`)  | dedupe key component |
| `MatchFileNames`       | `match_file_names`         | dedupe key component |
| `ExcludeDirs`          | `exclude_dirs`             | dedupe key component |
| `IsRegexSearch`        | `regex_search`             | dedupe key component |
| `IsFilesSearch`        | `files_search`             | dedupe key component |
| `SearchCaseSensitive`  | `search_case_sensitive`    | dedupe key component |
| `RespectGitignore`     | `respect_gitignore`        | overwritten on re-add |
| `IncludeSubfolders`    | `include_subfolders`       | overwritten on re-add |
| `IncludeHiddenItems`   | `include_hidden_items`     | overwritten on re-add |
| `IncludeBinaryFiles`   | `include_binary_files`     | overwritten on re-add |
| `Timestamp` (`DateTime.Now`) | `timestamp_unix` (`u64`) | overwritten on re-add; Grex uses local time, Grexa uses Unix seconds |
| `ResultCount`          | `result_count`             | overwritten on re-add |

### Display Helpers

`RecentSearch` exposes computed display helpers used by the UI:

- `DisplayText` → `"<term up to 37 chars + '...' if longer>"<{Regex if regex}> - {N} result(s)`
- `SecondaryText` → `"<path, head ellided if > 50 chars> | {Timestamp:g}"`

These belong in Grexa's view layer, not the storage struct. The audit
records them only so the UI port can reproduce the exact truncation lengths
(37/40 for term, 47/50 for path, `g` short date/time format).

### Grexa Skeleton Gaps

`crates/grexa-core/src/storage.rs::SearchHistoryStore`:

- ✅ Limit of 20.
- ✅ Newest-first insertion.
- ❌ Dedupe key. Current Grexa code:
  ```rust
  searches.retain(|existing| {
      existing.search_path != search.search_path
          || existing.search_term != search.search_term
      });
  ```
  This only keys on `(path, term)`. Grex keys on seven fields. **Fix**:
  expand the dedupe condition to the full seven-field key. The simplest
  faithful port is a `key(&self) -> String` method that joins the seven
  fields with `|`, matching Grex's `GetKey()` byte for byte so future
  migration of an existing user's `search_history.json` keeps the same
  identity.
- ❌ No empty-term guard. Add it.
- ❌ No `remove(&self, key: &str)` and no `clear(&self)`. Add both.
- ❌ No `filter(&self, query: &str)` helper. Add it (case-insensitive
  substring against term OR path).

### Tests To Port

From `RecentSearchesServiceTests.cs`:

- empty history → empty list
- add valid → count 1
- duplicate key add → moved to top, latest data wins (e.g., updated
  `ResultCount`)
- empty `SearchTerm` → not added
- null search → no-throw
- remove existing → removed
- clear → empty
- 25 adds → 20 retained, newest first
- filter empty → all
- filter by term substring
- filter by path substring
- `GetKey` distinguishes by `IsRegexSearch`
- `GetKey` ignores `ResultCount`

`DisplayText` / `SecondaryText` tests belong in the GUI crate, not core.

## SearchProfilesService Contract

Source: `Services/SearchProfilesService.cs`, `Models/SearchProfile.cs`,
`Tests/Services/SearchProfilesServiceTests.cs`.

- File path: `%LocalAppData%\Grex\search_profiles.json`.
- JSON shape: `List<SearchProfile>`.
- Cap: none.
- Locking: global static lock.
- Errors: silently swallowed.

Operations:

| Operation                       | Behavior |
| ------------------------------- | -------- |
| `GetProfiles()`                 | Verbatim list. |
| `Exists(name)`                  | Case-insensitive (`OrdinalIgnoreCase`) name match. Empty/whitespace → false. |
| `AddOrUpdateProfile(profile)`   | Ignores null and entries with empty `Name`. Finds existing by case-insensitive name match. If found: preserve `CreatedAt`, refresh `UpdatedAt` to `DateTime.Now`, replace at index 0 (move to top). If not found: set `CreatedAt`/`UpdatedAt` if defaults, insert at 0. |
| `DeleteProfile(name)`           | Case-insensitive removal. Empty/whitespace → no-op. |
| `ClearProfiles()`               | Deletes the file. |

Sort order:

- **Most-recently-modified-first**, achieved by inserting at index 0 on
  every add/update. There is no alphabetical sort anywhere in the service
  or its tests.

### Schema

`SearchProfile` is an extension of `SearchOptions` plus a few flags:

| Grex field                | Grexa equivalent                                      | Notes |
| ------------------------- | ----------------------------------------------------- | ----- |
| `Name`                    | (top-level `name`)                                    | required, case-insensitive uniqueness |
| `SearchPath`              | `search_options.path`                                 | embedded in `SearchOptions` |
| `SearchTerm`              | `search_options.search_term`                          | embedded |
| `IsRegexSearch`           | `search_options.regex`                                | embedded |
| `IsFilesSearch`           | top-level `files_search`                              | Grex stores this at profile root, not on `SearchOptions`. Grexa already does the same |
| `RespectGitignore`        | `search_options.respect_gitignore`                    | embedded |
| `SearchCaseSensitive`     | `search_options.case_sensitive`                       | embedded |
| `IncludeSystemFiles`      | `search_options.include_system`                       | embedded |
| `IncludeSubfolders`       | `search_options.include_subfolders`                   | embedded |
| `IncludeHiddenItems`      | `search_options.include_hidden`                       | embedded |
| `IncludeBinaryFiles`      | `search_options.include_binary`                       | embedded |
| `IncludeSymbolicLinks`    | `search_options.include_symlinks`                     | embedded |
| `UseWindowsSearchIndex`   | not on `SearchOptions`                                | per-profile opt-in; on Linux this becomes "use Baloo seed". Decision: add `use_file_index` to `SearchOptions` *or* a parallel `SearchProfile.use_file_index` field. Recommendation: add it to `SearchOptions` so the runtime carries the flag through the entire search call |
| `MatchFileNames`          | `search_options.match_file_names`                     | embedded |
| `ExcludeDirs`             | `search_options.exclude_dirs`                         | embedded |
| `SizeLimitType`           | `search_options.size_limit_type`                      | embedded |
| `SizeLimitKB`             | `search_options.size_limit_kb`                        | embedded (Grex uses `long?`, Grexa uses `Option<u64>`) |
| `SizeUnit`                | `search_options.size_unit`                            | embedded |
| `StringComparisonMode`    | `search_options.string_comparison_mode`               | embedded |
| `UnicodeNormalizationMode`| `search_options.unicode_normalization_mode`           | embedded |
| `DiacriticSensitive`      | `search_options.diacritic_sensitive`                  | embedded |
| `Culture`                 | `search_options.culture`                              | Grex: `string` default `""`; Grexa: `Option<String>` (None == ordinal/default) |
| `CreatedAt`               | `created_unix` (`u64`)                                | Grex `DateTime` → Grexa Unix seconds |
| `UpdatedAt`               | `updated_unix` (`u64`)                                | Grex `DateTime` → Grexa Unix seconds |

`SearchProfile.SecondaryText` is a UI helper; not stored.

### Grexa Skeleton Gaps

`crates/grexa-core/src/storage.rs::SearchProfileStore`:

- ✅ Upsert by name with `CreatedAt` preserved.
- ✅ Remove by name.
- ❌ **Sort order**: current code calls
  `profiles.sort_by_key(|profile| profile.name.to_lowercase());` after
  upsert. Grex does **not** sort alphabetically; it moves the touched
  profile to the front of the list. The Settings UI consumes that order.
  Fix: drop the sort and either (a) move the upserted profile to index 0
  to match Grex, or (b) leave existing relative order and re-insert
  upserted at index 0. Option (a) matches Grex exactly.
- ❌ **Name comparison**: Grex matches names with
  `StringComparison.OrdinalIgnoreCase`. Grexa currently uses `==`
  (case-sensitive). Switch to ASCII case-insensitive at minimum; Unicode
  case folding is acceptable but a deliberate upgrade.
- ❌ No `exists(name)` helper. Add it.
- ❌ No `clear()` helper. Add it.
- ❌ No guard against empty `name` in `upsert`. Add one.
- ❌ `SearchProfile::new` sets both `created_unix` and `updated_unix` to
  the same `unix_now()`. That matches Grex's "if default" logic for new
  profiles. The upsert path correctly preserves `created_unix` on update.
  Keep both behaviors.
- ❌ `SearchProfile` has no `use_file_index` field. Decide via the
  `SearchOptions` extension above and update both structs.

### Tests To Port

From `SearchProfilesServiceTests.cs`:

- missing file → empty list
- add new → at top, both timestamps set
- update existing → moves to top, `CreatedAt` preserved, `UpdatedAt`
  advances
- `Exists` is case-insensitive
- delete → removed
- null / empty-name profile → not added
- `SecondaryText` truncates path and term — UI test, not storage

## Import From Grex Backups

PLAN.md phase 10 line 375 requires "Grex-to-Grexa import semantics for
Windows paths, drive letters, UNC paths, WSL paths, Windows-only settings,
culture names, saved regex patterns, Docker settings, profiles, history,
and recent paths."

This audit codifies what the import has to translate. The CLI/GUI importer
should accept a Grex `%LocalAppData%\Grex` directory (or an exported zip)
and produce four Grexa JSON files:

| Source                                | Sink                                    | Translation |
| ------------------------------------- | --------------------------------------- | ----------- |
| `Grex\settings.json`                  | `$XDG_CONFIG_HOME/grexa/settings.json`  | per [Import Semantics](#import-semantics) and translations above |
| `Grex\search_path_history.json`       | `$XDG_DATA_HOME/grexa/recent_paths.json`| each path passed through path translator (Windows → Linux); drive-letter, UNC, WSL paths get either translated or recorded as "unavailable on Linux" entries the user can remove |
| `Grex\search_history.json`            | `$XDG_DATA_HOME/grexa/search_history.json` | same path translation; preserve dedupe keys; `Timestamp` (local) converted to Unix seconds (UTC) |
| `Grex\search_profiles.json`           | `$XDG_DATA_HOME/grexa/search_profiles.json` | same path translation; `UseWindowsSearchIndex` becomes Linux `use_file_index`; `CreatedAt`/`UpdatedAt` to Unix seconds; preserve insertion order |

Path translation rules (also referenced by `docs/linux-decisions.md`):

- `C:\Users\<u>\…` → `$HOME/...` when the user accepts a one-time mapping.
- Other drive letters: kept as opaque strings with a "not available"
  marker; user can remove.
- UNC paths (`\\server\share\…`): translated to `/run/user/<uid>/gvfs/…` or
  similar mounted-share equivalents only when the user explicitly maps
  them; otherwise marked unavailable.
- WSL paths (`\\wsl$\<distro>\…`): always recorded as unavailable.
  Grexa never tries to access a WSL filesystem from Linux.

Out of scope for this audit: regex pattern compatibility (covered by PLAN
phase 2 regex spike) and Docker settings translation (covered by
`grex-docker-search-service-audit.md`).

## Concrete Follow-Up Tasks

These are the actionable gaps between this audit and
`crates/grexa-core/src/storage.rs`. Each is a candidate for a small PR.

1. `DefaultSettings`: add `theme_preference: ThemePreference` and the
   `ThemePreference` enum. Default to `System`. Round-trip with Grex's
   integer encoding.
2. `DefaultSettings`: drop the implicit assumption that
   `WindowX`/`WindowY` will ever be present. Importer drops these.
3. `DefaultSettings`: add `ai_search_api_key` *handling* (not a field):
   importer routes the key to the secret store; the on-disk struct does
   not carry it.
4. `SettingsStore`: add `delete(&self)`; tolerate missing file.
5. `SettingsStore`: add `export_json(&self) -> Result<String, …>` and
   `import_json(&self, json: &str) -> Result<(), ImportError>` that
   implement the merge rules in [Import Semantics](#import-semantics).
6. `RecentPathStore::add`: skip empty/whitespace input.
7. `RecentPathStore`: add `filter(&self, query: &str)` matching Grex.
8. `SearchHistoryStore`: expand dedupe key to the full Grex 7-field key;
   add a `key()` helper on `RecentSearch` that joins them with `|` to
   match Grex byte-for-byte.
9. `SearchHistoryStore::add`: skip entries with empty `search_term`.
10. `SearchHistoryStore`: add `remove(&self, key: &str)`,
    `clear(&self)`, `filter(&self, query: &str)`.
11. `SearchProfileStore::upsert`: stop sorting alphabetically; move
    touched profile to index 0.
12. `SearchProfileStore`: name comparisons case-insensitive
    (`eq_ignore_ascii_case` at minimum).
13. `SearchProfileStore`: add `exists`, `clear`, and a guard against
    empty names.
14. `SearchOptions`: add `use_file_index: bool` so profiles can carry the
    Linux-equivalent of `UseWindowsSearchIndex`.
15. Port the test list from this document into `crates/grexa-core/tests`
    or unit tests under each store. Mirror the Grex test names so the
    parity intent is obvious.
16. Add an `AppPaths::override_for_tests` shim or document that
    `AppPaths::under(tempdir)` is the supported test-only construction
    path (already true).
17. Add a Grex backup importer entry point (out of scope for the storage
    crate; tracked in PLAN phase 10).

Once these are landed, PLAN.md line 149 can be checked off, and the
checkboxes around it that depend on this contract (lines 369, 374, 375,
376) become unblocked.
