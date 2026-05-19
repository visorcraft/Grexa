# Grex SearchTabContent Code-Behind Audit

This document records Grex `Controls/SearchTabContent.xaml.cs` behavior that
Grexa must preserve, replace, or intentionally drop when building the
Linux-native Search page.

Source evidence:

- `Controls/SearchTabContent.xaml.cs`
- `Controls/SearchTabContent.xaml`
- `ViewModels/TabViewModel.cs`
- `Services/RecentPathsService.cs`
- `Services/RecentSearchesService.cs`
- `Services/SearchProfilesService.cs`
- `Services/ExportService.cs`
- `Services/ContextMenuService.cs`
- `Services/ContextPreviewService.cs`
- `Services/AiSearchService.cs`
- `UITests/SearchUITests.cs`
- `UITests/AiSearchUiWiringTests.cs`
- `Tests/SearchTabContentRightClickTests.cs`
- `Tests/Controls/ExcludeDirsValidationTests.cs`
- `IntegrationTests/RightClickContextMenuTests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `docs/grex-tab-viewmodel-audit.md`

Note: this audit reflects the current Grex worktree, including the active
uncommitted AI Search code-behind changes in `SearchTabContent.xaml.cs`.

## Role

`SearchTabContent` is a WinUI `UserControl` that binds a `TabViewModel` to the
Search tab UI. It is much more than passive binding glue. It owns or coordinates:

- command bar button behavior
- input field updates
- recent path suggestions and removal
- filter pane visibility
- search and replace start/cancel flow
- WSL warning dialogs
- result grid item source assignment
- result grid width and visibility management
- result row double-click/open behavior
- right-click context menus
- context preview dialog launch
- export actions
- profile save/load/delete actions
- search history save/load/remove/clear actions
- AI chat mode and request lifecycle
- localization refresh
- high-contrast theme refresh
- localized tooltip registration
- public methods used by main-window keyboard shortcuts

Grexa replacement:

- Split this behavior across a QML Search page, a Rust tab/session controller,
  and small platform integration services.
- Keep UI-only layout behavior in QML.
- Keep search, replace, filtering, sorting, export formatting, history, profile,
  AI request, and container target state in Rust services/controllers.
- Replace WinUI, WinForms, Windows clipboard, Windows dialogs, Explorer, WSL,
  and Windows editor-launch behavior with Linux/KDE equivalents.

## Construction And Lifetime

Constructor behavior:

- initializes XAML components
- binds `AiMessagesListView.ItemsSource` to `_aiChatMessages`
- updates AI empty state
- registers localized tooltips once
- subscribes `Loaded`, `Unloaded`, and `DataContextChanged`

Loaded behavior:

- binds the current `TabViewModel`
- initializes content column widths
- refreshes recent path suggestions
- hides result headers until data/filter exists
- subscribes to localization changes
- refreshes localized UI text
- subscribes to `MainWindow.ThemeChanged`
- enqueues initial high-contrast theme application

Unloaded behavior:

- unsubscribes from theme changes
- cancels and disposes active AI request token
- clears AI in-flight state
- unsubscribes from localization changes

Grexa requirements:

- QML components should initialize from a tab/session model without recursive
  DataContext-style mutation.
- Active AI requests and search/replace tasks must be cancelled or detached
  cleanly when a tab/page is destroyed.
- Tooltip/localization subscriptions should be registered once per component
  lifetime and cleaned up on destruction.
- Theme refresh should rely on Qt/Kirigami theme propagation where possible,
  with a thin Grexa theme layer only for app-specific density/contrast choices.

## Binding And ViewModel Synchronization

`SearchTabContent` exposes a nullable `ViewModel` wrapper over `DataContext`.
The setter uses `_isUpdatingDataContext` to avoid recursive
`DataContextChanged` loops.

`DataContextChanged` behavior:

- ignores recursive updates
- unbinds the old view model
- sets the new data context under the recursion guard
- immediately syncs search type/results combo boxes when loaded
- calls `BindViewModel()` when already loaded
- clears bindings when the data context becomes null

`BindViewModel()` copies view-model values into controls:

- path
- search term
- replace text
- search-within-results filter
- search type
- content/files result mode
- all filter checkboxes
- Windows Search checkbox
- match-file and exclude-dir text
- size limit type/value/unit
- result item sources
- header visibility
- command enablement
- status display hook
- Windows Search checkbox state
- column context menu checkmarks

It then subscribes to `ViewModel.PropertyChanged`.

`UnbindViewModel()` unsubscribes property-change handling.

`ViewModel_PropertyChanged` dispatches to the UI thread and handles:

- status text
- search state and Search/Replace button labels
- command eligibility
- content/files column visibility
- search and file result collection changes
- files/content mode changes
- Windows Search option state

Grexa requirements:

- Use explicit QML bindings and model signals instead of manual copy/sync
  where possible.
- Avoid duplicated truth between controls and tab state.
- Preserve the result that switching selected tabs restores all tab-specific UI
  fields, filters, result mode, result rows, and button state.
- UI-thread marshalling must be explicit for updates coming from Rust workers.

## Command State

`UpdateSearchButtonState()` drives:

- Search button enabled state from `ViewModel.CanSearchOrStop`
- AI button enabled state from active AI request or required path/query inputs
- Replace button disabled in AI mode
- Replace button enabled when replace is running or replacement inputs are valid
- Replace checkbox disabled in AI mode
- Replace textbox collapsed in AI mode
- AI send button/progress state

`SetAiMode(bool)` toggles AI mode and then refreshes result visibility, search
button state, and export button state.

Grexa requirements:

- One controller should compute command eligibility for toolbar actions and
  keyboard shortcuts.
- AI mode must disable replace and export, and hide result grids/filter controls.
- Search, replace, and AI request cancellation must not conflict with each
  other.

## Path Input And Recent Paths

Path input behavior:

- user typing refreshes suggestions and updates `ViewModel.SearchPath`
- suggestion choice sets the full path, not the formatted display text
- query submit uses chosen suggestion or typed query
- focus refreshes suggestions
- browse uses a WinForms `FolderBrowserDialog`
- browse writes selected path into UI and view model
- browse adds selected path to recent paths

Suggestion behavior:

- `RecentPathsService.FilterPaths(searchText)` returns matching paths
- suggestions are `PathSuggestion(fullPath, formattedDisplay)`
- display formatting trims separators, keeps short paths, and abbreviates long
  paths as first segment plus last segment
- each suggestion has a remove button
- removing a suggestion calls `RecentPathsService.RemoveRecentPath`, refreshes
  suggestions, restores current text, and focuses the path box

Grexa replacement:

- Use KDE/portal folder selection instead of WinForms.
- Use Linux path abbreviation with `/` separators.
- Preserve recent path filtering, browse capture, add-on-search, and per-entry
  removal.
- Keep full path separate from display text.
- Treat mounted paths as normal Linux paths; no WSL classification.

## Search Input And Filters

Search term changes update `ViewModel.SearchTerm` and command state.

Results filter controls:

- text changes update `ViewModel.ResultsFilterText`
- regex toggle updates `ViewModel.ResultsFilterIsRegex`
- both refresh result header visibility and export state
- a guard prevents programmatic textbox changes from feeding back into the view
  model

Filter controls update the view model directly:

- Text/Regex combo
- Content/Files combo
- respect `.gitignore`
- case-sensitive search
- include system files
- include subfolders
- include hidden items
- include binary/searchable documents
- include symbolic links
- Windows Search index
- size limit type
- size limit value
- size unit

Size value parsing:

- blank clears the limit
- positive numeric values are rounded up with `Math.Ceiling`
- invalid values leave the previous view-model value unchanged

Exclude-dir validation before search/replace:

- `*` and `**` are rejected as "no results possible"
- values without commas that contain `^`, `$`, or `|` are treated as likely
  regex and validated
- invalid regex shows an error notification and cancels the operation

Grexa requirements:

- Keep all filter controls and update the tab/session model immediately.
- Validate exclude-dir regex before dispatching search/replace.
- Preserve size value rounding semantics unless the core size-limit model is
  changed and documented.
- Move regex validation into shared Rust logic so GUI and CLI can share behavior
  where appropriate.
- Replace Windows Search checkbox with Linux file-index/Baloo eligibility.

## Keyboard And Shortcut Surface

Code-behind keyboard behavior:

- Enter in search textbox starts AI discussion when AI mode is active.
- Enter in search textbox starts search when AI mode is inactive and
  `ViewModel.CanSearch` is true.
- Enter in AI chat input sends a follow-up message.
- AI Send button sends a follow-up message.
- Enter in replace textbox starts replace when `ViewModel.CanReplace` is true.
- Space on selected content result opens context preview.
- Double-click on content or file result opens the file.

Public methods used by host-level shortcuts:

- `CanExecuteSearchShortcut`
- `ExecuteSearchShortcut()`
- `TryCancelActiveOperationFromShortcut()`
- `ClearSearchAndReplaceInputsFromShortcut()`

`TryCancelActiveOperationFromShortcut()` routes cancellation to the Replace or
Search button depending on `_isCurrentOperationReplace`.

`ClearSearchAndReplaceInputsFromShortcut()` clears search text first, then
replacement text, updates the view model, and returns whether anything changed.

Grexa requirements:

- Preserve Enter-to-search, Enter-to-replace, Space-to-preview, double-click to
  open result, and host-level stop/clear shortcuts.
- Host shortcuts should call controller actions, not synthesize button clicks.
- Shortcut handling must respect current mode: AI chat, search, replace, or
  normal result browsing.

## Search Flow

`AppBarSearchButton_Click` cancels the active operation if
`ViewModel.IsSearching` is true; otherwise it calls `SearchButton_Click`.

`SearchButton_Click`:

1. cancels any AI request
2. exits AI mode
3. marks current operation as Search
4. validates exclude-dir input
5. checks for likely mounted WSL home paths and offers a WSL warning dialog
6. collapses the filter options pane
7. clears search-within-results filter
8. clears result list item sources
9. copies current UI input values into the view model
10. adds the path to recent paths
11. resets and initializes column widths
12. awaits `ViewModel.PerformSearchAsync()`
13. updates headers and result item sources
14. adjusts content column widths
15. updates row widths
16. records the search in history
17. refreshes export enabled state
18. refreshes command state in `finally`

Grexa replacement:

- Drop WSL warning and conversion.
- Search dispatch should be a controller action rather than a UI event handler
  doing all orchestration.
- Keep filter-pane collapse on search start.
- Keep clearing local search-within-results for a new search.
- Keep recent-path and search-history capture after search.
- Keep result-mode-specific post-search UI updates, but implement them through
  QML model signals rather than resetting item sources manually.

## Replace Flow

`AppBarReplaceButton_Click` cancels the active operation when any search/replace
is running; otherwise it calls `ReplaceButton_Click`.

`ReplaceButton_Click`:

1. cancels any AI request
2. exits AI mode
3. marks current operation as Replace
4. validates exclude-dir input
5. checks for likely mounted WSL home paths and offers a WSL warning dialog
6. shows a confirmation dialog
7. returns unless the user confirms
8. resets column widths
9. clears search-within-results filter
10. clears result item sources
11. copies current UI input values into the view model
12. forces Files mode
13. adds the path to recent paths
14. initializes Files-mode column widths
15. awaits `ViewModel.PerformReplaceAsync()`
16. updates Files result item source and row widths
17. refreshes header visibility
18. refreshes command state in `finally`

Observed Grex edge cases:

- `_isCurrentOperationReplace` is set before validation and confirmation. If
  validation fails or the user cancels confirmation, the flag can remain true
  until another operation updates it.
- Export enabled state is explicitly refreshed after search, but not after
  replace in the same handler.

Grexa requirements:

- Keep replace confirmation before file writes.
- Validate command eligibility through the same controller rule that enables
  the Replace action.
- Keep replace disabled for container targets.
- Reset operation-kind state when validation or confirmation cancels replace.
- Refresh export/action state after replace results are available.

## AI Search Chat

AI state:

- `_isAiModeActive`
- `_isAiRequestInFlight`
- `_aiChatMessages`
- `_aiConversationHistory`
- `_aiRequestCancellationTokenSource`

AI entry behavior:

- AI toolbar button cancels the current AI request when one is in flight.
- Otherwise it starts a new AI discussion.
- A new discussion requires configured endpoint, nonblank path, and nonblank
  search query.
- Starting AI mode collapses filter options and hides result grids/filter UI.
- Starting a new conversation clears prior messages/history and input text.
- The initial user message is the current search query.

Follow-up behavior:

- only allowed in AI mode
- ignores blank messages
- clears input before sending
- appends user turn to chat and conversation history
- sends endpoint, optional API key, optional model, search context, and full
  conversation history to `AiSearchService.SendDiscussionTurnAsync`
- context includes path, search query, regex/files flags, and filter
  suggestions
- successful assistant response is appended to chat and history
- failed response appends localized failure text
- cancellation appends localized cancellation text
- completion clears in-flight state and updates buttons

Filter suggestions sent to AI:

- respect `.gitignore`
- case sensitivity
- include subfolders
- include hidden items
- include binary files
- include symbolic links
- Windows Search index
- match-file patterns when present
- exclude dirs when present
- size limit when active

Grexa requirements:

- Preserve OpenAI-compatible endpoint, optional API key, optional model, model
  discovery/test, and follow-up conversation.
- Add the PLAN-required privacy/opt-in gate before sending local path, query,
  and filters to AI.
- Replace Windows Search wording in AI context with Linux file-index wording.
- Keep AI mode hiding result grids/search-within-results and disabling replace
  and export.
- AI requests must be cancellable and isolated per tab.

## Result Header And Grid Visibility

`UpdateResultsHeaderVisibility()`:

- AI mode hides result filter, Content grid, and Files grid, and shows chat
- no view model hides all result areas
- otherwise result UI is shown only when the current mode has backing results
  or a search-within-results filter is active
- Content and Files grids are mutually exclusive

`UpdateResultsHeader()`:

- AI mode hides both grids
- otherwise switches visible grid based on `ViewModel.IsFilesSearch`

Grexa requirements:

- Preserve no-result hidden grid behavior.
- Preserve filter-visible-when-filter-active behavior, even if filtered result
  count is zero.
- Keep AI chat mutually exclusive with result grids.

## Result Columns

Code-behind owns dependency properties for Content widths:

- name
- line
- column
- text
- path

And Files widths:

- name
- size
- matches
- path
- extension
- encoding
- date modified

Column behavior:

- resizer drag changes pixel widths with a minimum of 60
- resizer double-tap auto-sizes based on localized header and displayed values
- content columns reset to name 220, line 80, column 90, text star, path star
- files columns reset to name 220, size 100, matches 80, path star, extension
  80, encoding 100, date 150
- row grids are manually synchronized to header widths
- content name column shrinks to 90 when all filenames are short
- path column shrinks to 60 when no displayed rows have a directory path
- row width updates are retried after layout to account for virtualized
  containers

Column visibility:

- Content context menus can hide line, column, and path.
- Files context menus can hide size, matches, path, extension, encoding, and
  date modified.
- check marks are represented by prefixing menu item text
- visibility changes update both header and realized row column widths
- visibility state is stored in `TabViewModel` and persisted by settings

Grexa requirements:

- Use Qt/QML table header resize/visibility facilities where possible.
- Preserve resizable, auto-size, sortable, and hide/show columns.
- Persist column visibility and likely widths.
- Avoid manual per-realized-row width synchronization if the QML table model can
  keep header/body layout aligned.
- Validate large result tables with 100k+ rows before final UI commitment.

## Sorting

Header click handlers call `ViewModel.SortResults`:

- name -> `FileName`
- extension -> `Extension`
- encoding -> `Encoding`
- matches -> `MatchCount`

Other columns are likely wired in XAML or through repeated header context
menus. `TabViewModel` owns sort direction and filtering interaction.

Grexa requirements:

- Keep sorting in the Rust model/controller, not in ad hoc QML item order.
- Header click wiring should map every visible sortable column to an explicit
  sort field.
- Preserve sort-before-filter behavior from `TabViewModel`.

## Result Opening

Double-click behavior:

- content rows open the exact `SearchResult`
- file rows are converted to a synthetic `SearchResult` at line 1, column 1
- fallback item-click double-click detection uses a 500 ms interval for content
  rows

`OpenFileInEditor`:

- converts WSL paths to Windows paths
- skips opening when non-WSL file does not exist
- opens `.env` with Notepad++ when available
- opens PHP-related extensions with PhpStorm when available
- otherwise uses the default application
- line/column are passed to Notepad++ only; PhpStorm/default open ignore line
  and column
- errors fall back to shell-opening the file

Grexa replacement:

- Drop WSL conversion and Windows editor discovery.
- Use user-configurable Linux editor command templates for line/column.
- Use `xdg-open` or portals as the default open path.
- Support `org.freedesktop.FileManager1` or portal-based reveal for folders.
- Container results should offer copy path and possibly a future runtime-aware
  open/reveal strategy only when meaningful.

## Context Preview And Context Menus

Space key on a selected content result opens context preview.

Content right-click behavior:

- finds the clicked `SearchResult`
- Docker/container mode shows a limited container menu
- local mode shows a custom menu with Preview, Show in Explorer, Copy Path, and
  Copy File Name

Preview behavior:

- converts WSL path to Windows path
- reads before/after counts from settings
- calls `ContextPreviewService.GetContextAsync`
- shows `ContextPreviewDialog`
- primary action opens in editor
- errors show a preview error dialog

Files right-click behavior:

- Docker/container mode shows the limited container menu
- local mode delegates to `ContextMenuService.ShowContextMenu`

Docker context menu:

- Copy container path
- Copy file name

Grexa requirements:

- Preserve Space-to-preview and right-click preview actions for content rows.
- Context preview should be Linux-native and use Grexa encoding detection.
- Replace Explorer with file-manager reveal.
- Preserve copy path and copy filename.
- Keep container context menus target-aware and read-only.

## Profiles

Profiles flyout:

- refreshes profiles on opening
- shows empty state when none exist

Save profile flow:

- prompts for name
- cancels unless Save is clicked
- rejects blank name
- requires path and search term
- confirms overwrite when a profile already exists
- saves a `SearchProfile` with full filter snapshot:
  - path and term
  - match files and exclude dirs
  - regex/files mode
  - `.gitignore`
  - case sensitivity
  - include system, subfolders, hidden, binary, symbolic links
  - Windows Search index
  - size limit type/value/unit
  - string comparison mode
  - Unicode normalization mode
  - diacritic sensitivity
  - culture

Apply profile flow:

- copies saved fields into controls and view model
- updates size controls
- updates culture/comparison state in the view model
- hides the flyout
- refreshes command and Windows Search state

Delete profile flow:

- confirms deletion
- deletes by profile name
- refreshes profile list

Grexa requirements:

- Preserve full profile snapshots.
- Replace Windows Search field with Linux file-index setting during migration.
- Keep apply-profile as a state update, not an immediate search unless a future
  feature explicitly adds "apply and search".
- Use QML dialogs and Rust storage.

## Search History

History flyout:

- refreshes recent searches on opening
- shows empty state when none exist

Apply history item flow restores:

- path
- search term
- match files
- exclude dirs
- regex/files mode
- case sensitivity
- `.gitignore`
- include subfolders
- include hidden items
- include binary files

It does not restore every profile/default field. Notably absent:

- include system files
- include symbolic links
- Windows Search index
- size limit
- culture/comparison settings

Add-current-search flow records after search:

- search term
- search path
- match files
- exclude dirs
- regex/files mode
- case sensitivity
- `.gitignore`
- include subfolders
- include hidden items
- include binary files
- timestamp
- displayed result count

Remove and clear actions delegate to `RecentSearchesService`.

Grexa requirements:

- Decide whether to preserve Grex's partial history snapshot or upgrade history
  to full filter snapshots.
- If upgraded, provide Grex import compatibility for old partial entries.
- Keep remove and clear actions.
- Keep cap and dedupe behavior from the service audit.

## Export

Export button state:

- disabled without a view model
- disabled in AI mode
- enabled when the currently displayed result collection has rows

CSV export:

- chooses content or files exporter by current mode
- default filename prefix `search_results_yyyyMMdd_HHmmss`
- opens a file save dialog via `ExportService.SaveToFileAsync`
- shows success notification when saved

JSON export:

- same mode and filename behavior as CSV
- uses JSON exporter and success/error notifications

Clipboard export:

- uses content/files clipboard formatter by current mode
- copies through `ExportService.CopyToClipboard`
- shows success or error notification

Grexa requirements:

- Preserve CSV, JSON, and clipboard export for current displayed result mode.
- Export should respect search-within-results filtering because it uses
  displayed result collections.
- Use KDE/portal file save and clipboard APIs.
- Keep success/error notifications.

## Localization

Localized tooltip registration covers major Search tab controls, including
recent path, browse, search/replace, filters, Docker refresh, history, AI, and
results filter controls.

`RefreshLocalization()` manually updates:

- combo box items
- filter checkbox labels
- command bar labels
- profile flyout labels
- replace checkbox
- filter labels
- placeholder text
- AI chat header, empty state, input placeholder, and send button
- size limit and size unit items
- content result headers
- files result headers

It refreshes combo-box selection while temporarily detaching handlers, then
invalidates layout.

Grexa requirements:

- Runtime language switching should refresh all visible strings, tooltips,
  accessible names, menu labels, dialogs, and placeholders.
- Prefer declarative QML translation bindings where possible.
- Keep selected values stable through language refresh.

## Theme And Pointer Behavior

The code-behind manually applies Grex high-contrast themes by:

- clearing resources when returning to normal themes
- setting ListView item background/foreground resources
- applying foreground colors recursively through the visual tree
- setting button, checkbox, combo box, textbox, and command bar resources
- forcing WinUI visual states to refresh

Pointer handlers set hand cursors for buttons, checkboxes, combo boxes, and
combo box items through reflection on WinUI protected cursor internals.

Grexa replacement:

- Use Qt/Kirigami/Breeze theme integration first.
- Limit Grexa-specific theme code to documented density/color preferences.
- Cursor behavior should use standard QML pointer handlers and controls.

## Windows-Specific Behavior To Replace Or Drop

Drop:

- WinUI `UserControl`, `DependencyProperty`, `ContentDialog`, `MenuFlyout`,
  `ListView`, and `DispatcherQueue`
- WinForms `FolderBrowserDialog`
- Windows clipboard API
- Explorer reveal
- Notepad++/PhpStorm Windows executable discovery
- WSL path warning and conversion
- Windows Search checkbox
- manual WinUI high-contrast resource mutation
- `%Temp%\Grex.log`

Replace with:

- Qt/QML/Kirigami components
- KDE/portal folder and save dialogs
- Qt/KDE clipboard
- file-manager reveal through Freedesktop/KDE mechanisms
- Linux editor command templates and `xdg-open`
- optional Baloo/file-index toggle
- structured `tracing` logs under XDG state

## Test Coverage To Preserve

Grex coverage includes:

- Search tab code-behind has right-click handlers and context menu service
- right-click does not throw for content/files rows and empty areas
- context menu routes for content and files result rows
- Search UI updates search state and status through `TabViewModel`
- search-within-results filters and restores status
- sorting updates displayed order
- clearing results resets UI state
- tab title updates from path
- AI UI wiring exists in XAML/code-behind
- AI mode start collapses filter options pane
- export service formats CSV, JSON, and clipboard output
- profile and history services manage persisted entries

Grexa test requirements:

- QML/controller tests for search, replace, reset, AI mode, export state, and
  result visibility.
- Keyboard shortcut tests for Enter, Space, double-click, stop, and clear.
- Context menu tests for local and container targets.
- Profile/history flow tests at the controller level.
- Export tests that prove filtered displayed rows are exported.
- Localization refresh tests for visible Search page text.
- Large-result column/table responsiveness tests.

## Current Grexa Status

Grexa now has a QML Search page backed by `SearchController`. Implemented
surface includes recent paths, filters, search/replace flow with confirmation
and result summary dialogs, AI panel lifecycle, search-within-results, result
model roles, Space-to-preview, open/reveal/copy actions, export, and
history/profile UI.

Remaining gaps:

- add UI automation for command enablement and keyboard/pointer behavior
- decide whether Grex-style column controls remain required
- broaden tests for export of filtered rows and profile/history interactions
- add localization-refresh tests for visible Search page text
- search history flyout UI
- export menu UI
- localization refresh for the Search page
- theme/density integration for the Search page
