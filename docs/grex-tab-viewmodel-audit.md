# Grex TabViewModel Audit

This document records Grex `ViewModels/TabViewModel.cs` behavior that Grexa
must preserve, replace, or intentionally drop in its Linux-native tab
controller.

Source evidence:

- `ViewModels/TabViewModel.cs`
- `Models/SearchResult.cs`
- `Models/FileSearchResult.cs`
- `Models/SearchResultSortField.cs`
- `Tests/ViewModels/TabViewModelTests.cs`
- `UITests/SearchUITests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `Strings/en-US/Resources.resw`
- `docs/grex-search-service-audit.md`
- `docs/grex-docker-search-service-audit.md`
- `docs/grex-windows-search-integration-audit.md`
- `docs/grex-wsl-audit.md`

## Role

Grex `TabViewModel` owns one search tab's mutable UI state. It coordinates:

- search/replace inputs
- filter settings
- selected result mode
- search/replace lifecycle
- cancellation
- result collections
- search-within-results filtering
- result sorting
- status text
- Docker/container mode
- Windows Search option availability
- column visibility defaults
- localized status refresh
- tab title updates

Grexa replacement:

- Keep one isolated state object per tab.
- Put platform-neutral tab/session state in Rust.
- Expose a thin Qt/QML controller or model facade for UI bindings.
- Keep search engine behavior in `grexa-core`, container behavior in
  `grexa-containers`, and QML-only display concerns in the GUI layer.

## Construction And Default State

Constructor behavior:

- requires an `ISearchService`
- accepts optional tab title
- uses singleton localization and notification services
- uses injected `DockerSearchService` or `DockerSearchService.Instance`
- creates `SearchResults` and `FileSearchResults` observable collections
- sets status to `ReadyStatus`
- creates a timestamp title when no title is supplied
- loads default settings from `SettingsService.GetDefaultSettings()`
- subscribes to `SettingsService.DockerSearchEnabledChanged`
- asynchronously initializes Docker support when globally enabled
- rebuilds Docker option list immediately

Default settings loaded into the tab:

- regex mode
- files mode
- match file patterns
- excluded directories
- `.gitignore` respect
- case sensitivity
- include system files
- include subfolders
- include hidden items
- include binary/searchable document files
- include symbolic links
- Windows Search index preference
- size unit
- culture-aware string comparison mode
- Unicode normalization mode
- diacritic sensitivity
- culture
- content/files column visibility

Grexa requirements:

- Tab construction should clone current defaults into independent tab state.
- Later changes in global defaults should not implicitly mutate open tabs unless
  a deliberate sync feature is designed.
- Docker/Podman runtime discovery should be async and non-fatal.
- Default column visibility should persist through settings, but table models
  should remain independent of persisted UI preferences.

## Core Tab State

Grex tab input fields:

- `SearchPath`
- `SearchTerm`
- `ReplaceWith`
- `IsRegexSearch`
- `RespectGitignore`
- `SearchCaseSensitive`
- `IncludeSystemFiles`
- `IncludeSubfolders`
- `IncludeHiddenItems`
- `IncludeBinaryFiles`
- `IncludeSymbolicLinks`
- `MatchFileNames`
- `ExcludeDirs`
- `IsFilesSearch`
- `SizeLimitType`
- `SizeLimitKB`
- `SizeUnit`
- `StringComparisonMode`
- `UnicodeNormalizationMode`
- `DiacriticSensitive`
- `Culture`

Derived command state:

- `CanSearch`: path and term are nonblank and no operation is running
- `CanSearchOrStop`: can start a search, or a search operation is running
- `CanReplace`: path, term, and replacement are nonblank, no operation is
  running, and Docker mode is inactive
- `CanReplaceOrStop`: can start replace, or a replace operation is running
- `IsFileBrowserEnabled`: false when Docker mode is active

Property changes for path, term, replacement, search state, and Docker
selection raise dependent command-state notifications.

Grexa requirements:

- Model explicit command eligibility instead of deriving it only in QML.
- Distinguish search and replace operations so Stop buttons target the active
  operation and the inactive command remains disabled.
- Preserve replace-disabled-for-container-targets behavior.
- Use one state update/event path so QML buttons, shortcuts, and menu actions
  cannot bypass eligibility rules.

## Search Lifecycle

`PerformSearchAsync` returns immediately when `CanSearch` is false. Otherwise it:

1. Calls `CancelSearch()` for any previous operation.
2. Creates a fresh `CancellationTokenSource`.
3. Marks the current operation as Search, not Replace.
4. Sets `IsSearching = true`.
5. Clears search-within-results text without updating status.
6. Clears backing and displayed result collections.
7. Clears stored result summary.
8. Sets status to `SearchingStatus`.
9. Restarts the search stopwatch.

For local search, it calls `ISearchService.SearchAsync` with all active tab
filters and comparison settings.

For Docker search, it:

1. Ensures Docker is available and a container is selected.
2. Tries direct in-container grep.
3. Uses direct grep results when successful.
4. Falls back to local mirror search when grep is missing or fails.
5. Reports Docker errors through status and notifications.
6. Cleans old mirrors when returning to local mode.

After search results arrive:

- Content mode stores raw line results in `_allSearchResults`.
- Files mode aggregates line results by full path into `_allFileSearchResults`.
- Files mode computes file metadata, extension, encoding, modified time, first
  match line, and up to five preview matches.
- Files mode default sort is `MatchCount` descending.
- Content mode default sort is `FileName` ascending.
- `ApplyResultsFilter()` populates visible collections.
- status summary is set to `FoundMatchesStatus`.
- `IsSearching` is set false in `finally`.

On `OperationCanceledException`, status returns to `ReadyStatus`. On other
exceptions, status becomes `ErrorStatus` and an error notification is shown.

Grexa requirements:

- Search should stream progress/results, but completed tab state must still
  preserve Grex's summaries, default sort choices, and filtered/unfiltered
  backing collections.
- Content and Files modes should be views over the same search output where
  possible, not separate searches.
- Docker/Podman direct search and mirror fallback should return the same tab
  result shape as local search.
- Container path display should be preserved after mirror fallback.
- Cancellation should end with a non-error cancelled/ready state unless Grexa
  intentionally adds a distinct cancelled status.

## Replace Lifecycle

`PerformReplaceAsync` returns immediately when `CanSearch` is false or
`ReplaceWith` is blank. It does not check `CanReplace` directly.

When allowed, it:

1. Cancels any previous operation.
2. Creates a fresh cancellation token.
3. Marks the current operation as Replace.
4. Forces `IsFilesSearch = true`.
5. Sets `IsSearching = true`.
6. Clears result filters and result collections.
7. Sets status to `ReplacingStatus`.
8. Restarts the stopwatch.
9. Calls `ISearchService.ReplaceAsync` with active filters and comparison
   settings.
10. Stores returned `FileSearchResult` values.
11. Sets status summary to `ReplacedMatchesStatus`.
12. Applies search-within-results filtering.

On cancellation, status returns to `ReadyStatus`. On errors, status becomes
`ErrorStatus` and an error notification is shown.

Grex UI disables replace when Docker mode is active, but the public
`PerformReplaceAsync` method uses `CanSearch` rather than `CanReplace`. This
means a non-UI caller could bypass the Docker replace guard.

Grexa requirements:

- Replace must validate the replacement command against the canonical
  `CanReplace`/target eligibility rule, not just search eligibility.
- Replace must force Files mode or an equivalent changed-files result view.
- Replace should keep the safe confirmation and cancellation flow from the UI
  audit before writing files.
- Container replace must remain disabled unless a later feature explicitly
  designs writable container replace.

## Cancellation

`CancelSearch()`:

- cancels the current token source when present and not already cancelled
- ignores `ObjectDisposedException`
- logs other cancellation errors
- disposes the source and clears the field in `finally`

`SetIsSearching(false)` happens in the async operation's `finally` block, not in
`CancelSearch()` itself. Tests verify that the search service receives
cancellation and the tab becomes not-searching when the task completes.

`ClearResults()`:

- clears result filters and all result collections
- clears the stored summary
- sets status to `ReadyStatus`
- sets `IsSearching = false`
- schedules active Docker mirror cleanup

Grexa requirements:

- Keep cancellation cooperative and observable by the search worker.
- Do not dispose or drop shared cancellation state in a way that prevents the
  worker from seeing cancellation.
- Define whether clearing results cancels active worker tasks or only resets UI
  state; Grex currently resets state while the old task may still complete.
- Add cancellation-latency tests under heavy result streaming.

## Result Storage

Grex keeps separate backing and displayed collections:

- `_allSearchResults`: unfiltered Content results
- `_allFileSearchResults`: unfiltered Files results
- `SearchResults`: displayed Content results
- `FileSearchResults`: displayed Files results

`TotalContentResultsCount` and `TotalFileResultsCount` report backing counts,
not displayed counts.

Grexa requirements:

- Preserve backing versus filtered result distinction.
- Avoid duplicating large row payloads excessively; consider indexes or filtered
  row id lists over copied result objects.
- Keep total-count properties available for status text and result metadata.

## Files Mode Aggregation

Files mode groups search line results by `FullPath`. For each file:

- sums `MatchCount`
- sorts preview matches by line then column
- keeps up to five preview matches
- uses the first preview for first line and preview text
- attempts metadata lookup for size and modified date
- derives extension from the path
- detects encoding from the first 4096 bytes using simple BOM/UTF-8 checks
- catches metadata/encoding errors and still emits a file result with defaults

Defaults on metadata failure:

- size `0`
- encoding `Unknown`
- modified date `DateTime.MinValue`

Special Windows behavior:

- WSL paths are converted to Windows/UNC paths before metadata lookup.
- WSL UNC encoding detection returns `UTF-8`.

Grexa requirements:

- Move aggregation into `grexa-core` or a shared model layer.
- Preserve first-match and preview-match behavior.
- Use Linux-native path metadata directly.
- Do not port WSL path conversion.
- Replace Grex's simple encoding detection with the fuller encoding support
  specified in the encoding phase.

## Search-Within-Results Filtering

Filter state:

- `ResultsFilterText`
- `ResultsFilterIsRegex`
- cached `_resultsFilterRegex`

Changing filter text or mode immediately reapplies the filter when not
searching. During an active operation, changes are stored but not applied until
the operation updates results.

Plain text filtering is case-insensitive ordinal substring matching.

Content result fields searched:

- file name
- relative path
- line content

Files result fields searched:

- file name
- relative path
- extension
- encoding

Regex filtering:

- compiled with `RegexOptions.IgnoreCase | RegexOptions.Compiled`
- invalid regex is treated as no active regex match; because the filter branch
  still asks `MatchesFilter` with regex mode and a null regex, no rows match for
  a nonblank invalid regex
- no user-visible invalid-regex status is produced by `TabViewModel`

Grexa requirements:

- Preserve separate content/files filter field sets.
- Decide whether invalid search-within-results regex should show all rows,
  show no rows, or surface a validation error. Grex's comment says show all,
  but implementation yields no matches.
- Keep filtering local to the result set and avoid rerunning the filesystem
  search.
- Implement filtering in Rust/model code if QML proxy performance is
  insufficient for large result sets.

## Status Text

Status is stored as a resource key plus arguments so it can be reformatted on
language changes.

Important keys:

- `ReadyStatus`: `Ready`
- `SearchingStatus`: `Searching...`
- `ReplacingStatus`: `Replacing...`
- `ErrorStatus`: `Error: {0}`
- `FoundMatchesStatus`: `Found {0} matches in {1} files in {2}`
- `FilteredMatchesStatus`: `Showing {0} matches in {1} files (filtered from {2}
  matches in {3} files) in {4}`
- `ReplacedMatchesStatus`: `Replaced {0} matches in {1} files in {2}`

`SetResultsSummary` stores:

- summary key
- total matches
- total files
- elapsed display text

`UpdateResultsStatusForFilter` restores the original summary when the filter is
blank. When a filter is active, it reports filtered matches/files plus original
matches/files. Content mode counts filtered files by distinct `FullPath` using
case-insensitive comparison; Files mode counts displayed file rows.

Elapsed-time formatting:

- under 30 seconds: two decimals, e.g. `12.43 seconds`
- 30 to 59 seconds: rounded whole seconds
- 60 seconds to under 60 minutes: minutes and seconds
- 60 minutes or more: hours and minutes
- singular/plural labels come from localization keys

Grexa requirements:

- Keep status as localizable key plus typed arguments where possible.
- Preserve filtered-from-original summaries.
- Use robust pluralization for localized Linux app strings rather than
  hard-coded English concatenation.
- Consider a distinct cancelled status only if it is specified and localized.

## Sorting

Grex sort enum:

- `None`
- `FileName`
- `LineNumber`
- `ColumnNumber`
- `RelativePath`
- `Extension`
- `Encoding`
- `MatchCount`

`SortResults(field)`:

- ignores `None`
- initializes backing lists from displayed lists if needed
- returns when there is zero or one backing result
- toggles descending when sorting by the current field again
- resets to ascending when switching fields
- sorts the backing list, then reapplies the active filter

Content supported keys:

- `FileName`
- `LineNumber`
- `ColumnNumber`
- `RelativePath`
- fallback `FileName`

Files supported keys:

- `FileName`
- `RelativePath`
- `Extension`
- `Encoding`
- `MatchCount`
- fallback `FileName`

Default sort after search:

- Content mode: `FileName` ascending
- Files mode: `MatchCount` descending

Grexa requirements:

- Keep sort state per tab.
- Sort backing result order before applying the local filter.
- Preserve default sort choices or document any changed default.
- Add stable tie-breakers if large-result order must be deterministic across
  Rust parallel search runs.

## Tab Title Behavior

`SearchPath` updates the tab title.

Rules:

- blank path resets to original title
- paths up to 30 characters display as-is after trimming trailing separators
- longer paths are split on `\` and `/`
- UNC paths keep the server as the first segment
- paths with two or fewer parts display as-is
- longer paths display `first\...\last`
- parsing errors keep the current title

Grexa requirements:

- Use Linux separators in generated titles, e.g. `/home/.../project`.
- Preserve short-path full display and long-path abbreviation.
- Do not carry UNC-specific formatting into normal Linux path title logic.
- Handle container paths separately so `/app/.../src` remains meaningful.

## Docker State

Docker-related tab state:

- global Docker enabled flag
- Docker CLI availability
- discovered running containers
- selected container
- selected local/container option
- active mirror

Behavior:

- option list always includes local disk first
- selecting a container activates Docker mode only when Docker is globally
  enabled and CLI is available
- Docker mode disables the file browser
- Docker mode disables replace
- container refresh is non-fatal
- container disappearance clears the selected container
- active mirror is cleaned when changing container, clearing results, disabling
  Docker, disposing the tab, or returning to local mode
- `ResolveDockerPath` maps local mirror result paths back to container paths

Grexa requirements:

- Generalize Docker state to Docker/Podman runtime state.
- Keep a Local target option first.
- Preserve target-specific path display.
- Mirror cleanup should be deterministic and cancellable where possible, with
  best-effort cleanup on tab disposal.

## Windows Search And WSL Behavior To Drop

Windows Search option availability is true only when:

- search is not regex
- Docker mode is inactive
- path is a drive-letter Windows path

The option is automatically cleared when it becomes ineligible.

WSL-specific helpers convert Linux-looking paths back to Windows/UNC paths for
metadata lookup and default to a discovered or guessed WSL distribution.

Grexa replacement:

- Replace Windows Search option with optional Linux file-index/Baloo candidate
  seeding.
- Eligibility should depend on local Linux path, non-regex mode, index
  availability, and target support.
- Drop WSL conversion and distribution discovery entirely.

## Localization Refresh

`RefreshLocalization()`:

- reformats current status from stored key/args
- rebuilds Docker option labels, including the localized local-disk option
- logs and swallows errors

Grexa requirements:

- Runtime language switching must refresh status text, target labels, command
  labels, tooltips, and table headers.
- Stored status keys and typed args should make language refresh deterministic.

## Test Coverage To Preserve

Grex tests cover:

- constructor defaults
- property change notifications
- command eligibility
- successful search and replace service calls
- content and files result population
- search-within-results filtering and status restoration
- clearing results
- sorting and sort-direction toggling
- exception and cancellation handling
- tab title abbreviation/reset
- replace forcing Files mode
- match-file and exclude-dir argument passing
- Windows Search eligibility reset
- Docker mode flags and replace disablement
- Docker path resolution null/mirror cases

Grexa test requirements:

- Add Rust unit tests for tab/session state transitions where logic lives in
  Rust.
- Add QML/controller integration tests for command enablement and binding
  updates.
- Add large-result tests for sort/filter responsiveness and memory behavior.
- Add cancellation tests under active streaming.
- Add target-mode tests for local, Docker, and Podman.

## Current Grexa Status

Grexa now has a QML tab strip with stable tab ids and Rust-side
`SearchController` snapshots for per-tab rows, filters, result mode, and
status. Core supports search summaries, file aggregation, sorting, replace,
cancellation, and progress events. The GUI supports search/replace command
flow, search-within-results, tab title abbreviation, and local/container target
selection.

Remaining gaps:

- add GUI automation for command eligibility bindings and tab switching
- test cancellation latency and active-search cleanup when closing tabs
- add large-result responsiveness tests for filtering/sorting and snapshot
  memory use
- decide whether persistent tab/session restore is in scope
- Baloo/file-index option eligibility
- runtime localization refresh for open tabs

These gaps should be implemented before the Search UI MVP is considered
feature-equivalent to Grex.
