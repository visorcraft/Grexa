# Grex SearchTabContent XAML Audit

This document records Grex `Controls/SearchTabContent.xaml` layout, controls,
result columns, and visual states that Grexa must preserve, replace, or
intentionally redesign in the Linux-native Search page.

Source evidence:

- `Controls/SearchTabContent.xaml`
- `Controls/SearchTabContent.xaml.cs`
- `ViewModels/TabViewModel.cs`
- `Models/SearchResult.cs`
- `Models/FileSearchResult.cs`
- `Models/AiChatMessage.cs`
- `UITests/AiSearchUiWiringTests.cs`
- `docs/grex-search-tab-content-codebehind-audit.md`
- `docs/grex-tab-viewmodel-audit.md`

Note: this audit reflects the current Grex worktree, including the active
uncommitted AI Search XAML layout changes in `SearchTabContent.xaml`.

## Top-Level Shape

`SearchTabContent` is a WinUI `UserControl` named `SearchTabControl`.

Top-level resources:

- `BooleanToVisibilityConverter`
- `SearchResultTooltipLinePrefixConverter`

The main content is a padded, stretch-aligned grid with five rows:

1. path and search target row
2. search input and command bar row
3. filter options pane
4. search-within-results panel
5. AI chat panel or results tables

Visual states:

- `Wide`: active at `MinWindowWidth=800`
- `Narrow`: active below 800 and changes `OptionsPanel` and
  `SearchInputPanel` orientation to vertical

Grexa replacement:

- Build the Search page as the first usable tool surface, not a landing page.
- Use a dense Kirigami/QML layout with the same workflow order.
- Preserve narrow-window reflow for path, search input, and filters.
- Prefer native Qt/Kirigami controls and layouts instead of WinUI grids.

## Path And Target Row

Path row controls:

- `PathAutoSuggestBox`
- recent path suggestion template
- per-suggestion remove button
- Docker target combo box
- Docker refresh button
- Browse button

`PathAutoSuggestBox`:

- placeholder: enter or paste path
- handles text changed, suggestion chosen, query submitted, and focus
- item template displays `DisplayText` with ellipsis
- remove button is transparent, 24x24, and registered by `Loaded`

Container target:

- visible only when `IsDockerSearchGloballyEnabled`
- `DockerContainerComboBox` binds `DockerContainerOptions`
- selected item binds two-way to `SelectedDockerOption`
- displays `Label`
- tooltip/header text says search target
- refresh button uses accent style and refresh glyph

Browse:

- accent button
- content starts as `Browse...`
- minimum width 40
- enabled through `IsFileBrowserEnabled`
- text is shortened by code-behind when narrow

Grexa requirements:

- Preserve path text entry, browse, recent path suggestions, and per-entry
  removal.
- Preserve a target selector with Local first, plus Docker and Podman targets.
- Use a KDE/portal folder picker.
- Disable browse when a container target is active.
- Use Linux path display and no WSL/UNC-specific formatting.

## Search Input And Command Row

Search input area:

- horizontal stack in wide mode, vertical in narrow mode
- `SearchTextBox` with minimum width 350 and width 400
- search text key handling supports Enter
- `SearchHistoryButton` opens a history flyout
- `ReplaceCheckBox` toggles replacement input visibility
- `ReplaceWithTextBox` is in the second row and initially collapsed

Search history flyout:

- width 400, max height 400
- title row with `SearchHistoryTitleTextBlock`
- `ClearHistoryButton`
- `SearchHistoryListView`
- list template shows `DisplayText` and `SecondaryText`
- each row has a remove button
- `NoSearchHistoryTextBlock` empty state

Command bar controls:

- Search button
- AI button
- Replace button
- Reset button
- Export button with CSV, JSON, and clipboard menu
- Profiles button with save/list/remove flyout
- separator
- Filter Options toggle

Command bar details:

- Search label is visually collapsed in the current XAML.
- AI button uses an emoji font icon in the current worktree.
- Search, AI, Replace, and Export start disabled.
- Export flyout offers CSV, JSON, and Copy to Clipboard.
- Profiles flyout is width 420, max height 450, and has Save Current, list, row
  remove buttons, and empty state.
- Filter Options toggle starts checked, so the filter pane is visible at first
  render.

Grexa requirements:

- Keep a compact command strip with icon-first actions and tooltips.
- Use Breeze/lucide-equivalent native icon names where available rather than
  emoji glyphs.
- Preserve Search/Stop, AI/cancel, Replace/Stop, Reset, Export, Profiles,
  History, and Filter Options command affordances.
- Preserve search history and profile flyout workflows or equivalent QML popups.
- Keep text from overflowing compact buttons; prefer icon-only actions with
  accessible names.

## Filter Options Pane

`FilterOptionsPane` is row 2 and initially visible.

The pane contains:

- match/exclude and mode grid
- size limit controls
- two-row checkbox grid

Match/exclude and mode grid:

- fixed label/input columns: 140, 400, 140, 400
- row 0: Match Files and Exclude Dirs
- row 1: Search Type and Search Results
- row 2: Size Limit

Controls:

- `MatchFileNamesTextBox`
- `ExcludeDirsTextBox`
- `SearchTypeComboBox`: Text Search, Regex Search
- `SearchResultsComboBox`: Content, Files
- `SizeLimitComboBox`: No Limit, Less Than, Equal To, Greater Than
- `SizeLimitNumberBox`: numeric input, initially hidden with its panel
- `SizeUnitComboBox`: KB, MB, GB

Checkbox grid:

- `RespectGitignoreCheckBox`
- `SearchCaseSensitiveCheckBox`
- `IncludeSystemFilesCheckBox`
- `IncludeSubfoldersCheckBox`, checked by default
- `IncludeHiddenItemsCheckBox`
- `IncludeBinaryFilesCheckBox`
- `IncludeSymbolicLinksCheckBox`
- `UseWindowsSearchCheckBox`

Grexa replacement:

- Preserve all non-Windows filters.
- Replace `UseWindowsSearchCheckBox` with optional Linux file index/Baloo.
- Preserve match-file and exclude-dir examples but adapt path examples to
  Linux.
- Keep filters scannable and compact; avoid nested cards.
- Use responsive wrapping so four-column desktop layout becomes usable on
  narrow windows.

## Search-Within-Results Panel

`ResultsFilterPanel`:

- row 3
- initially collapsed
- card-like background with radius 8
- contains `ResultsFilterTextBox`
- contains `ResultsFilterRegexToggle` with `.*`
- regex toggle uses monospace bold text

The code-behind shows this panel when there are results or an active local
result filter.

Grexa requirements:

- Preserve search-within-results text box and regex toggle.
- Hide the panel when no results/filter exists.
- Hide the panel in AI mode.
- Implement filtering in the model/controller for large result sets.

## AI Chat Panel

`AiChatPanel`:

- row 4
- initially collapsed
- same grid row as results tables
- stretch-aligned
- background uses app page background
- radius 8 and padding 12

The current worktree changed it from row 3 with row span to row 4 without row
span so it shares the lower workspace with results.

AI panel layout:

- header row with `AiChatHeaderTextBlock`
- progress ring at the right
- stretch message list area with min height 0
- `AiMessagesListView`
- centered empty state text
- input row with `AiChatInputTextBox`
- accent `AiSendButton`

Message template:

- card-like border with radius 8
- speaker text
- timestamp text
- wrapping content text

Grexa requirements:

- Preserve AI chat as a lower-workspace mode that replaces result grids.
- Keep progress indicator and empty state.
- Keep message list scrollable and vertically stretchable.
- Keep follow-up input and Send button.
- Add explicit privacy/opt-in UI before sending path/query/filter context.

## Results Workspace

The results container is row 4 and contains mutually exclusive grids:

- `ContentResultsGrid`
- `FilesResultsGrid`

Both start collapsed. Each uses:

- rounded outer border
- header row
- separator line
- scrollable `ListView`
- transparent row background
- stretch item content
- horizontal and vertical scrollbars

Grexa requirements:

- Build two virtualized result views or one table model with mode-specific
  columns.
- Keep result tables hidden until results/filter exist.
- Keep Content, Files, and AI modes mutually exclusive.
- Use QML table virtualization, not a non-virtual list if row counts are large.

## Content Results Table

Content columns:

- Name, width 220
- Line, width 80
- Column, width 90
- Text, width `2*`
- Path, width `2*`

Header behavior:

- `ContentNameHeaderButton` is sortable through `NameHeaderButton_Click`
- line, column, text, and path headers have context flyouts but no direct click
  sort handler in the XAML shown
- every header context menu can hide Line, Column, and Path
- `Thumb` resizers exist for Name, Line, Column, and Text
- no `Thumb` is present for Path in the inspected XAML

Row behavior:

- `ResultsListView` handles container changes, double-tap, item click,
  right-tap, and preview key down
- single selection mode
- item tooltip shows relative path and highlighted match preview
- row displays file name, line, column, trimmed line content, and directory path
- file name uses accent color and semibold weight

Grexa requirements:

- Preserve all Content columns and preview tooltip content.
- Preserve right-click, double-click, and Space preview hooks.
- Make every visible sortable column intentionally sortable or intentionally
  not sortable with documented behavior.
- Keep column hiding for Line, Column, and Path.
- Add consistent resize behavior for Path if the final QML table supports it.

## Files Results Table

Files columns:

- Name, width 220
- Size, width 100
- Matches, width 80
- Path, width `2*`
- Ext, width 80
- Encoding, width 100
- Date Modified, width 150

Header behavior:

- Name, Matches, Ext, and Encoding headers are sortable through click handlers
- Size, Path, and Date Modified headers have context menus but no direct sort
  handler in the inspected XAML
- every header context menu can hide Size, Matches, Path, Ext, Encoding, and
  Date Modified
- resizer thumbs exist for every Files column

Row behavior:

- `FilesResultsListView` handles container changes, double-tap, item click, and
  right-tap
- tooltip shows relative path and every preview match from `PreviewMatches`
- row displays file name, formatted size, match count, directory path,
  extension, encoding, and formatted modified date

Grexa requirements:

- Preserve Files columns and preview-match tooltip list.
- Preserve hide/show context menus for optional columns.
- Decide whether Size, Path, and Date Modified should be sortable in Grexa; if
  so, implement sort fields in the model.
- Keep Files-mode rows dense and scannable.

## Localization

Nearly every user-facing XAML control has `x:Uid`:

- page root
- path/search controls
- history controls
- command bar buttons and menu items
- profiles controls
- filter labels and inputs
- checkbox filters
- search-within-results controls
- AI controls
- table headers
- column context menu items

Grexa requirements:

- Use Qt translation IDs for all visible strings, placeholders, tooltips, and
  accessible names.
- Preserve runtime language switching from the code-behind audit.
- Avoid hard-coded English fallback text in the final QML where translation
  bindings are available.

## Visual Style

Grex Search tab visual traits:

- dense, utilitarian layout
- restrained row spacing
- rounded panels and result containers at radius 8
- theme-resource colors instead of fixed colors
- accent file names
- compact 11-14px table typography
- transparent command/list backgrounds
- separator lines between headers and rows

Grexa direction:

- Keep dense, work-focused Search UI.
- Use KDE Plasma/Breeze colors and focus rings.
- Avoid decorative cards, hero areas, gradients, and oversized marketing-style
  panels.
- Cards should be limited to repeated items, popups, dialogs, and focused tools.
- Keep table text compact but readable.

## Windows-Specific Markup To Replace

Drop or replace:

- WinUI `UserControl`, `CommandBar`, `AppBarButton`, `AppBarToggleButton`,
  `AutoSuggestBox`, `ListView`, `MenuFlyout`, `Flyout`, `Thumb`, `FontIcon`,
  `ProgressRing`, and XAML visual states
- Segoe/WinUI icon glyphs and emoji AI icon
- `UseWindowsSearchCheckBox`
- Windows-specific Browse behavior implied by code-behind

Use instead:

- Kirigami/QML page/component structure
- Qt Quick Controls and Kirigami actions
- KDE/Breeze icon names with bundled fallbacks where needed
- QML table/header components
- KDE/portal dialogs
- optional Baloo/file-index control

## Test Coverage To Preserve

Grex tests assert:

- Search button exists and uses compact collapsed label
- AI button exists with click handler and icon
- AI chat panel, input, and send button exist
- AI message list is scrollable
- AI panel lives in row 4, stretches, and does not row-span the filter panel in
  the current worktree
- code-behind collapses filter options when AI starts
- `SearchTabContent` has right-click handlers for Content and Files results
- UI/view-model tests cover result filtering, sorting, clearing, and tab title
  update

Grexa test requirements:

- QML smoke tests for Search page load and named action availability.
- Screenshot/geometry checks for desktop and narrow widths.
- AI mode visibility tests: filter/results hidden, chat visible.
- Search mode visibility tests: Content vs Files grid.
- Table column tests for headers, widths, hide/show menus, and tooltips.
- Keyboard and pointer interaction tests for result rows.

## Current Grexa Status

Grexa now ships `SearchPage.qml` with path/recent-path controls, local/container
target selection, search/history/profile/replace flows, filter controls, an
AI chat panel, result rows, export actions, and per-tab snapshots.

Remaining gaps:

- add QML screenshot/geometry automation for desktop and narrow widths
- add column/table parity if Grexa keeps Grex-style column header menus and
  resizers
- broaden keyboard and pointer tests for row actions, export, preview, and AI
  mode visibility
- responsive/narrow visual states
- localized QML string IDs
