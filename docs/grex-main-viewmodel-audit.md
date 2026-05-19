# Grex MainViewModel Audit

This document records Grex `ViewModels/MainViewModel.cs` tab lifecycle behavior
that Grexa must preserve or improve in its Linux-native GUI shell.

Source evidence:

- `ViewModels/MainViewModel.cs`
- `ViewModels/TabViewModel.cs`
- `Tests/ViewModels/MainViewModelTests.cs`
- `UITests/SearchUITests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `docs/grex-tab-viewmodel-audit.md`

## Role

Grex `MainViewModel` is the top-level tab owner. It does not perform search,
replace, settings, localization, navigation, or persistence work directly.

Responsibilities:

- own the tab collection
- create the initial tab
- create additional tabs
- track the selected tab
- prevent closing the last tab
- choose a neighboring selected tab after close
- dispose tab view models when removed or when the main view model is disposed

Grexa replacement:

- Keep a top-level window/session controller that owns tab sessions.
- Keep per-tab search state in the tab/session model described in
  `docs/grex-tab-viewmodel-audit.md`.
- Keep navigation pages such as Search, Regex Builder, Settings, and About out
  of this tab collection unless they are deliberately modeled as search tabs.

## Construction

Constructor behavior:

- accepts an optional `ISearchService`
- creates a concrete `SearchService` when no service is injected
- creates an `ObservableCollection<TabViewModel>`
- creates one initial tab with title `Search 1`
- adds the initial tab to `Tabs`
- sets `SelectedTab` to the initial tab
- logs construction progress and rethrows constructor errors

Tests verify:

- construction creates exactly one tab
- selected tab is the first tab
- injected search service is accepted
- tabs collection is an `ObservableCollection<TabViewModel>`

Grexa requirements:

- Start the Search workspace with one active search tab.
- Use dependency injection or explicit controller construction so tests can use
  fake search services/controllers.
- The initial tab should use a localized/default title equivalent to `Search 1`
  unless restored session state provides a better title.

## Tab Collection

Grex exposes:

```csharp
public ObservableCollection<TabViewModel> Tabs { get; }
```

The collection object is never reassigned after construction. `AddTab` and
`RemoveTab` explicitly raise `PropertyChanged(nameof(Tabs))` even though
`ObservableCollection` already emits collection-changed events.

Tests verify:

- direct modification of the collection does not raise `MainViewModel`
  `PropertyChanged("Tabs")`
- `AddTab` raises `PropertyChanged("Tabs")`
- UI tests treat the collection count as the tab strip source of truth

Grexa requirements:

- Expose tab changes through a Qt model or signal path that QML can observe.
- Prefer a controlled API over exposing a mutable collection directly.
- Emit explicit signals for row insertion/removal and selected-tab changes.
- Do not require QML to mutate the backing tab collection itself.

## Selected Tab

`SelectedTab` is nullable but is expected to be non-null during normal app use.
The setter:

- compares object references
- updates the field only when the reference changes
- raises `PropertyChanged(nameof(SelectedTab))` only on change

Tests verify:

- changing selection raises `SelectedTab`
- setting the same selected tab again does not raise `SelectedTab`
- integration tests assume there is always an active selected tab

Grexa requirements:

- Treat selected tab as required when at least one tab exists.
- Keep no-op selection updates silent.
- Model selected tab by stable tab id or row index for QML, rather than passing
  raw object references across layers.

## Adding Tabs

`AddTab()`:

1. creates a new `TabViewModel` with the shared search service
2. titles it `Search {Tabs.Count + 1}`
3. appends it to `Tabs`
4. selects it
5. raises `PropertyChanged("Tabs")`
6. raises `PropertyChanged("CanRemoveTab")`

Tests verify:

- tab count increases by one
- the new tab is selected
- titles are sequential when tabs are only appended
- after multiple adds, titles are `Search 2`, `Search 3`, `Search 4`, etc.
- `CanRemoveTab` becomes true when more than one tab exists

Grexa requirements:

- New tabs should become the selected active tab.
- New tab creation should clone current default search settings into a fresh
  isolated tab state.
- Tab titles should remain stable after creation unless the tab's path/title
  logic updates them.
- Use a monotonic tab counter if Grexa supports restoring closed tabs,
  user-renamed tabs, or nontrivial tab reorder behavior. `Tabs.Count + 1` can
  duplicate names after middle-tab removal.

## Removing Tabs

`CanRemoveTab` returns `Tabs.Count > 1`.

`RemoveTab(tab)`:

1. returns immediately when only one tab exists
2. finds the tab index
3. disposes the tab
4. removes it from `Tabs`
5. raises `PropertyChanged("Tabs")`
6. raises `PropertyChanged("CanRemoveTab")`
7. if the removed tab was selected:
   - selects the previous tab when the removed index was greater than zero
   - otherwise selects the first remaining tab
   - otherwise sets selected tab to null

Tests verify:

- the last remaining tab cannot be removed
- removing the selected second tab selects the previous tab
- removing the first tab while another tab is selected leaves selection valid
- removing a tab leaves selected tab in the remaining collection
- `CanRemoveTab` becomes false again after returning to one tab
- null and non-existent tab removal do not throw only in the single-tab test
  setup, because `RemoveTab` returns before dereferencing the argument

Observed Grex edge cases:

- If `RemoveTab(null)` is called while multiple tabs exist, `tab.Dispose()`
  would throw.
- If a non-owned tab is passed while multiple tabs exist, Grex disposes it even
  though it is not in `Tabs`, then raises tab-change notifications despite no
  owned tab being removed.

Grexa requirements:

- Remove by stable tab id or verified index.
- Return without side effects if the requested tab does not belong to the
  collection.
- Never dispose a non-owned tab/session.
- Preserve the "always keep at least one tab" rule.
- Preserve previous-tab selection when closing the selected tab after the first
  position.
- Dispose/cancel any running tab work before or during removal.

## Disposal

`Dispose()` iterates over a snapshot of `Tabs` and calls `Dispose()` on each
tab. It does not clear `Tabs`, does not set `SelectedTab` to null, and does not
raise property changes.

Tab disposal unsubscribes Docker settings events and schedules mirror cleanup.

Grexa requirements:

- Window/session disposal should dispose all tab sessions.
- Active searches/replaces should be cancelled before the application exits or
  when a tab is destroyed.
- Container mirror cleanup should be best-effort but tracked enough for tests.
- Avoid leaving QML models pointing at disposed Rust tab objects.

## Logging

Grex writes constructor log messages to:

```text
%Temp%\Grex.log
```

Errors are swallowed for logging itself. Constructor errors are logged and
rethrown.

Grexa replacement:

- Use structured `tracing` logs.
- Default log file should be under `$XDG_STATE_HOME/grexa/grexa.log`.
- Main controller construction and tab lifecycle events should be low-volume
  debug logs, not user-facing status messages.

## Localization And Titles

`MainViewModel` hard-codes tab creation titles as `Search 1`, `Search 2`, etc.
`TabViewModel` separately owns path-based title abbreviation and localized
default timestamp titles when no explicit title is supplied.

Grexa requirements:

- Localize default tab titles.
- Keep explicit user/path-derived titles stable across language switches unless
  they are still a generated default title.
- If session restore is implemented, preserve saved tab titles and selected tab.

## Test Coverage To Preserve

Grex tests cover:

- one initial selected tab
- injected search service construction
- observable tab collection type
- add-tab count, title, and selection
- multiple add-tab titles
- remove-tab prevention for last tab
- selected tab after removing selected and first tabs
- `CanRemoveTab` false/true/false transitions
- `SelectedTab` property change behavior
- `Tabs` and `CanRemoveTab` property-change notifications through API calls
- direct collection mutation not raising `MainViewModel.PropertyChanged`
- integration search in a newly added tab

Grexa test requirements:

- Unit-test tab store/controller transitions in Rust.
- Integration-test QML tab model signals for add/remove/select.
- Test removal of invalid/non-owned tab ids.
- Test closing a tab with an active search cancels the search and releases
  resources.
- Test selected-tab restore if session persistence is implemented.

## Current Grexa Status

`apps/grexa-gui` is no longer a placeholder binary. The Search page owns an
in-session QML tab model, starts with an initial Search tab, supports
add/remove/select flows, and stores per-tab result snapshots through
`SearchController`.

Remaining gaps:

- move tab lifecycle coverage into GUI/QML automation
- test close/remove behavior while a search or replace is active
- add many-tabs stress coverage
- decide whether session persistence is in scope and test restore behavior if
  it lands
