# Grex Settings View Audit

This document records Grex `Controls/SettingsView.xaml`,
`Controls/SettingsView.xaml.cs`, and the settings persistence surface that
Grexa must preserve, replace, or deliberately redesign for Linux.

Source evidence:

- `Controls/SettingsView.xaml`
- `Controls/SettingsView.xaml.cs`
- `Services/SettingsService.cs`
- `Tests/Services/SettingsServiceTests.cs`
- `Tests/Controls/SettingsViewAiEndpointHelpersTests.cs`
- `IntegrationTests/AiSearchSettingsIntegrationTests.cs`
- `IntegrationTests/AiSearchLocalizationIntegrationTests.cs`
- `UITests/AiSearchUiWiringTests.cs`
- `crates/grexa-core/src/storage.rs`
- `crates/grexa-ai/src/lib.rs`

Note: the Settings files themselves are clean in the current Grex worktree.
The repository still has unrelated active AI Search UI changes in
`SearchTabContent.xaml`, `SearchTabContent.xaml.cs`, and
`UITests/AiSearchUiWiringTests.cs`.

## Top-Level Shape

`SettingsView` is a WinUI `UserControl` named `SettingsControl`.

The visual hierarchy is:

- padded root grid
- vertical `ScrollViewer`
- header stack
- four-column `SettingsFormGrid`
- grouped sections in fixed row order

Sections in display order:

1. header
2. theme preference
3. UI language
4. filter options
5. string comparison
6. Docker search
7. AI search
8. context preview
9. backup and restore
10. debug

The code-behind is not a passive view. It loads and saves settings directly,
manually refreshes localized text, applies custom theme colors, shows dialogs,
tests notifications, tests localization, normalizes AI endpoint URLs, and
starts app restarts.

Grexa replacement:

- Keep Settings as a dense, scrollable native KDE settings surface.
- Move non-UI behavior into Rust services where possible.
- Keep QML/C++ glue responsible for widgets, dialogs, and native platform
  integration only.
- Treat settings updates as instant-save, matching Grex.

## Persistence Contract

Grex `SettingsService` stores JSON at:

```text
%LocalAppData%\Grex\settings.json
```

The service behavior:

- caches a `DefaultSettings` instance in memory
- invalidates cache on demand
- creates the parent directory on save
- ignores load, save, and delete failures
- deserializes with case-insensitive property names
- skips JSON comments
- allows trailing commas
- exports indented JSON
- deletes the settings file for restore defaults

Default values:

- `IsRegexSearch`: false
- `IsFilesSearch`: false
- `RespectGitignore`: false
- `SearchCaseSensitive`: false
- `IncludeSystemFiles`: false
- `IncludeSubfolders`: true
- `IncludeHiddenItems`: false
- `IncludeBinaryFiles`: false
- `IncludeSymbolicLinks`: false
- `UseWindowsSearchIndex`: false
- `EnableDockerSearch`: false
- `SizeUnit`: KB
- `ThemePreference`: `GentleGecko`
- `UILanguage`: `en-US`
- `StringComparisonMode`: `Ordinal`
- `UnicodeNormalizationMode`: `None`
- `DiacriticSensitive`: true
- `Culture`: current process culture
- `DefaultMatchFiles`: empty
- `DefaultExcludeDirs`: empty
- all content and files result columns visible
- window size: 1100x700
- context preview lines before and after: 5
- AI endpoint: `https://api.openai.com/v1`
- AI API key: empty
- AI model: `gpt-4o-mini`

Import/export details:

- Import returns `(Success, ErrorMessage)`.
- Invalid JSON and `null` fail.
- Unknown properties are ignored.
- `WindowX`, `WindowY`, `WindowWidth`, and `WindowHeight` are intentionally
  not copied during import, despite being part of the settings object.
- The implementation comments call import a merge, but missing boolean and enum
  properties are deserialized as `DefaultSettings` defaults and then copied over
  current settings. Grexa should not rely on partial JSON preserving current
  booleans unless it explicitly implements that behavior.
- AI endpoint and model are trimmed on import; AI API key is preserved exactly.
- Current Grex exports include `AiSearchApiKey` in plaintext.

Current Grexa state:

- `crates/grexa-core/src/storage.rs` already defines `AppPaths` with XDG
  config, data, cache, and state directories.
- `settings.json` is already located under `$XDG_CONFIG_HOME/grexa`.
- recent paths, search history, and profiles are already routed to
  `$XDG_DATA_HOME/grexa`.
- `DefaultSettings` already contains most search/filter/comparison fields,
  column visibility, window width/height, context preview counts, endpoint, and
  model.
- Grexa settings currently do not store theme preference, AI API key, or window
  x/y position.

Grexa replacement:

- Keep `$XDG_CONFIG_HOME/grexa/settings.json` for portable JSON settings unless
  KDE `KConfig` is deliberately adopted with JSON import/export compatibility.
- Keep `$XDG_DATA_HOME/grexa` for history and profiles.
- Keep `$XDG_CACHE_HOME/grexa/container-mirrors` for container mirrors.
- Support importing Grex backups by translating or ignoring Windows-only keys.
- Do not include secrets in normal settings exports by default; make API key
  backup an explicit user choice.

## Theme Preference

XAML controls:

- `ThemePreferenceTextBlock`
- `ThemePreferenceComboBox`

Visible theme choices:

- Light
- Dark
- Gentle Gecko
- Black Knight
- Diamond
- Dreams
- Paranoid
- Red Velvet
- Subspace
- Tiefling
- Vibes

`ThemePreference.System` still exists in the enum, and localization refresh code
still has a `System` case, but the current dropdown does not include a System
item. `GetThemePreferenceIndex(System)` maps System to Light.

Theme changes:

- save the selected enum immediately
- show a restart prompt
- Secondary button restarts the app with `/settings`
- Primary button applies the theme immediately through `MainWindow`

High-contrast/custom theme handling:

- the Settings view subscribes to `MainWindow.ThemeChanged`
- custom themes manually push brushes into control resources
- text blocks, content presenters, check boxes, buttons, and combo boxes are
  walked through the WinUI visual tree
- returning to Light/Dark clears those overrides

Grexa replacement:

- Prefer a `System`, `Light`, `Dark`, and optional high-contrast/custom theme
  model that respects KDE/Breeze and system color scheme.
- If Grex theme names are retained for migration, map unsupported names to the
  nearest Grexa theme or preserve them as custom palettes.
- Avoid WinUI visual-tree brush walking; use QML themes, palette bindings, and
  KDE color roles.
- Do not require restart for changes that Qt/Kirigami can apply live.

## UI Language

XAML controls:

- `UILanguageHeaderTextBlock`
- `UILanguageLabelTextBlock`
- `UILanguageComboBox`

Behavior:

- available languages are discovered from `Strings/<culture>/Resources.resw`
  under the app base directory or current directory
- if files are unavailable, a hard-coded 100+ culture fallback list is used
- cultures are validated through `CultureInfo`
- combo items display localized `CultureInfo.DisplayName`
- items sort by current display name
- selecting a language saves `UILanguage`
- `LocalizationService.Instance.SetCulture(language)` is called immediately
- a full UI refresh is queued

The code defers saves while the combo box is open so keyboard navigation does
not commit intermediate values.

Grexa replacement:

- Use Qt translation catalog discovery and KDE language conventions.
- Preserve runtime language switching if practical; otherwise make the restart
  requirement explicit.
- Keep the saved language as a BCP 47 culture string for Grex import
  compatibility.
- Use Qt locale display names instead of .NET `CultureInfo`.

## Filter Options

XAML controls:

- `FilterOptionsHeaderTextBlock`
- `DefaultMatchFilesTextBox`
- `DefaultExcludeDirsTextBox`
- `DefaultSearchResultsComboBox`
- `DefaultSearchTypeComboBox`
- `DefaultRespectGitignoreCheckBox`
- `DefaultIncludeSystemFilesCheckBox`
- `DefaultIncludeSubfoldersCheckBox`
- `DefaultUseWindowsSearchCheckBox`
- `DefaultIncludeHiddenItemsCheckBox`
- `DefaultIncludeBinaryFilesCheckBox`
- `DefaultIncludeSymbolicLinksCheckBox`

Behavior:

- search type saves `IsRegexSearch`
- search results saves `IsFilesSearch`
- text boxes save default match files and exclude dirs on every change
- each check box saves its corresponding default immediately
- `_isLoadingSettings` suppresses saves while the view is loading

Grexa replacement:

- Preserve every default search option:
  - content vs files result mode
  - text vs regex search
  - match file names
  - exclude dirs
  - respect `.gitignore`
  - include system files
  - include subfolders
  - include hidden items
  - include binary/searchable document files
  - include symbolic links
- Replace `UseWindowsSearchIndex` with `use_file_index`, backed first by an
  optional Baloo candidate provider.
- Disable file-index use for regex searches, container targets, unavailable
  index services, and paths that cannot be mapped to local files.
- Keep imported Grex `UseWindowsSearchIndex` as a migration input only.
- For Linux `include_system_files`, define pseudo-filesystem behavior for
  `/proc`, `/sys`, `/dev`, `/run`, mounts, package/dependency folders, and
  permission-denied directories.

## String Comparison

XAML controls:

- `StringComparisonHeaderTextBlock`
- `CultureComboBox`
- `DiacriticSensitiveCheckBox`
- `DefaultSearchCaseSensitiveCheckBox`
- `UnicodeNormalizationModeComboBox`
- `StringComparisonModeComboBox`

Modes:

- string comparison: `Ordinal`, `CurrentCulture`, `InvariantCulture`
- Unicode normalization: `None`, `FormC`, `FormD`, `FormKC`, `FormKD`
- diacritic sensitivity: true/false
- case sensitivity: true/false
- culture: BCP 47 culture code

Behavior:

- culture combo is populated from the same culture list as UI language
- culture display names re-localize when UI language changes
- the string comparison culture is updated to match the new UI language during
  refresh when needed
- dropdown-open flags defer culture saves during keyboard navigation
- `SettingsService.GetContextPreviewLinesBefore/After` clamp values elsewhere,
  but string comparison setters do not validate enum values

Implementation risk to preserve intentionally or fix:

- `StringComparisonModeComboBox` and `UnicodeNormalizationModeComboBox` declare
  string `Tag` values in XAML, while the code casts `selectedItem.Tag` to enum
  values. Grexa should map these choices explicitly instead of relying on UI tag
  coercion.

Grexa replacement:

- Keep the settings schema values and search behavior.
- Use ICU or a clearly documented Rust/Qt equivalent for culture-aware text
  comparison.
- Preserve Grex's rule from the search audit that regex search honors case
  sensitivity but ignores culture, Unicode normalization, and diacritic
  settings.
- Add fixtures for Turkish case, composed/decomposed accents, invariant vs
  current culture, and diacritic-insensitive matching.

## Docker Search

XAML controls:

- `DockerSettingsHeaderTextBlock`
- `DockerSettingsDescriptionTextBlock`
- `EnableDockerSearchLabelTextBlock`
- `EnableDockerSearchToggleSwitch`

Behavior:

- toggling the switch saves `EnableDockerSearch`
- `SettingsService.SetEnableDockerSearch` raises
  `DockerSearchEnabledChanged` only when the value changes
- `TabViewModel` listens to this event to refresh container availability and
  target selection

Grexa replacement:

- Replace the user-facing concept with container search, supporting Docker and
  Podman.
- Preserve a global enable toggle so local-only users do not see container
  target UI by default.
- Keep the value-change event or equivalent reactive signal.
- Explain container socket privilege risk in Settings.
- Keep search read-only for containers.

## AI Search

XAML controls:

- `AiSearchSettingsHeaderTextBlock`
- `AiSearchSettingsDescriptionTextBlock`
- `AiSearchEndpointTextBox`
- `AiSearchApiKeyPasswordBox`
- `AiSearchModelTextBox`
- `TestAiEndpointButton`

Saved fields:

- `AiSearchEndpoint`
- `AiSearchApiKey`
- `AiSearchModel`

Save behavior:

- endpoint is trimmed
- API key is preserved exactly, including spaces
- model is trimmed
- values are saved on every change

Test endpoint behavior:

- blank endpoint shows an error dialog
- missing scheme defaults to `https://`
- trailing slash is removed
- endpoint ending in `/models` is used as-is
- endpoint ending in `/v1` becomes `/v1/models`
- any other endpoint becomes `/v1/models`
- request is `GET`
- optional API key is sent as `Authorization: Bearer <trimmed-key>`
- timeout is 20 seconds
- success displays the tested endpoint and first nonblank `data[].id`
- unknown model displays a localized unknown-model string
- OpenAI-style `{ "error": { "message": "..." } }` is preferred for errors
- string `{ "error": "..." }` payloads are also supported
- fallback error is the HTTP reason phrase, then `Request failed.`

Grexa replacement:

- Keep OpenAI-compatible endpoint, optional API key, optional model, and
  `/v1/models` test/discovery.
- Align endpoint normalization with `crates/grexa-ai/src/lib.rs`; the existing
  Grexa helper strips `/v1` and rebuilds `/v1/models`, which is compatible for
  common endpoint shapes but should be tested against Grex helper cases.
- Store API keys outside normal exported settings, ideally through KDE Wallet or
  another explicit secret mechanism.
- Add an explicit opt-in and context preview before sending local path, query,
  filters, and result context.
- Keep API keys out of logs, screenshots, diagnostics, and default exports.

## Context Preview

XAML controls:

- `ContextPreviewHeaderTextBlock`
- `ContextPreviewLinesBeforeNumberBox`
- `ContextPreviewLinesAfterNumberBox`

Behavior:

- WinUI `NumberBox`
- minimum 1
- maximum 20
- small change 1
- large change 5
- values save immediately
- service getters and setters clamp values to `1..20`
- defaults are 5 before and 5 after

Grexa replacement:

- Preserve the `1..20` clamp and default values.
- Use these settings for local previews, mirrored container previews, and direct
  container previews when available.
- Add boundary tests for start of file, end of file, short files, UTF-8,
  UTF-16, missing files, and permission errors.

## Backup And Restore

XAML controls:

- `BackupRestoreHeaderTextBlock`
- `BackupRestoreExplanationTextBlock`
- `ExportSettingsButton`
- `ImportSettingsButton`
- `RestoreDefaultsButton`

Export behavior:

- uses Windows `FileSavePicker`
- starts in Documents
- restricts file type to `.json`
- suggests `settings_yyyy_MM_dd_H_mm_ss`
- writes `SettingsService.ExportSettingsAsJson()`
- shows success or error dialog

Import behavior:

- uses Windows `FileOpenPicker`
- restricts file type to `.json`
- reads full file text
- calls `SettingsService.ImportSettingsFromJson`
- on success, shows dialog, saves window position, and restarts the app
- on failure, shows localized error

Restore defaults behavior:

- confirmation dialog with Yes/No
- deletes the settings file
- restarts the app

Grexa replacement:

- Use KDE/portal file dialogs.
- Preserve import/export/restore defaults as first-class Settings actions.
- Preserve Grex import tolerance for case-insensitive names, comments, trailing
  commas, and unknown properties where feasible.
- Make secret export explicit.
- Replace Windows app restart APIs with Qt/KDE restart handling or avoid restart
  when settings can be applied live.

## Debug Section

XAML controls:

- `DebugHeaderTextBlock`
- `RestartApplicationButton`
- `TestNotificationButton`
- `TestLocalizationButton`
- explanatory text blocks

Restart behavior:

- saves the window position
- restarts the app with `/settings`

Notification diagnostics:

- runs `NotificationService.CheckRegistration`
- runs `NotificationService.CheckSupport`
- reports support/registration failure details
- sends a test notification when possible
- logs to `%LocalAppData%\Grex\notification_test.log`

Localization diagnostics:

- tests `en-US`, `de-DE`, `es-ES`, and `fr-FR`
- checks a hard-coded localization key list
- reports missing keys by notification
- restores the original culture

Grexa replacement:

- Keep restart only if a setting genuinely needs it.
- Move notification diagnostics to a developer/debug subsection or diagnostics
  page.
- Use KDE `KNotifications` or Freedesktop notifications and log to
  `$XDG_STATE_HOME/grexa`.
- Replace hard-coded localization key testing with translation catalog tests in
  the build/test suite.

## Localization And Tooltips

`SettingsView` has both XAML `x:Uid` localization and manual refresh code.

Manual refresh updates:

- section headers and descriptions
- labels
- combo box items
- check box content
- toggle switch on/off content
- button content
- password/text box placeholders
- dialog button labels
- tooltips registered through `LocalizedToolTipRegistry`

Refresh behavior:

- `LocalizationService.PropertyChanged` triggers `RefreshUI`
- refresh is queued through `DispatcherQueue`
- a short delay allows culture propagation
- settings are reloaded
- registered tooltips are refreshed
- layout is invalidated and updated

Grexa replacement:

- Use Qt translation bindings where possible instead of manual widget-by-widget
  refresh.
- Preserve localized tooltips and accessible labels for all Settings controls.
- Add tests for placeholder text, button text, tooltip keys, and plural/
  placeholder correctness.

## Pointer Cursor Handling

The code-behind has repeated pointer-enter and pointer-exit handlers for radio
buttons, combo boxes, combo box items, check boxes, buttons, and text boxes.

Behavior:

- uses reflection to set WinUI `UIElement.ProtectedCursor`
- falls back to setting the control cursor
- buttons/check boxes/combos use hand cursor
- text boxes use I-beam cursor

Grexa replacement:

- Drop this WinUI-specific reflection layer.
- Use normal Qt cursor properties only where native controls do not already do
  the right thing.

## Windows-Specific Items To Replace

Windows-only APIs and concepts in this surface:

- `%LocalAppData%\Grex\settings.json`
- `%LocalAppData%\Grex\notification_test.log`
- `UseWindowsSearchIndex`
- Windows `FileSavePicker` and `FileOpenPicker`
- WinRT picker window initialization
- `Microsoft.Windows.AppLifecycle.AppInstance.Restart`
- WinUI `ContentDialog`
- WinUI `x:Uid` localization refresh behavior
- `ProtectedCursor` reflection
- Windows notification diagnostics

Grexa replacements:

- XDG config/data/cache/state paths
- optional Baloo-backed file index
- KDE/portal dialogs
- Qt/KDE restart or live-apply behavior
- Kirigami dialogs
- Qt translation catalog refresh
- native Qt cursors
- KNotifications/Freedesktop notifications

## Test Coverage To Preserve

Existing Grex tests cover:

- valid settings JSON export
- formatted export
- export of modified search/theme/language settings
- AI settings export and import
- invalid, empty, and null import errors
- unknown property tolerance
- case-insensitive property names
- trailing commas/comments
- settings delete and cache invalidation
- search settings round trip
- Docker enable default, persistence, and event firing
- no Docker event when value does not change
- all high-contrast theme enum values and JSON numeric round trips
- AI endpoint helper normalization and error extraction
- XAML presence of AI Settings controls
- AI Settings localization keys in resource files

Grexa should add tests for:

- XDG path selection under explicit env vars
- Grex backup import and Windows-only key translation
- secret export exclusion by default
- `UseWindowsSearchIndex` to `use_file_index` migration
- `EnableDockerSearch` to container search migration
- partial import semantics that are deliberately chosen and documented
- context preview clamp behavior
- theme migration from Grex custom theme names

## Current Grexa Status

Grexa now covers XDG paths, theme preference persistence, AI endpoint/model
settings, Secret-Service-backed API key storage, container search settings,
context preview settings, editor/replace/privacy/accessibility toggles, and
Grex-compatible settings export/import behavior in `grexa-core`.

Remaining gaps:

- add import/export/restore controls to the Settings UI
- add an explicit settings schema version or migration framework
- add UI for file-index eligibility and column visibility if those features
  remain in scope
- complete localized Settings tooltip/accessibility coverage
- add notification diagnostics replacement
- broaden Grex-to-Grexa settings import edge-case tests
