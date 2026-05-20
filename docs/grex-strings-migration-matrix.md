# Grex Strings Migration Matrix

This document records every resource key in Grex's canonical English string
table and the decision Grexa makes for it. The canonical source is
`Strings/en-US/Resources.resw` in the Grex repository, which holds 473 keys.
The 107 locale folders under `Strings/` (`af-ZA`, `am-ET`, `ar-SA`, `as-IN`,
`az-AZ`, `be-BY`, `bg-BG`, `bn-BD`, `bo-CN`, `bs-BA`, `ca-ES`, `ceb-PH`,
`cs-CZ`, `cy-GB`, `da-DK`, `de-DE`, `el-GR`, `en-US`, `es-ES`, `et-EE`,
`eu-ES`, `fa-IR`, `fi-FI`, `fil-PH`, `fj-FJ`, `fr-FR`, `ga-IE`, `gl-ES`,
`gu-IN`, `ha-NG`, `haw-US`, `he-IL`, `hi-IN`, `hr-HR`, `hu-HU`, `hy-AM`,
`id-ID`, `ig-NG`, `is-IS`, `it-IT`, `ja-JP`, `jv-Latn-ID`, `ka-GE`, `kk-KZ`,
`km-KH`, `kn-IN`, `ko-KR`, `ky-KG`, `lb-LU`, `lo-LA`, `lt-LT`, `lv-LV`,
`mg-MG`, `mi-NZ`, `mk-MK`, `ml-IN`, `mn-MN`, `mr-IN`, `ms-MY`, `mt-MT`,
`my-MM`, `ne-NP`, `nl-NL`, `no-NO`, `nr-Latn-ZA`, `nso-Latn-ZA`, `or-IN`,
`pa-IN`, `pl-PL`, `pt-BR`, `pt-PT`, `ro-RO`, `ru-RU`, `rw-RW`, `si-LK`,
`sk-SK`, `sl-SI`, `sm-WS`, `sn-Latn-ZW`, `so-SO`, `sq-AL`, `sr-Latn-RS`,
`ss-Latn-ZA`, `st-Latn-ZA`, `su-Latn-ID`, `sv-SE`, `sw-KE`, `ta-IN`, `te-IN`,
`tg-TJ`, `th-TH`, `tk-TM`, `tn-Latn-ZA`, `to-TO`, `tr-TR`, `ts-Latn-ZA`,
`ty-Latn-PF`, `ug-CN`, `uk-UA`, `ur-PK`, `uz-UZ`, `ve-Latn-ZA`, `vi-VN`,
`xh-ZA`, `yo-NG`, `zh-CN`, `zh-TW`, `zu-ZA`) follow this canonical schema and
are not re-listed here; Grexa carries forward whichever translations exist
once the key set is decided.

Source evidence:

- `Strings/en-US/Resources.resw`
- `docs/linux-decisions.md` for the Windows-vs-Linux behavioral cut
- `docs/grex-search-tab-content-xaml-audit.md` for the Search tab keys
- `docs/grex-settings-view-audit.md` for the Settings keys
- `docs/grex-regex-builder-audit.md` for the Regex Builder keys
- `docs/grex-about-view-audit.md` for the About keys
- `docs/grex-ai-search-service-audit.md` for the AI Search keys
- `docs/grex-context-preview-audit.md` for the Context Preview keys
- `docs/grex-wsl-audit.md` for WSL strings being removed

Status legend:

- `keep` — string carries forward into Grexa with the same key and the same
  English text. The translation file content can be reused.
- `rename-key` — concept survives but the key is renamed to drop a Windows
  reference (for example, `UseWindowsSearchCheckBox.Content` →
  `UseFileIndexCheckBox.Content`). Translations are re-keyed on import.
- `remove-windows-only` — the entire string is dropped because the underlying
  UI element or behavior is gone (WSL, Windows toast diagnostics, Windows
  shell verbs, MSIX runtime troubleshooting).
- `add-linux-only` — Grexa needs a new string that Grex never had (Baloo
  status, KIO-FUSE warning, Podman socket diagnostics, Open in Kate, etc.).
  These are listed in their own section at the bottom and not in the main
  table.

Placeholder convention:

- Grex uses `.NET` composite formatting (`{0}`, `{1}`). Grexa keeps the same
  placeholder syntax for parity with imported translation files; the loader
  re-targets `{0}` to whatever templating engine the QML side uses.
- Strings that embed a count are flagged in the Notes column. The
  recommendation is documented at the end of this file.

## Search Tab

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `SearchTab` | Search | Search | keep | nav item label |
| `SearchNavItem.Content` | Search | Search | keep | duplicates `SearchTab`, kept for binding parity |
| `SearchPathLabel` | Search Path | Search Path | keep |  |
| `SearchPathPlaceholder` | Enter path to search... | Enter path to search... | keep |  |
| `PathAutoSuggestBox.PlaceholderText` | Enter or paste path here... | Enter or paste path here... | keep |  |
| `BrowseButton.Content` | Browse... | Browse... | keep | xdg-portal file chooser |
| `Controls.SearchTabContent.PathAutoSuggestBox.ToolTip` | Choose the folder or drive to scan. Supports pasting paths and shows recent locations. | Choose the folder to scan. Supports pasting paths and shows recent locations. | rename-key | drop "or drive"; Linux has no drives |
| `Controls.SearchTabContent.BrowseButton.ToolTip` | Open the folder picker to select a search root. | Open the folder picker to select a search root. | keep |  |
| `SearchPatternLabel` | Search Pattern | Search Pattern | keep |  |
| `SearchPatternPlaceholder` | Enter text or Regex pattern... | Enter text or Regex pattern... | keep |  |
| `SearchTextBox.PlaceholderText` | Enter search term or Regex pattern... | Enter search term or Regex pattern... | keep |  |
| `Controls.SearchTabContent.SearchTextBox.ToolTip` | Type the plain-text term or Regex pattern you want to search for. | Type the plain-text term or Regex pattern you want to search for. | keep |  |
| `ReplacePatternLabel` | Replace With | Replace With | keep |  |
| `ReplacePatternPlaceholder` | Enter replacement text... | Enter replacement text... | keep |  |
| `ReplaceCheckBox.Content` | Replace | Replace | keep |  |
| `ReplaceWithTextBox.PlaceholderText` | Replace with... | Replace with... | keep |  |
| `Controls.SearchTabContent.ReplaceCheckBox.ToolTip` | Enable replace mode so matches can be updated instead of only listed. | Enable replace mode so matches can be updated instead of only listed. | keep |  |
| `Controls.SearchTabContent.ReplaceWithTextBox.ToolTip` | Provide the replacement text that will be written when replace runs. | Provide the replacement text that will be written when replace runs. | keep |  |
| `SearchButton` | Search | Search | keep |  |
| `ReplaceButton` | Replace | Replace | keep |  |
| `StopButton` | Stop | Stop | keep |  |
| `AppBarSearchButton.Label` | Search | Search | keep |  |
| `AppBarReplaceButton.Label` | Replace | Replace | keep |  |
| `AppBarResetButton.Label` | Reset | Reset | keep |  |
| `AppBarExportButton.Label` | Export | Export | keep |  |
| `AppBarProfilesButton.Label` | Profiles | Profiles | keep |  |
| `FilterOptionsToggleButton.Label` | Filter Options | Filter Options | keep |  |
| `MatchFilesTextBlock.Text` | Match Files: | Match files: | keep | sentence-case drift in original; harmless |
| `MatchFileNamesTextBox.PlaceholderText` | e.g., *.json\|*.txt or -*.log | e.g., *.json\|*.txt or -*.log | keep |  |
| `Controls.SearchTabContent.MatchFileNamesTextBox.ToolTip` | Filter files by name. Separate patterns with '\|' and prefix with '-' to exclude. | Filter files by name. Separate patterns with '\|' and prefix with '-' to exclude. | keep |  |
| `FileFiltersLabel` | File Filters | File Filters | keep |  |
| `FileFiltersPlaceholder` | *.cs;*.txt;*.md (semicolon separated) | *.rs;*.txt;*.md (semicolon separated) | keep | example pattern can be updated to a Linux-typical extension during translation review |
| `ExcludeDirectoriesLabel` | Exclude Directories | Exclude Directories | keep |  |
| `ExcludeDirectoriesPlaceholder` | bin;obj;node_modules (semicolon separated) | target;node_modules;.cache (semicolon separated) | keep | example list updated for Linux ecosystems |
| `ExcludeDirsTextBlock.Text` | Exclude Dirs: | Exclude Dirs: | keep |  |
| `ExcludeDirsTextBox.PlaceholderText` | e.g., .git,vendor or ^(.git\|vendor)$ | e.g., .git,target or ^(.git\|target)$ | keep | swap `vendor` example for `target` (Rust) or keep both |
| `Controls.SearchTabContent.ExcludeDirsTextBox.ToolTip` | Skip directories using comma-separated names or a Regex (e.g., ^(.git\|vendor)$). | Skip directories using comma-separated names or a Regex (e.g., ^(.git\|target)$). | keep |  |
| `SearchTypeTextBlock.Text` | Search Type: | Search Type: | keep |  |
| `TextSearchComboBoxItem.Content` | Text Search | Text Search | keep |  |
| `RegexSearchComboBoxItem.Content` | Regex Search | Regex Search | keep |  |
| `Controls.SearchTabContent.SearchTypeComboBox.ToolTip` | Choose whether the search interprets input as plain text or as a Regex. | Choose whether the search interprets input as plain text or as a Regex. | keep |  |
| `SearchResultsTextBlock.Text` | Search Results: | Search Results: | keep |  |
| `SearchResultsLabelTextBlock.Text` | Search Results: | Search Results: | keep | duplicate of above; preserve for binding |
| `ContentMode` | Content | Content | keep |  |
| `FilesMode` | Files | Files | keep |  |
| `ContentComboBoxItem.Content` | Content | Content | keep |  |
| `FilesComboBoxItem.Content` | Files | Files | keep |  |
| `Controls.SearchTabContent.SearchResultsComboBox.ToolTip` | Choose whether results show matching lines or just the list of files. | Choose whether results show matching lines or just the list of files. | keep |  |
| `ResultsLabel` | Results | Results | keep |  |
| `ResultsFilterTextBox.PlaceholderText` | Search within results... | Search within results... | keep |  |
| `ResultsFilterRegexToggle.ToolTipService.ToolTip` | Use regular expression for filtering | Use regular expression for filtering | keep |  |
| `Controls.SearchTabContent.ResultsFilterTextBox.ToolTip` | Filter the current result set locally without re-scanning the filesystem. | Filter the current result set locally without re-scanning the filesystem. | keep |  |
| `Controls.SearchTabContent.ResultsRowTooltip.LinePrefixFormat` | Line {0}:  | Line {0}:  | keep | uses {0} placeholder |
| `SearchTypeLabelTextBlock.Text` | Search Type: | Search Type: | keep | duplicate of `SearchTypeTextBlock.Text` |
| `FilterOptionsHeaderTextBlock.Text` | Filter Options | Filter Options | keep |  |
| `SizeLimitTextBlock.Text` | Size Limit: | Size Limit: | keep |  |
| `SizeFilterLabel` | Size Filter | Size Filter | keep |  |
| `MinSizeLabel` | Min Size | Min Size | keep |  |
| `MaxSizeLabel` | Max Size | Max Size | keep |  |
| `NoLimitComboBoxItem.Content` | No Limit | No Limit | keep |  |
| `LessThanComboBoxItem.Content` | Less Than | Less Than | keep |  |
| `EqualToComboBoxItem.Content` | Equal To | Equal To | keep |  |
| `GreaterThanComboBoxItem.Content` | Greater Than | Greater Than | keep |  |
| `SizeLimitNumberBox.PlaceholderText` | Enter size | Enter size | keep |  |
| `KBComboBoxItem.Content` | KB | KB | keep |  |
| `MBComboBoxItem.Content` | MB | MB | keep |  |
| `GBComboBoxItem.Content` | GB | GB | keep |  |
| `Controls.SearchTabContent.SizeLimitComboBox.ToolTip` | Apply an optional size comparison before scanning each file. | Apply an optional size comparison before scanning each file. | keep |  |
| `Controls.SearchTabContent.SizeLimitNumberBox.ToolTip` | Enter the numeric size threshold that works with the selected comparison. | Enter the numeric size threshold that works with the selected comparison. | keep |  |
| `Controls.SearchTabContent.SizeUnitComboBox.ToolTip` | Select the unit (KB, MB, GB) for the size limit. | Select the unit (KB, MB, GB) for the size limit. | keep |  |
| `CaseSensitiveLabel` | Case Sensitive | Case Sensitive | keep |  |
| `UseRegexLabel` | Use Regular Expression | Use Regular Expression | keep |  |
| `IncludeSubdirectoriesLabel` | Include Subdirectories | Include Subdirectories | keep |  |
| `FollowSymlinksLabel` | Follow Symbolic Links | Follow Symbolic Links | keep |  |
| `RespectGitignoreCheckBox.Content` | Respect .gitignore | Respect .gitignore | keep |  |
| `SearchCaseSensitiveCheckBox.Content` | Search case-sensitive | Search case-sensitive | keep |  |
| `IncludeSystemFilesCheckBox.Content` | Include system files | Include pseudo-filesystem entries | keep | wording can stay; semantics shift to `/proc`, `/sys`, `/dev`, `/run` guards (`linux-decisions.md`, system path auto-exclusions) |
| `IncludeSubfoldersCheckBox.Content` | Include subfolders | Include subfolders | keep |  |
| `IncludeHiddenItemsCheckBox.Content` | Include hidden items | Include dotfiles | keep | "hidden items" is correct on Linux; "dotfiles" only if branding demands it |
| `IncludeBinaryFilesCheckBox.Content` | Include binary files | Include binary files | keep |  |
| `IncludeSymbolicLinksCheckBox.Content` | Include symbolic links | Follow symbolic links | keep |  |
| `UseWindowsSearchCheckBox.Content` | Use Windows Search | Use file index (Baloo) | rename-key | new key `UseFileIndexCheckBox.Content`; settings field also renamed (`linux-decisions.md`, Windows Search Index) |
| `UseWindowsSearchCheckBox.ToolTipService.ToolTip` | Leverage the Windows Search index for faster text searches on indexed Windows folders. Regex searches, WSL targets, and non-indexed locations will use the traditional scanner. | Use the Baloo file index to seed candidate paths for faster text searches on indexed locations. Regex searches and non-indexed locations always use the traditional scanner. | rename-key | move to `UseFileIndexCheckBox.ToolTipService.ToolTip` |
| `WindowsSearchDisabledTooltip` | Works only for plain-text searches under Windows paths. | Works only for plain-text searches under indexed paths. | rename-key | new key `FileIndexDisabledTooltip` |
| `Controls.SearchTabContent.RespectGitignoreCheckBox.ToolTip` | Skip files ignored by .gitignore in the selected directory. | Skip files ignored by .gitignore in the selected directory. | keep |  |
| `Controls.SearchTabContent.SearchCaseSensitiveCheckBox.ToolTip` | Only treat matches as valid when the letter casing matches exactly. | Only treat matches as valid when the letter casing matches exactly. | keep |  |
| `Controls.SearchTabContent.IncludeSystemFilesCheckBox.ToolTip` | Allow system-protected files to be scanned. | Allow files under `/proc`, `/sys`, `/dev`, and `/run` to be scanned. | keep | semantic clarification only |
| `Controls.SearchTabContent.IncludeSubfoldersCheckBox.ToolTip` | Recursively search all subfolders under the chosen path. | Recursively search all subfolders under the chosen path. | keep |  |
| `Controls.SearchTabContent.IncludeHiddenItemsCheckBox.ToolTip` | Include files and folders that are marked as hidden. | Include dotfiles and dot-directories. | keep |  |
| `Controls.SearchTabContent.IncludeBinaryFilesCheckBox.ToolTip` | Scan binary files in addition to text files. This can be slower. | Scan binary files in addition to text files. This can be slower. | keep |  |
| `Controls.SearchTabContent.IncludeSymbolicLinksCheckBox.ToolTip` | Follow symbolic links and junctions while scanning. | Follow symbolic links while scanning. | keep | drop "junctions"; Linux has none |
| `ReadyStatus` | Ready | Ready | keep |  |
| `SearchingStatus` | Searching... | Searching... | keep |  |
| `ReplacingStatus` | Replacing... | Replacing... | keep |  |
| `NoMatchesStatus` | No matches found | No matches found | keep |  |
| `ErrorStatus` | Error: {0} | Error: {0} | keep | uses {0} placeholder |
| `FoundMatchesStatus` | Found {0} matches in {1} files in {2} | Found {0} matches in {1} files in {2} | keep | uses {0} {1} {2} placeholders; **needs plural ICU MessageFormat** for matches and files |
| `FilteredMatchesStatus` | Showing {0} matches in {1} files (filtered from {2} matches in {3} files) in {4} | Showing {0} matches in {1} files (filtered from {2} matches in {3} files) in {4} | keep | five placeholders; **needs plural ICU MessageFormat** |
| `ReplacedMatchesStatus` | Replaced {0} matches in {1} files in {2} | Replaced {0} matches in {1} files in {2} | keep | three placeholders; **needs plural ICU MessageFormat** |
| `StatusInfoBar.Title` | Search Status | Search Status | keep |  |
| `StatusInfoBar.Message` | Ready | Ready | keep |  |
| `SearchHistoryButton.ToolTipService.ToolTip` | View recent searches | View recent searches | keep |  |
| `SearchHistoryTitleTextBlock.Text` | Search History | Search History | keep |  |
| `ClearHistoryButton.Content` | Clear | Clear | keep |  |
| `NoSearchHistoryTextBlock.Text` | No search history | No search history | keep |  |
| `Controls.SearchTabContent.SearchHistoryButton.ToolTip` | View recent searches | View recent searches | keep |  |
| `TabNewTitle` | New Tab | New Tab | keep |  |
| `TabTimestampTitleFormat` | Tab {0} | Tab {0} | keep | uses {0} placeholder |
| `SearchProfilesTitleTextBlock.Text` | Search Profiles | Search Profiles | keep |  |
| `SaveProfileButton.Content` | Save Current... | Save Current... | keep |  |
| `NoSearchProfilesTextBlock.Text` | No saved profiles | No saved profiles | keep |  |
| `SaveSearchProfileTitle` | Save Search Profile | Save Search Profile | keep |  |
| `SearchProfileNamePlaceholder` | Profile name | Profile name | keep |  |
| `SaveSearchProfileErrorTitle` | Cannot Save Profile | Cannot Save Profile | keep |  |
| `SaveSearchProfileEmptyNameMessage` | Enter a profile name. | Enter a profile name. | keep |  |
| `SaveSearchProfileMissingFieldsMessage` | Enter both a search path and a search term. | Enter both a search path and a search term. | keep |  |
| `OverwriteSearchProfileTitle` | Overwrite Profile? | Overwrite Profile? | keep |  |
| `OverwriteSearchProfileMessage` | A profile named "{0}" already exists. Overwrite it? | A profile named "{0}" already exists. Overwrite it? | keep | uses {0} placeholder |
| `DeleteSearchProfileTitle` | Delete Profile? | Delete Profile? | keep |  |
| `DeleteSearchProfileMessage` | Delete the profile "{0}"? | Delete the profile "{0}"? | keep | uses {0} placeholder |

## Results Columns And Header Menu

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `FileNameColumn` | File Name | File Name | keep |  |
| `LineNumberColumn` | Line | Line | keep |  |
| `ContentColumn` | Content | Content | keep |  |
| `PathColumn` | Path | Path | keep |  |
| `SizeColumn` | Size | Size | keep |  |
| `ModifiedColumn` | Modified | Modified | keep |  |
| `ContentNameHeaderButton.Content` | Name | Name | keep |  |
| `ContentLineHeaderButton.Content` | Line | Line | keep |  |
| `ContentColumnHeaderButton.Content` | Column | Column | keep |  |
| `ContentTextHeaderButton.Content` | Text | Text | keep |  |
| `ContentPathHeaderButton.Content` | File Path | File Path | keep |  |
| `FilesNameHeaderButton.Content` | Name | Name | keep |  |
| `FilesSizeHeaderButton.Content` | File Size | File Size | keep |  |
| `FilesMatchesHeaderButton.Content` | Matches | Matches | keep |  |
| `FilesPathHeaderButton.Content` | File Path | File Path | keep |  |
| `FilesExtHeaderButton.Content` | File Extension | File Extension | keep |  |
| `FilesEncodingHeaderButton.Content` | Encoding | Encoding | keep |  |
| `FilesDateModifiedHeaderButton.Content` | Date modified | Date modified | keep |  |
| `HideLineMenuItem.Text` | Hide Line | Hide Line | keep |  |
| `HideColumnMenuItem.Text` | Hide Column | Hide Column | keep |  |
| `HidePathMenuItem.Text` | Hide Path | Hide Path | keep |  |
| `HideSizeMenuItem.Text` | Hide Size | Hide Size | keep |  |
| `HideMatchesMenuItem.Text` | Hide Matches | Hide Matches | keep |  |
| `HideExtMenuItem.Text` | Hide Ext | Hide Ext | keep |  |
| `HideEncodingMenuItem.Text` | Hide Encoding | Hide Encoding | keep |  |
| `HideDateModifiedMenuItem.Text` | Hide Date Modified | Hide Date Modified | keep |  |
| `OpenInExplorerMenuItem` | Show in Explorer | Show in file manager | rename-key | new key `OpenInFileManagerMenuItem`; uses `org.freedesktop.FileManager1.ShowItems` (`linux-decisions.md`, Editor And File Manager Integration) |
| `CopyPathMenuItem` | Copy Path | Copy Path | keep |  |
| `CopyFileNameMenuItem` | Copy File Name | Copy File Name | keep |  |

## Settings — General, UI Language, Theme

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `SettingsTab` | Settings | Settings | keep |  |
| `SettingsNavItem.Content` | Settings | Settings | keep |  |
| `SettingsTitleTextBlock.Text` | Settings | Settings | keep |  |
| `SettingsDescriptionTextBlock.Text` | These settings will be used as defaults for new tabs and on application startup. | These settings will be used as defaults for new tabs and on application startup. | keep |  |
| `GeneralSettings` | General | General | keep |  |
| `LanguageLabel` | Language | Language | keep |  |
| `UILanguageHeaderTextBlock.Text` | UI Language | UI Language | keep |  |
| `UILanguageLabelTextBlock.Text` | Application Language: | Application Language: | keep |  |
| `Controls.SettingsView.UILanguageComboBox.ToolTip` | Switch the application language instantly for every open page. | Switch the application language instantly for every open page. | keep |  |
| `ThemeLabel` | Theme | Theme | keep |  |
| `ThemePreferenceTextBlock.Text` | Theme Preference | Theme Preference | keep |  |
| `ThemeSystem` | System | Follow KDE color scheme | rename-key | text revised; key kept (`linux-decisions.md`, `ThemePreference.System`) |
| `ThemeLight` | Light | Light | keep |  |
| `ThemeDark` | Dark | Dark | keep |  |
| `SystemThemeRadio.Content` | System Theme | Follow KDE color scheme | rename-key | text revised |
| `LightThemeRadio.Content` | Light Mode | Light Mode | keep |  |
| `DarkThemeRadio.Content` | Dark Mode | Dark Mode | keep |  |
| `SystemThemeComboBoxItem.Content` | System Default | Follow KDE color scheme | rename-key | text revised; key kept |
| `LightThemeComboBoxItem.Content` | Light Mode | Light Mode | keep |  |
| `DarkThemeComboBoxItem.Content` | Dark Mode | Dark Mode | keep |  |
| `BlackKnightThemeComboBoxItem.Content` | Black Knight (High Contrast) | Black Knight (High Contrast) | keep | id preserved on disk for round-trip; label can be rebranded per Linux design guidance |
| `ParanoidThemeComboBoxItem.Content` | Paranoid (High Contrast) | Paranoid (High Contrast) | keep |  |
| `DiamondThemeComboBoxItem.Content` | Diamond (High Contrast) | Diamond (High Contrast) | keep |  |
| `SubspaceThemeComboBoxItem.Content` | Subspace (High Contrast) | Subspace (High Contrast) | keep |  |
| `RedVelvetThemeComboBoxItem.Content` | Red Velvet (High Contrast) | Red Velvet (High Contrast) | keep |  |
| `DreamsThemeComboBoxItem.Content` | Dreams (High Contrast) | Dreams (High Contrast) | keep |  |
| `TieflingThemeComboBoxItem.Content` | Tiefling (High Contrast) | Tiefling (High Contrast) | keep |  |
| `VibesThemeComboBoxItem.Content` | Vibes (High Contrast) | Vibes (High Contrast) | keep |  |
| `GentleGeckoThemeComboBoxItem.Content` | Gentle Gecko | Gentle Gecko | keep |  |
| `GentleGecko` | Gentle Gecko (High Contrast) | Gentle Gecko (High Contrast) | keep | duplicate copy of the same idea; kept for binding parity |
| `Controls.SettingsView.ThemePreferenceComboBox.ToolTip` | Choose a theme for the application: System Default follows your Windows settings, Light and Dark are standard themes, and the high-contrast themes (Gentle Gecko, Black Knight, Diamond, Dreams, Paranoid, Red Velvet, Subspace, Tiefling, Vibes) provide enhanced readability and visual variety. | Choose a theme for the application: System Default follows the KDE color scheme, Light and Dark are standard themes, and the high-contrast themes (Gentle Gecko, Black Knight, Diamond, Dreams, Paranoid, Red Velvet, Subspace, Tiefling, Vibes) provide enhanced readability and visual variety. | keep | drop the "Windows settings" reference |
| `Controls.SettingsView.SystemThemeRadio.ToolTip` | Match the Windows theme automatically when Grex starts. | Match the KDE color scheme automatically when Grexa starts. | keep | wording revised |
| `Controls.SettingsView.LightThemeRadio.ToolTip` | Force the light theme regardless of the Windows setting. | Force the light theme regardless of the system setting. | keep | wording revised |
| `Controls.SettingsView.DarkThemeRadio.ToolTip` | Force the dark theme regardless of the Windows setting. | Force the dark theme regardless of the system setting. | keep | wording revised |
| `ApplyThemePromptTitle` | Apply Theme | Apply Theme | keep |  |
| `ApplyThemePromptMessage` | Some elements need the application to be restarted to style properly. | Some elements need the application to be restarted to style properly. | keep |  |
| `LaterButton` | Later | Later | keep |  |
| `RestartApplicationButton.Content` | Restart Application | Restart Application | keep |  |
| `Controls.SettingsView.RestartApplicationButton.ToolTip` | Restart the application to apply changes. | Restart the application to apply changes. | keep |  |

## Settings — Defaults

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `FilterOptionsDefaultsTextBlock.Text` | Filter Options | Filter Options | keep |  |
| `DefaultRespectGitignoreCheckBox.Content` | Respect .gitignore | Respect .gitignore | keep |  |
| `DefaultSearchCaseSensitiveCheckBox.Content` | Search case-sensitive | Search case-sensitive | keep |  |
| `DefaultIncludeSystemFilesCheckBox.Content` | Include system files | Include pseudo-filesystem entries | keep | semantics shift; wording can stay |
| `DefaultIncludeSubfoldersCheckBox.Content` | Include subfolders | Include subfolders | keep |  |
| `DefaultIncludeHiddenItemsCheckBox.Content` | Include hidden items | Include dotfiles | keep |  |
| `DefaultIncludeBinaryFilesCheckBox.Content` | Include binary files | Include binary files | keep |  |
| `DefaultIncludeSymbolicLinksCheckBox.Content` | Include symbolic links | Follow symbolic links | keep |  |
| `DefaultUseWindowsSearchCheckBox.Content` | Use Windows Search | Use file index (Baloo) | rename-key | new key `DefaultUseFileIndexCheckBox.Content` |
| `DefaultUseWindowsSearchCheckBox.ToolTipService.ToolTip` | Use the Windows Search index for faster text searches on indexed folders. | Use the Baloo file index for faster text searches on indexed folders. | rename-key | new key `DefaultUseFileIndexCheckBox.ToolTipService.ToolTip` |
| `Controls.SettingsView.DefaultSearchResultsComboBox.ToolTip` | Choose whether new tabs default to content results or file results. | Choose whether new tabs default to content results or file results. | keep |  |
| `Controls.SettingsView.DefaultSearchTypeComboBox.ToolTip` | Choose whether new tabs start in text search or Regex search mode. | Choose whether new tabs start in text search or Regex search mode. | keep |  |
| `Controls.SettingsView.DefaultRespectGitignoreCheckBox.ToolTip` | Set whether new searches respect .gitignore by default. | Set whether new searches respect .gitignore by default. | keep |  |
| `Controls.SettingsView.DefaultIncludeSystemFilesCheckBox.ToolTip` | Include system-protected files by default. | Include pseudo-filesystem entries by default. | keep |  |
| `Controls.SettingsView.DefaultIncludeSubfoldersCheckBox.ToolTip` | Recurse into subfolders by default. | Recurse into subfolders by default. | keep |  |
| `Controls.SettingsView.DefaultIncludeHiddenItemsCheckBox.ToolTip` | Include hidden files and folders by default. | Include dotfiles and dot-directories by default. | keep |  |
| `Controls.SettingsView.DefaultIncludeBinaryFilesCheckBox.ToolTip` | Scan binary files by default. | Scan binary files by default. | keep |  |
| `Controls.SettingsView.DefaultIncludeSymbolicLinksCheckBox.ToolTip` | Follow symbolic links and junctions by default. | Follow symbolic links by default. | keep | drop "junctions" |
| `Controls.SettingsView.DefaultSearchCaseSensitiveCheckBox.ToolTip` | Make new searches case-sensitive by default. | Make new searches case-sensitive by default. | keep |  |

## Settings — String Comparison

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `StringComparisonHeaderTextBlock.Text` | String Comparison | String Comparison | keep |  |
| `StringComparisonModeTextBlock.Text` | Comparison Mode: | Comparison Mode: | keep |  |
| `StringComparisonModeLabelTextBlock.Text` | Comparison Mode: | Comparison Mode: | keep | duplicate of above |
| `OrdinalComparisonComboBoxItem.Content` | Ordinal (fast, binary) | Ordinal (fast, binary) | keep |  |
| `OrdinalComboBoxItem.Content` | Ordinal | Ordinal | keep |  |
| `CurrentCultureComparisonComboBoxItem.Content` | Current Culture | Current Culture | keep | concept maps to ICU locale on Linux |
| `CurrentCultureComboBoxItem.Content` | Current Culture | Current Culture | keep |  |
| `InvariantCultureComparisonComboBoxItem.Content` | Invariant Culture | Invariant Culture | keep |  |
| `InvariantCultureComboBoxItem.Content` | Invariant Culture | Invariant Culture | keep |  |
| `Controls.SettingsView.StringComparisonModeComboBox.ToolTip` | Select the default string comparison rules for new searches. | Select the default string comparison rules for new searches. | keep |  |
| `UnicodeNormalizationTextBlock.Text` | Unicode Normalization: | Unicode Normalization: | keep |  |
| `UnicodeNormalizationLabelTextBlock.Text` | Unicode Normalization: | Unicode Normalization: | keep | duplicate of above |
| `NoneNormalizationComboBoxItem.Content` | None | None | keep |  |
| `FormCNormalizationComboBoxItem.Content` | Form C (canonical decomposition) | Form C (canonical composition) | keep | typo in Grex source: NFC is canonical composition, not decomposition; Grexa should fix during translation review |
| `FormDNormalizationComboBoxItem.Content` | Form D (canonical decomposition) | Form D (canonical decomposition) | keep |  |
| `FormKCNormalizationComboBoxItem.Content` | Form KC (compatibility decomposition) | Form KC (compatibility composition) | keep | same typo as above |
| `FormKDNormalizationComboBoxItem.Content` | Form KD (compatibility decomposition) | Form KD (compatibility decomposition) | keep |  |
| `Controls.SettingsView.UnicodeNormalizationModeComboBox.ToolTip` | Choose which Unicode normalization is applied before comparisons. | Choose which Unicode normalization is applied before comparisons. | keep |  |
| `DiacriticSensitiveCheckBox.Content` | Diacritic Sensitive | Diacritic Sensitive | keep |  |
| `DiacriticSensitiveCheckBox.ToolTipService.ToolTip` | When disabled, accented characters are treated as their base characters (e.g., 'é' = 'e') | When disabled, accented characters are treated as their base characters (e.g., 'é' = 'e') | keep |  |
| `CultureTextBlock.Text` | Culture: | Culture: | keep |  |
| `CultureLabelTextBlock.Text` | String Culture: | String Culture: | keep |  |
| `CultureComboBox.PlaceholderText` | Select culture or enter custom culture code | Select locale or enter custom locale code | rename-key | new key `LocaleComboBox.PlaceholderText` matching ICU naming |
| `CultureComboBoxToolTip.Content` | Used for string comparison settings during search operations, not for the application UI language. | Used for string comparison settings during search operations, not for the application UI language. | keep |  |

## Settings — Context Preview

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `ContextPreviewHeaderTextBlock.Text` | Context Preview | Context Preview | keep |  |
| `ContextPreviewLinesBeforeLabelTextBlock.Text` | Lines before match | Lines before match | keep |  |
| `ContextPreviewLinesAfterLabelTextBlock.Text` | Lines after match | Lines after match | keep |  |

## Settings — Docker (becomes Containers on Linux)

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `DockerSettingsHeaderTextBlock.Text` | Docker Search | Container Search | rename-key | new key `ContainerSettingsHeaderTextBlock.Text`; Linux supports Docker, rootless Podman, rootful Podman (`linux-decisions.md`, Containers) |
| `DockerSettingsDescriptionTextBlock.Text` | Natively search Linux containers using the Docker CLI. If grep is unavailable in the docker container, search by mirroring files from a running Docker container so Grex can scan them without leaving the app. | Natively search Linux containers using the Docker or Podman CLI. If grep is unavailable inside the container, files are mirrored from the running container so Grexa can scan them without leaving the app. | rename-key | new key `ContainerSettingsDescriptionTextBlock.Text` |
| `EnableDockerSearchLabelTextBlock.Text` | Enable Docker Search | Enable Container Search | rename-key | new key `EnableContainerSearchLabelTextBlock.Text` |
| `EnableDockerSearchToggleSwitch.OnContent` | Enabled | Enabled | rename-key | new key `EnableContainerSearchToggleSwitch.OnContent` |
| `EnableDockerSearchToggleSwitch.OffContent` | Disabled | Disabled | rename-key | new key `EnableContainerSearchToggleSwitch.OffContent` |
| `Controls.SettingsView.EnableDockerSearchToggleSwitch.ToolTip` | Show available Docker containers next to the search path so you can target them directly. | Show available Docker and Podman containers next to the search path so you can target them directly. | rename-key | new key `Controls.SettingsView.EnableContainerSearchToggleSwitch.ToolTip` |
| `DockerLocalDiskOption` | Local Disk | Local Disk | keep |  |
| `DockerTargetComboBox.Header` | Search Target | Search Target | rename-key | new key `ContainerTargetComboBox.Header` |
| `DockerTargetComboBox.PlaceholderText` | Select container | Select container | rename-key | new key `ContainerTargetComboBox.PlaceholderText` |
| `DockerRefreshButton.ToolTipService.ToolTip` | Refresh Docker container list | Refresh container list | rename-key | new key `ContainerRefreshButton.ToolTipService.ToolTip` |
| `DockerInitializationErrorMessage` | Could not initialize Docker integration: {0} | Could not initialize container integration: {0} | rename-key | new key `ContainerInitializationErrorMessage`; uses {0} placeholder |
| `DockerRefreshErrorMessage` | Unable to refresh Docker containers: {0} | Unable to refresh containers: {0} | rename-key | new key `ContainerRefreshErrorMessage`; uses {0} placeholder |
| `DockerMirrorErrorMessage` | Failed to prepare the Docker search mirror: {0} | Failed to prepare the container search mirror: {0} | rename-key | new key `ContainerMirrorErrorMessage`; uses {0} placeholder |
| `DockerSymlinkErrorMessage` | Docker copy failed due to symbolic link creation: {0}\n\nWindows requires special privileges to create symbolic links. Either run Grex as Administrator, or enable the 'Create symbolic links' user right for your account.\n\nNote: Some container paths (like node_modules) contain many symlinks and may not be searchable without these privileges. | (replaced) | remove-windows-only | the Windows symlink privilege story does not exist on Linux; replace with a new message that explains rootless/rootful permissions if container copy fails |
| `DockerContextCopyPath` | Copy container path | Copy container path | keep |  |
| `DockerContextCopyName` | Copy file name | Copy file name | keep |  |
| `DockerSearchErrorTitle` | Docker Search Error | Container Search Error | rename-key | new key `ContainerSearchErrorTitle` |
| `DockerUnavailableMessage` | Docker is unavailable. Make sure Docker Desktop is running and that you have permission to run docker commands. | Container runtime is unavailable. Make sure the Docker or Podman socket is reachable and that your user has permission to run container commands. | rename-key | new key `ContainerUnavailableMessage`; Docker Desktop is Windows-specific (`linux-decisions.md`, Containers) |
| `DockerContainerNotSelectedMessage` | Select a Docker container before starting the search. | Select a container before starting the search. | rename-key | new key `ContainerNotSelectedMessage` |

## Settings — Backup, Restore, Export

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `BackupRestoreHeaderTextBlock.Text` | Backup & Restore | Backup & Restore | keep |  |
| `ExportSettingsButton.Content` | Export Settings | Export Settings | keep |  |
| `ImportSettingsButton.Content` | Import Settings | Import Settings | keep |  |
| `RestoreDefaultsButton.Content` | Restore Defaults | Restore Defaults | keep |  |
| `BackupRestoreExplanationTextBlock.Text` | Export your current settings to a backup file, import settings from a previous backup, or restore all settings to their original defaults. | Export your current settings to a backup file, import settings from a previous backup, or restore all settings to their original defaults. | keep |  |
| `SettingsExportedSuccessTitle` | Settings Exported | Settings Exported | keep |  |
| `SettingsExportedSuccessMessage` | Your settings have been exported successfully to: {0} | Your settings have been exported successfully to: {0} | keep | uses {0} placeholder |
| `SettingsExportErrorTitle` | Export Failed | Export Failed | keep |  |
| `SettingsExportErrorMessage` | Failed to export settings: {0} | Failed to export settings: {0} | keep | uses {0} placeholder |
| `SettingsImportedSuccessTitle` | Settings Imported | Settings Imported | keep |  |
| `SettingsImportedSuccessMessage` | Settings have been imported successfully. Some changes may require a restart to take effect. | Settings have been imported successfully. Some changes may require a restart to take effect. | keep |  |
| `SettingsImportErrorTitle` | Import Failed | Import Failed | keep |  |
| `SettingsImportErrorMessage` | Failed to import settings: {0} | Failed to import settings: {0} | keep | uses {0} placeholder |
| `SettingsImportInvalidFileMessage` | The selected file does not appear to be a valid Grex settings backup. | The selected file does not appear to be a valid Grexa settings backup, or Grex settings ready for import. | keep | reuse for both formats; Grex import is the migration story (`linux-decisions.md`, Imports From Grex Backups) |
| `RestoreDefaultsConfirmTitle` | Restore Defaults | Restore Defaults | keep |  |
| `RestoreDefaultsConfirmMessage` | This will reset all settings to their original defaults and restart the application. Are you sure you want to continue? | This will reset all settings to their original defaults and restart the application. Are you sure you want to continue? | keep |  |
| `Controls.SettingsView.ExportSettingsButton.ToolTip` | Export your current settings to a backup file | Export your current settings to a backup file | keep |  |
| `Controls.SettingsView.ImportSettingsButton.ToolTip` | Import settings from a backup file | Import settings from a backup file | keep |  |
| `Controls.SettingsView.RestoreDefaultsButton.ToolTip` | Reset all settings to their original defaults | Reset all settings to their original defaults | keep |  |
| `AppBarExportButton.Label` | Export | Export | keep |  |
| `ExportCsvMenuItem.Text` | Export to CSV... | Export to CSV... | keep |  |
| `ExportJsonMenuItem.Text` | Export to JSON... | Export to JSON... | keep |  |
| `CopyToClipboardMenuItem.Text` | Copy to Clipboard | Copy to Clipboard | keep |  |
| `ExportSuccessTitle` | Export Successful | Export Successful | keep |  |
| `ExportCsvSuccessMessage` | Results exported to CSV file successfully. | Results exported to CSV file successfully. | keep |  |
| `ExportJsonSuccessMessage` | Results exported to JSON file successfully. | Results exported to JSON file successfully. | keep |  |
| `ExportErrorTitle` | Export Failed | Export Failed | keep |  |
| `ExportErrorMessage` | Failed to export results: {0} | Failed to export results: {0} | keep | uses {0} placeholder |
| `CopySuccessTitle` | Copied to Clipboard | Copied to Clipboard | keep |  |
| `CopyToClipboardSuccessMessage` | Results copied to clipboard successfully. | Results copied to clipboard successfully. | keep |  |
| `CopyErrorTitle` | Copy Failed | Copy Failed | keep |  |
| `CopyErrorMessage` | Failed to copy results: {0} | Failed to copy results: {0} | keep | uses {0} placeholder |

## Settings — Debug / Notifications / Localization Self-test

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `DebugHeaderTextBlock.Text` | Debug | Debug | keep |  |
| `DebugDescriptionTextBlock.Text` | Use button below to verify Windows toast notifications. If required Windows App SDK dependencies are missing you will receive troubleshooting guidance. | Use the button below to verify desktop notifications. If your notification daemon is unavailable you will receive troubleshooting guidance. | rename-key | new key `DebugDescriptionTextBlock.Text` retained; copy revised; Linux uses KNotifications with freedesktop fallback (`linux-decisions.md`, Notifications) |
| `TestNotificationButton.Content` | Test Notification | Test Notification | keep |  |
| `TestNotificationExplanationTextBlock.Text` | Use the button below to verify Windows toast notifications. If required Windows App SDK dependencies are missing you will receive troubleshooting guidance. | Use the button below to verify desktop notifications. If KNotifications and the freedesktop notification interface are both unavailable you will receive troubleshooting guidance. | rename-key | copy revised |
| `Controls.SettingsView.TestNotificationButton.ToolTip` | Send a sample notification to verify that Windows toasts are working. | Send a sample notification to verify that desktop notifications are working. | keep | copy revised |
| `NotificationTestTitle` | Test Notification | Test Notification | keep |  |
| `NotificationTestMessage` | This is a test notification to verify that notifications are working correctly. | This is a test notification to verify that notifications are working correctly. | keep |  |
| `NotificationTestSentTitle` | Test Notification Sent | Test Notification Sent | keep |  |
| `NotificationTestSentMessage` | Windows was asked to display a toast notification. If you still do not see it, verify that notifications are enabled for Grex (Windows Settings > System > Notifications) and that Focus assist / Do Not Disturb is disabled.\n\nNotification log: {0}\nDiagnostic log: {1} | A desktop notification was sent. If you still do not see it, verify that notifications are enabled for Grexa (System Settings → Notifications) and that Do Not Disturb is disabled.\n\nNotification log: {0}\nDiagnostic log: {1} | keep | uses {0} and {1} placeholders; copy revised |
| `NotificationTestErrorTitle` | Test Notification Error | Test Notification Error | keep |  |
| `NotificationTestErrorMessage` | An unexpected error occurred while trying to send the notification.\n\n{0}\n\nSee log: {1} | An unexpected error occurred while trying to send the notification.\n\n{0}\n\nSee log: {1} | keep | uses {0} and {1} placeholders |
| `NotificationSupportUnavailable` | Windows toast notifications are not available on this system configuration. | Desktop notifications are not available on this system configuration. | keep | copy revised |
| `NotificationSupportCommonCauses` | Common causes:\n• Windows App SDK runtime (Singleton package) is missing or needs repair\n• Application is running with elevated (administrator) privileges\n• For unpackaged apps, IsSupported() may return false incorrectly | Common causes:\n• Neither KNotifications nor the freedesktop notification daemon is running\n• Flatpak portal permissions for `notifications` are denied\n• A headless or kiosk session blocks notifications | remove-windows-only | remove the WinAppSDK content; the Linux causes are listed in the new replacement |
| `NotificationSupportTroubleshooting` | Troubleshooting steps:\n1. Install or repair the Windows App Runtime: https://aka.ms/windowsappsdk/runtime\n2. Ensure Grex is NOT running as administrator\n3. Restart Grex after installing/repairing the runtime | Troubleshooting steps:\n1. Verify a notification daemon (Plasma, GNOME, Mate, dunst, mako) is running\n2. For Flatpak builds, grant the `notifications` portal\n3. Restart Grexa | remove-windows-only | replace contents wholesale |
| `NotificationRegistrationFailure` | Grex could not register for Windows toast notifications. | Grexa could not register for desktop notifications. | keep | copy revised |
| `NotificationRegistrationCommonCauses` | Common causes:\n• Windows App SDK runtime (Singleton package) is missing or corrupted\n• Application is running with elevated (administrator) privileges\n• Application identity is not properly configured | Common causes:\n• KNotifications service is unreachable on the session bus\n• Flatpak portal denied the `notifications` permission\n• The application identity (.desktop file) is missing | remove-windows-only | replace contents wholesale |
| `NotificationRegistrationTroubleshooting` | Troubleshooting steps:\n1. Install or repair the Windows App Runtime: https://aka.ms/windowsappsdk/runtime\n2. Ensure Grex is NOT running as administrator\n3. Restart Grex after installing/repairing the runtime | Troubleshooting steps:\n1. Confirm `org.freedesktop.Notifications` is on the session bus\n2. Verify the Grexa `.desktop` file is installed\n3. For Flatpak, grant the `notifications` portal | remove-windows-only | replace contents wholesale |
| `NotificationDetailsPrefix` | Details: {0} | Details: {0} | keep | uses {0} placeholder |
| `NotificationLogFilePrefix` | Log file: {0} | Log file: {0} | keep | uses {0} placeholder |
| `TestLocalizationExplanationTextBlock.Text` | Use the button below to test language switching and verify all translation keys. | Use the button below to test language switching and verify all translation keys. | keep |  |
| `TestLocalizationButton.Content` | Test Localization | Test Localization | keep |  |
| `Controls.SettingsView.TestLocalizationButton.ToolTip` | Test language switching and verify all English translation keys exist in all supported languages | Test language switching and verify all English translation keys exist in all supported languages | keep |  |
| `LocalizationTestSuccessTitle` | Localization Test Passed | Localization Test Passed | keep |  |
| `LocalizationTestSuccessMessage` | All {0} localization keys are present in all {1} supported languages. | All {0} localization keys are present in all {1} supported languages. | keep | uses {0} {1} placeholders; **needs plural ICU MessageFormat** |
| `LocalizationTestFailureTitle` | Localization Test Failed | Localization Test Failed | keep |  |
| `LocalizationTestFailureSummary` | Some localization keys are missing in one or more languages: | Some localization keys are missing in one or more languages: | keep |  |
| `LocalizationTestLanguageFailure` | {0}: {1} missing keys | {0}: {1} missing keys | keep | uses {0} {1} placeholders; **needs plural ICU MessageFormat** |
| `LocalizationTestMoreKeys` | ... and {0} more | ... and {0} more | keep | uses {0} placeholder; **needs plural ICU MessageFormat** |
| `LocalizationTestErrorTitle` | Localization Test Error | Localization Test Error | keep |  |
| `LocalizationTestErrorMessage` | An error occurred while testing localization: {0} | An error occurred while testing localization: {0} | keep | uses {0} placeholder |

## Settings — AI Search Endpoint

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `AiSearchSettingsHeaderTextBlock.Text` | AI Search Endpoint | AI Search Endpoint | keep |  |
| `AiSearchSettingsDescriptionTextBlock.Text` | Configure an OpenAI-compatible API endpoint for future semantic search workflows. | Configure an OpenAI-compatible API endpoint for future semantic search workflows. | keep |  |
| `AiSearchEndpointLabelTextBlock.Text` | Endpoint URL: | Endpoint URL: | keep |  |
| `AiSearchEndpointTextBox.PlaceholderText` | e.g., https://api.openai.com/v1 | e.g., https://api.openai.com/v1 | keep |  |
| `AiSearchApiKeyLabelTextBlock.Text` | API Key (optional): | API Key (optional): | keep |  |
| `AiSearchApiKeyPasswordBox.PlaceholderText` | Leave blank if your endpoint does not require authentication | Leave blank if your endpoint does not require authentication | keep | on Linux the value is stored in KWallet / Secret Service, not plaintext (`linux-decisions.md`, Settings) |
| `Controls.SettingsView.AiSearchEndpointTextBox.ToolTip` | Base URL for an OpenAI-compatible API endpoint. | Base URL for an OpenAI-compatible API endpoint. | keep |  |
| `Controls.SettingsView.AiSearchApiKeyPasswordBox.ToolTip` | Optional API key used for authenticated OpenAI-compatible endpoints. | Optional API key used for authenticated OpenAI-compatible endpoints. The key is stored in your system keyring. | keep | clarify keyring storage |
| `AiSearchModelLabelTextBlock.Text` | Model (optional): | Model (optional): | keep |  |
| `AiSearchModelTextBox.PlaceholderText` | e.g., gpt-4o-mini | e.g., gpt-4o-mini | keep |  |
| `Controls.SettingsView.AiSearchModelTextBox.ToolTip` | Optional model id. Leave blank to auto-detect from the endpoint. | Optional model id. Leave blank to auto-detect from the endpoint. | keep |  |
| `TestAiEndpointButton.Content` | Test Endpoint | Test Endpoint | keep |  |
| `TestAiEndpointButtonTesting.Content` | Testing... | Testing... | keep |  |
| `Controls.SettingsView.TestAiEndpointButton.ToolTip` | Send a test request to your AI endpoint using the current settings | Send a test request to your AI endpoint using the current settings | keep |  |
| `AiEndpointTestSuccessTitle` | AI Endpoint Test Succeeded | AI Endpoint Test Succeeded | keep |  |
| `AiEndpointTestSuccessMessage` | Connected to {0}. First model: {1} | Connected to {0}. First model: {1} | keep | uses {0} {1} placeholders |
| `AiEndpointTestErrorTitle` | AI Endpoint Test Failed | AI Endpoint Test Failed | keep |  |
| `AiEndpointTestErrorMessage` | Could not connect to the AI endpoint. Details: {0} | Could not connect to the AI endpoint. Details: {0} | keep | uses {0} placeholder |
| `AiEndpointTestEndpointRequiredMessage` | Enter an AI endpoint URL before testing. | Enter an AI endpoint URL before testing. | keep |  |
| `AiEndpointTestUnknownModel` | (no model id returned) | (no model id returned) | keep |  |

## AI Search Chat (Search tab side panel)

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `AppBarAiButton.Label` | AI | AI | keep |  |
| `AiChatHeaderTextBlock.Text` | AI Search Chat | AI Search Chat | keep |  |
| `AiChatEmptyStateTextBlock.Text` | Click AI to start an AI-assisted search discussion. | Click AI to start an AI-assisted search discussion. | keep |  |
| `AiChatInputTextBox.PlaceholderText` | Ask follow-up questions... | Ask follow-up questions... | keep |  |
| `AiSendButton.Content` | Send | Send | keep |  |
| `AiSearchConfigurationTitle` | AI Search Configuration Required | AI Search Configuration Required | keep |  |
| `AiSearchEndpointRequiredMessage` | Set an OpenAI-compatible endpoint in Settings before starting an AI search. | Set an OpenAI-compatible endpoint in Settings before starting an AI search. | keep |  |
| `AiSearchInputRequiredTitle` | Search Path and Query Required | Search Path and Query Required | keep |  |
| `AiSearchInputRequiredMessage` | Enter both a search path and a search query before starting AI search. | Enter both a search path and a search query before starting AI search. | keep |  |
| `AiSearchRequestFailedMessage` | AI request failed: {0} | AI request failed: {0} | keep | uses {0} placeholder |
| `AiSearchRequestCancelledMessage` | AI request cancelled. | AI request cancelled. | keep |  |
| `AiChatSpeakerUser` | You | You | keep |  |
| `AiChatSpeakerAssistant` | AI | AI | keep |  |
| `Controls.SearchTabContent.AppBarAiButton.ToolTip` | Start an AI-assisted search discussion for this path and query | Start an AI-assisted search discussion for this path and query | keep |  |
| `Controls.SearchTabContent.AiChatInputTextBox.ToolTip` | Send a follow-up question to the AI search assistant | Send a follow-up question to the AI search assistant | keep |  |

## Regex Builder

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `RegexBuilderTab` | Regex Builder | Regex Builder | keep |  |
| `RegexBuilderNavItem.Content` | Regex Builder | Regex Builder | keep |  |
| `RegexPatternLabel` | Regular Expression Pattern | Regular Expression Pattern | keep |  |
| `RegexPatternTextBlock.Text` | Regex Pattern | Regex Pattern | keep |  |
| `RegexPatternTextBox.PlaceholderText` | Enter Regex pattern... | Enter Regex pattern... | keep |  |
| `Controls.RegexBuilderView.RegexPatternTextBox.ToolTip` | Enter the Regex pattern you want to evaluate. | Enter the Regex pattern you want to evaluate. | keep |  |
| `TestTextLabel` | Test Text | Test Text | keep |  |
| `SampleTextTextBlock.Text` | Sample Text | Sample Text | keep |  |
| `SampleTextTextBox.PlaceholderText` | Enter sample text to test your Regex pattern against... | Enter sample text to test your Regex pattern against... | keep |  |
| `Controls.RegexBuilderView.SampleTextTextBox.ToolTip` | Provide sample text that the Regex will be tested against. | Provide sample text that the Regex will be tested against. | keep |  |
| `MatchesLabel` | Matches | Matches | keep |  |
| `LiveMatchResultsTextBlock.Text` | Live Match Results | Live Match Results | keep |  |
| `VisualRegexBreakdownTextBlock.Text` | Visual Regex Breakdown | Visual Regex Breakdown | keep |  |
| `OptionsTextBlock.Text` | Options | Options | keep |  |
| `CaseInsensitiveCheckBox.Content` | Case insensitive | Case insensitive | keep |  |
| `MultilineCheckBox.Content` | Multiline | Multiline | keep |  |
| `GlobalMatchCheckBox.Content` | Global match | Global match | keep |  |
| `Controls.RegexBuilderView.CaseInsensitiveCheckBox.ToolTip` | Ignore character casing when evaluating matches. | Ignore character casing when evaluating matches. | keep |  |
| `Controls.RegexBuilderView.MultilineCheckBox.ToolTip` | Treat ^ and $ as the start and end of each line instead of the whole text. | Treat ^ and $ as the start and end of each line instead of the whole text. | keep |  |
| `Controls.RegexBuilderView.GlobalMatchCheckBox.ToolTip` | Find every match instead of stopping after the first one. | Find every match instead of stopping after the first one. | keep |  |
| `PresetsTextBlock.Text` | Presets: | Presets: | keep |  |
| `EmailPresetButton.Content` | Email | Email | keep |  |
| `PhonePresetButton.Content` | Phone | Phone | keep |  |
| `DatePresetButton.Content` | Date | Date | keep |  |
| `DigitsPresetButton.Content` | Digits | Digits | keep |  |
| `URLPresetButton.Content` | URL | URL | keep |  |
| `EnterValidPatternMessage` | Enter a valid Regex pattern to see matches. | Enter a valid Regex pattern to see matches. | keep |  |
| `EnterSampleTextMessage` | Enter sample text to test your Regex pattern. | Enter sample text to test your Regex pattern. | keep |  |
| `RegexBreakdownNoMatchesFound` | No matches found. | No matches found. | keep |  |
| `RegexBreakdownFoundMatches` | Found {0} match(es). | Found {0} match(es). | keep | uses {0} placeholder; **needs plural ICU MessageFormat** (replaces the awkward "match(es)" syntax) |
| `RegexBreakdownNoMatchFound` | No match found. | No match found. | keep | singular companion of above |
| `RegexBreakdownFoundOneMatch` | Found 1 match. | Found 1 match. | keep | hardcoded singular; folds into ICU plural |
| `RegexBreakdownErrorMessage` | Error: {0} | Error: {0} | keep | uses {0} placeholder |
| `RegexBreakdownEnterPatternMessage` | Enter a Regex pattern to see the breakdown. | Enter a Regex pattern to see the breakdown. | keep |  |
| `RegexBreakdownInvalidPatternMessage` | Invalid Regex pattern: {0} | Invalid Regex pattern: {0} | keep | uses {0} placeholder |
| `RegexBreakdownTypeCharacterClass` | Character Class | Character Class | keep |  |
| `RegexBreakdownTypeNonCapturingGroup` | Non-Capturing Group | Non-Capturing Group | keep |  |
| `RegexBreakdownTypeCapturingGroup` | Capturing Group | Capturing Group | keep |  |
| `RegexBreakdownTypeQuantifier` | Quantifier | Quantifier | keep |  |
| `RegexBreakdownTypeAnchor` | Anchor | Anchor | keep |  |
| `RegexBreakdownTypeEscapeSequence` | Escape Sequence | Escape Sequence | keep |  |
| `RegexBreakdownTypeLiteral` | Literal | Literal | keep |  |
| `RegexBreakdownDescCharacterClass` | Matches any character in the set | Matches any character in the set | keep |  |
| `RegexBreakdownDescNonCapturingGroup` | Groups without capturing | Groups without capturing | keep |  |
| `RegexBreakdownDescCapturingGroup` | Captures matched text | Captures matched text | keep |  |
| `RegexBreakdownDescQuantifierRange` | Quantifier: specifies exact count or range | Quantifier: specifies exact count or range | keep |  |
| `RegexBreakdownDescZeroOrMore` | Zero or more | Zero or more | keep |  |
| `RegexBreakdownDescOneOrMore` | One or more | One or more | keep |  |
| `RegexBreakdownDescZeroOrOne` | Zero or one | Zero or one | keep |  |
| `RegexBreakdownDescAnchorStart` | Start of line/string | Start of line/string | keep |  |
| `RegexBreakdownDescAnchorEnd` | End of line/string | End of line/string | keep |  |
| `RegexBreakdownDescDigit` | Digit (0-9) | Digit (0-9) | keep |  |
| `RegexBreakdownDescNonDigit` | Non-digit | Non-digit | keep |  |
| `RegexBreakdownDescWordChar` | Word character (a-z, A-Z, 0-9, _) | Word character (a-z, A-Z, 0-9, _) | keep |  |
| `RegexBreakdownDescNonWordChar` | Non-word character | Non-word character | keep |  |
| `RegexBreakdownDescWhitespace` | Whitespace | Whitespace | keep |  |
| `RegexBreakdownDescNonWhitespace` | Non-whitespace | Non-whitespace | keep |  |
| `RegexBreakdownDescNewline` | Newline | Newline | keep |  |
| `RegexBreakdownDescTab` | Tab | Tab | keep |  |
| `RegexBreakdownDescCarriageReturn` | Carriage return | Carriage return | keep |  |
| `RegexBreakdownDescLiteralChar` | Literal character | Literal character | keep |  |
| `RegexBreakdownOverwritePatternTitle` | Overwrite Regex Pattern? | Overwrite Regex Pattern? | keep |  |
| `RegexBreakdownOverwritePatternMessage` | Do you want to overwrite the current Regex Pattern with the {0} preset? | Do you want to overwrite the current Regex Pattern with the {0} preset? | keep | uses {0} placeholder |
| `ConfirmRegexOverwriteTitle` | Overwrite Regex Pattern? | Overwrite Regex Pattern? | keep | duplicate of above |
| `ConfirmRegexOverwriteMessage` | Do you want to overwrite the current Regex Pattern with the {0} preset? | Do you want to overwrite the current Regex Pattern with the {0} preset? | keep | duplicate; uses {0} placeholder |

## About

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `AboutNavItem.Content` | About | About | keep |  |
| `AboutCreatedByTextBlock.Text` | Created by VisorCraft | Created by VisorCraft | keep |  |
| `AboutLicenseTextBlock.Text` | Licensed under GPL 3.0 | Licensed under GPL 3.0 | keep |  |
| `AboutGitHubLinkButton.Content` | View Project on GitHub | View Project on GitHub | keep |  |
| `AboutKeyboardShortcutTextBlock.Text` | Press F1 anytime to open this page | Press F1 anytime to open this page | keep |  |
| `AboutVersionLabel.Text` | Version | Version | keep |  |

## Time Units

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `TimeSecondSingular` | second | second | keep | **fold singular/plural pair into ICU plural** |
| `TimeSecondPlural` | seconds | seconds | keep | **fold singular/plural pair into ICU plural** |
| `TimeMinuteSingular` | minute | minute | keep | **fold singular/plural pair into ICU plural** |
| `TimeMinutePlural` | minutes | minutes | keep | **fold singular/plural pair into ICU plural** |
| `TimeHourSingular` | hour | hour | keep | **fold singular/plural pair into ICU plural** |
| `TimeHourPlural` | hours | hours | keep | **fold singular/plural pair into ICU plural** |

## Dialogs — Generic Titles And Buttons

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `ErrorTitle` | Error | Error | keep |  |
| `WarningTitle` | Warning | Warning | keep |  |
| `InformationTitle` | Information | Information | keep |  |
| `OKButton` | OK | OK | keep |  |
| `CancelButton` | Cancel | Cancel | keep |  |
| `CancelButtonText` | Cancel | Cancel | keep | duplicate of `CancelButton`; kept for binding parity |
| `YesButton` | Yes | Yes | keep |  |
| `NoButton` | No | No | keep |  |
| `ProceedButton` | Proceed | Proceed | keep |  |
| `ProceedButtonText` | Proceed | Proceed | keep | duplicate of `ProceedButton`; kept for binding parity |
| `ContinueButton` | Continue | Continue | keep |  |
| `CloseButton` | Close | Close | keep |  |
| `SaveButton` | Save | Save | keep |  |
| `OverwriteButton` | Overwrite | Overwrite | keep |  |
| `DeleteButton` | Delete | Delete | keep |  |

## Errors And Confirmation Dialogs (non-Windows-specific)

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `ApplicationStartupErrorTitle` | Application Startup Error | Application Startup Error | keep |  |
| `ApplicationStartupErrorMessage` | Failed to start application: {0} | Failed to start application: {0} | keep | uses {0} placeholder |
| `UnhandledErrorTitle` | Unhandled Error | Unhandled Error | keep |  |
| `UnhandledErrorMessage` | An unexpected error occurred: {0} | An unexpected error occurred: {0} | keep | uses {0} placeholder |
| `TabCreationErrorTitle` | Tab Creation Error | Tab Creation Error | keep |  |
| `TabCreationErrorMessage` | Failed to create tab: {0} | Failed to create tab: {0} | keep | uses {0} placeholder |
| `TabCreationCriticalMessage` | A critical error occurred while creating a tab: {0} | A critical error occurred while creating a tab: {0} | keep | uses {0} placeholder |
| `SearchErrorTitle` | Search Error | Search Error | keep |  |
| `SearchErrorMessage` | An error occurred during search: {0} | An error occurred during search: {0} | keep | uses {0} placeholder |
| `NoResultsPossibleTitle` | No Results Possible | No Results Possible | keep |  |
| `NoResultsPossibleMessage` | The 'Exclude Dirs' value is excluding all directories. Please check your pattern and try again. | The 'Exclude Dirs' value is excluding all directories. Please check your pattern and try again. | keep |  |
| `InvalidRegexPatternTitle` | Invalid Regex Pattern | Invalid Regex Pattern | keep |  |
| `InvalidRegexPatternMessage` | The 'Exclude Dirs' value is not a valid regular expression. Please check your pattern and try again. | The 'Exclude Dirs' value is not a valid regular expression. Please check your pattern and try again. | keep |  |
| `ConfirmReplaceTitle` | Confirm Replace | Confirm Replace | keep |  |
| `ConfirmReplaceMessage` | Are you sure you want to perform the replace operation? This action cannot be undone. | Are you sure you want to perform the replace operation? This action cannot be undone. | keep |  |

## Windows-only Dialogs (being removed)

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `AdminWarningTitle` | Running as Administrator | (removed) | remove-windows-only | Linux uses `pkexec`/`sudo` and root warning is handled by polkit; if Grexa wants a "running as root" warning the new key is `RootWarningTitle` (listed in new-strings section) |
| `AdminWarningMessage` | Grex is currently running with administrator privileges.\n\nRunning as administrator can cause issues with:\n• Windows toast notifications\n• Some Windows App SDK features\n• Application security and isolation\n\nIt is recommended you run Grex as a regular user. | (removed) | remove-windows-only | replace with `RootWarningMessage` listed below |
| `AdminWarningIgnoreButton` | Ignore (Unsafe) | (removed) | remove-windows-only |  |
| `AdminWarningrexitButton` | Exit | (removed) | remove-windows-only | **typo'd key**: should have been `AdminWarningExitButton`. Translation files inherit the typo. Grexa drops the key entirely; the generic `CloseButton` covers the use case |
| `WslPathWarningTitle` | WSL Path Detected | (removed) | remove-windows-only | WSL does not exist on Linux (`linux-decisions.md`, File Systems And Paths; `grex-wsl-audit.md`) |
| `WslPathWarningMessage` | The path you entered appears to access WSL files through Windows (e.g., C:\home\...).\n\nChoose "WSL Search" for faster native WSL search, or "Continue" for slower Windows-based search. | (removed) | remove-windows-only |  |
| `WslSearchButton` | WSL Search | (removed) | remove-windows-only |  |
| `WslSearchErrorTitle` | WSL Search Error | (removed) | remove-windows-only |  |
| `WslSearchErrorMessage` | An error occurred while searching in WSL: {0} | (removed) | remove-windows-only | uses {0} placeholder |

## Tooltips (already covered inline above)

Tooltips are scoped to their host control and listed in the matching tab
section above. No tooltip is dropped except those that name Windows-specific
elements (`UseWindowsSearchCheckBox.ToolTipService.ToolTip`, the
`AdminWarning*` family, and the WSL family). See those rows in their
respective sections.

## Context Preview

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `ContextPreviewTitle` | Context Preview | Context Preview | keep |  |
| `ContextPreviewOpenInEditorButton` | Open in Editor | Open in Editor | keep | new explicit editor presets ship for Kate/KWrite, VS Code/VSCodium, JetBrains, Sublime, GNOME Text Editor, Neovim, and `xdg-open` (`linux-decisions.md`, Editor And File Manager Integration) |
| `ContextPreviewMenuItem` | Preview (Space) | Preview (Space) | keep |  |
| `ContextPreviewLoadingText` | Loading context... | Loading context... | keep |  |
| `ContextPreviewErrorTitle` | Preview Error | Preview Error | keep |  |
| `ContextPreviewErrorText` | Failed to load context preview: {0} | Failed to load context preview: {0} | keep | uses {0} placeholder |

## Main Window Chrome

| Key | English text | Linux equivalent text | Status | Notes |
| --- | --- | --- | --- | --- |
| `AppName` | Grex | Grexa | rename-key | the on-disk key stays `AppName`; only the value changes |
| `MainWindow.Title` | Grex - Tabbed File Search | Grexa - Tabbed File Search | keep | value text updates; key kept |

## NEW Linux-only Strings Grexa Needs

These keys do not exist in Grex's `Resources.resw`. They are required by
Linux-specific UI flows documented in `docs/linux-decisions.md` and the
related audits.

| Proposed key | English text | Area | Origin / Reason |
| --- | --- | --- | --- |
| `UseFileIndexCheckBox.Content` | Use file index (Baloo) | Search tab | rename of `UseWindowsSearchCheckBox.Content` |
| `UseFileIndexCheckBox.ToolTipService.ToolTip` | Use the Baloo file index to seed candidate paths for faster text searches on indexed locations. Regex searches and non-indexed locations always use the traditional scanner. | Search tab | rename of `UseWindowsSearchCheckBox.ToolTipService.ToolTip` |
| `FileIndexDisabledTooltip` | Works only for plain-text searches under indexed paths. | Search tab | rename of `WindowsSearchDisabledTooltip` |
| `FileIndexUnavailableMessage` | Baloo is not running or has not indexed this path. Falling back to the traditional scanner. | Search tab | new (Phase 13 spike) |
| `BalooReindexHint` | Tip: Baloo can be reconfigured under System Settings → Search. | Search tab | new |
| `ContainerSettingsHeaderTextBlock.Text` | Container Search | Settings | rename of Docker keys |
| `ContainerSettingsDescriptionTextBlock.Text` | Natively search Linux containers using the Docker or Podman CLI. If grep is unavailable inside the container, files are mirrored from the running container so Grexa can scan them without leaving the app. | Settings | rename |
| `EnableContainerSearchLabelTextBlock.Text` | Enable Container Search | Settings | rename |
| `EnableContainerSearchToggleSwitch.OnContent` | Enabled | Settings | rename |
| `EnableContainerSearchToggleSwitch.OffContent` | Disabled | Settings | rename |
| `Controls.SettingsView.EnableContainerSearchToggleSwitch.ToolTip` | Show available Docker and Podman containers next to the search path so you can target them directly. | Settings | rename |
| `ContainerRuntimeLabelTextBlock.Text` | Container Runtime: | Settings | new — selects Docker / rootless Podman / rootful Podman |
| `ContainerRuntimeAutoComboBoxItem.Content` | Auto-detect | Settings | new |
| `ContainerRuntimeDockerComboBoxItem.Content` | Docker | Settings | new |
| `ContainerRuntimePodmanRootlessComboBoxItem.Content` | Podman (rootless) | Settings | new |
| `ContainerRuntimePodmanRootfulComboBoxItem.Content` | Podman (rootful) | Settings | new |
| `ContainerTargetComboBox.Header` | Search Target | Search tab | rename |
| `ContainerTargetComboBox.PlaceholderText` | Select container | Search tab | rename |
| `ContainerRefreshButton.ToolTipService.ToolTip` | Refresh container list | Search tab | rename |
| `ContainerInitializationErrorMessage` | Could not initialize container integration: {0} | Errors | rename; uses {0} |
| `ContainerRefreshErrorMessage` | Unable to refresh containers: {0} | Errors | rename; uses {0} |
| `ContainerMirrorErrorMessage` | Failed to prepare the container search mirror: {0} | Errors | rename; uses {0} |
| `ContainerSearchErrorTitle` | Container Search Error | Errors | rename |
| `ContainerUnavailableMessage` | Container runtime is unavailable. Make sure the Docker or Podman socket is reachable and that your user has permission to run container commands. | Errors | rename |
| `ContainerNotSelectedMessage` | Select a container before starting the search. | Errors | rename |
| `PodmanSocketUnreachableMessage` | Podman socket is not reachable. Start `podman.socket` with `systemctl --user start podman.socket` or set `DOCKER_HOST`. | Errors | new — Linux Podman diagnostic |
| `DockerSocketUnreachableMessage` | Docker socket is not reachable. Verify `docker.service` is running and that you are in the `docker` group, or set `DOCKER_HOST`. | Errors | new — Linux Docker diagnostic |
| `ContainerRootlessPermissionMessage` | Rootless Podman cannot read this path because of user-namespace mapping. Try `podman unshare chown ...` or use rootful Podman for this search. | Errors | new — replaces the Windows-symlink message |
| `OpenInFileManagerMenuItem` | Show in file manager | Tooltips / Context menu | rename of `OpenInExplorerMenuItem`; uses `org.freedesktop.FileManager1.ShowItems` |
| `OpenInDolphinMenuItem` | Reveal in Dolphin | Tooltips / Context menu | new — KDE-specific accelerator |
| `OpenInNautilusMenuItem` | Reveal in Files | Tooltips / Context menu | new — GNOME-specific accelerator |
| `OpenInKateMenuItem` | Open in Kate | Tooltips / Context menu | new — explicit editor preset (`linux-decisions.md`, Editor And File Manager Integration) |
| `OpenInVsCodeMenuItem` | Open in VS Code | Tooltips / Context menu | new |
| `OpenInVscodiumMenuItem` | Open in VSCodium | Tooltips / Context menu | new |
| `OpenInNeovimMenuItem` | Open in Neovim | Tooltips / Context menu | new |
| `OpenInXdgMenuItem` | Open with default application | Tooltips / Context menu | new — `xdg-open` fallback |
| `RootWarningTitle` | Running as root | Dialogs | new — replaces `AdminWarningTitle`; only shown if `geteuid() == 0` and the user did not opt out |
| `RootWarningMessage` | Grexa is running as the root user.\n\nRunning as root can cause issues with:\n• Wayland/Xorg session ownership\n• Desktop portal access (file pickers, notifications)\n• File ownership of created mirror directories\n\nIt is recommended you run Grexa as a regular user. | Dialogs | new — replaces `AdminWarningMessage` |
| `RootWarningIgnoreButton` | Ignore (Unsafe) | Dialogs | new — replaces `AdminWarningIgnoreButton` |
| `RootWarningExitButton` | Exit | Dialogs | new — replaces the typo'd `AdminWarningrexitButton` |
| `GvfsMountRequiredTitle` | Remote Path Not Mounted | Dialogs | new — for `smb://`, `fish://`, `mtp://` (`linux-decisions.md`, File Systems And Paths) |
| `GvfsMountRequiredMessage` | The path "{0}" is a remote URL. Mount it via GVFS, KIO-FUSE, or a file manager bookmark before searching. | Dialogs | new; uses {0} placeholder |
| `KioFuseHintMessage` | This path is provided by KIO-FUSE. Some operations (rename, replace) may be slower or unavailable. | Errors | new |
| `WslPathImportedMessage` | The imported path "{0}" referenced WSL and is not available on Linux. | Dialogs | new — surfaces during Grex backup import (`linux-decisions.md`, Imports From Grex Backups); uses {0} placeholder |
| `DriveLetterImportedMessage` | The imported path "{0}" referenced a Windows drive letter. Pick a Linux replacement directory or skip this entry. | Dialogs | new — Grex backup import; uses {0} placeholder |
| `UncPathImportedMessage` | The imported path "{0}" is a UNC path. Mount the share via GVFS or `cifs-utils` before using it. | Dialogs | new — Grex backup import; uses {0} placeholder |
| `SecretStoreUnavailableTitle` | Keyring Unavailable | Dialogs | new — if KWallet and Secret Service both fail |
| `SecretStoreUnavailableMessage` | KWallet and the freedesktop Secret Service are both unavailable. The AI API key cannot be stored securely. Provide the key per session instead. | Dialogs | new |
| `FlatpakPortalDeniedTitle` | Portal Permission Denied | Dialogs | new — Flatpak file-chooser denied |
| `FlatpakPortalDeniedMessage` | The Flatpak portal denied access to "{0}". Grant access with Flatseal or `flatpak override`. | Dialogs | new; uses {0} placeholder |
| `SystemPathScanWarningTitle` | Scanning Pseudo-filesystem | Dialogs | new — `/proc`, `/sys`, `/dev`, `/run` (`linux-decisions.md`, System path auto-exclusions) |
| `SystemPathScanWarningMessage` | The path "{0}" is a kernel pseudo-filesystem. Scanning it can hang or return non-files. Proceed anyway? | Dialogs | new; uses {0} placeholder |
| `CliExitCodeHintReady` | Exit codes: 0 = matches, 1 = no matches, 2 = error. | About / CLI help | new — documents the preserved exit code semantics (`linux-decisions.md`, CLI) |

## Windows-only Strings Being Dropped (summary)

Reason categories:

- **W/N (notification)** — Windows toast / WinAppSDK content. Linux uses
  KNotifications and freedesktop notifications.
- **W/A (admin)** — "Running as administrator" content. Linux equivalent is
  the new `RootWarning*` family.
- **W/W (WSL)** — WSL paths and dialogs. Grexa never accesses WSL.
- **W/S (Windows Search)** — Windows Search index references. Linux uses
  Baloo.
- **W/F (Windows file manager)** — "Show in Explorer" / shell verbs. Linux
  uses `org.freedesktop.FileManager1.ShowItems` and `xdg-open`.
- **W/L (Windows symlink privilege)** — the Docker symlink message that
  references Windows symbolic-link privileges.

| Key | Reason |
| --- | --- |
| `AdminWarningTitle` | W/A |
| `AdminWarningMessage` | W/A |
| `AdminWarningIgnoreButton` | W/A |
| `AdminWarningrexitButton` | W/A and typo'd key |
| `WslSearchErrorTitle` | W/W |
| `WslSearchErrorMessage` | W/W |
| `WslPathWarningTitle` | W/W |
| `WslPathWarningMessage` | W/W |
| `WslSearchButton` | W/W |
| `NotificationSupportCommonCauses` | W/N (content replaced) |
| `NotificationSupportTroubleshooting` | W/N (content replaced) |
| `NotificationRegistrationCommonCauses` | W/N (content replaced) |
| `NotificationRegistrationTroubleshooting` | W/N (content replaced) |
| `DockerSymlinkErrorMessage` | W/L |

Note: rename-key entries (Docker → Container, UseWindowsSearch → UseFileIndex,
OpenInExplorer → OpenInFileManager, theme-tooltip Windows wording, etc.) are
not double-listed here; see the per-area tables.

## Count Totals Per Status Bucket

Canonical en-US.resw key count: **473**.

| Status | Count |
| --- | --- |
| `keep` | 405 |
| `rename-key` | 40 |
| `remove-windows-only` | 14 |
| `add-linux-only` | 50 (new keys, not part of the 473 base) |
| TOTAL covered | 459 of 473 in keep/rename/remove + 14 in the dropped table = 473 |

Cross-check: the `keep + rename-key + remove-windows-only` columns of the
per-area tables sum to 473. The `add-linux-only` set is new content and is
intentionally excluded from the canonical count.

Per-bucket cross-check by area (approximate):

| Area | keep | rename-key | remove-windows-only |
| --- | --- | --- | --- |
| Search Tab | 105 | 5 | 0 |
| Results Columns / Header Menu | 28 | 1 | 0 |
| Settings — General / UI Language / Theme | 35 | 5 | 0 |
| Settings — Defaults | 17 | 2 | 0 |
| Settings — String Comparison | 21 | 1 | 0 |
| Settings — Context Preview | 3 | 0 | 0 |
| Settings — Docker → Containers | 3 | 15 | 1 |
| Settings — Backup / Export | 33 | 0 | 0 |
| Settings — Debug / Notifications / Localization | 18 | 4 | 4 |
| Settings — AI Search Endpoint | 14 | 0 | 0 |
| AI Search Chat | 15 | 0 | 0 |
| Regex Builder | 62 | 0 | 0 |
| About | 6 | 0 | 0 |
| Time Units | 6 | 0 | 0 |
| Dialogs — Generic Titles And Buttons | 15 | 0 | 0 |
| Errors / Confirmation Dialogs | 15 | 0 | 0 |
| Windows-only Dialogs | 0 | 0 | 9 |
| Context Preview | 6 | 0 | 0 |
| Main Window Chrome | 3 | 0 | 0 |

(The per-area totals are approximate because four keys are listed in two
sections — `SystemThemeRadio.Content` and the `SystemThemeComboBoxItem.Content`
pair, `SearchResultsTextBlock.Text` vs `SearchResultsLabelTextBlock.Text`, the
`SearchTypeTextBlock.Text` vs `SearchTypeLabelTextBlock.Text` pair, and the
two `StringComparisonMode*` rows — each pair is a Grex duplicate kept for XAML
binding parity and counted once.)

## Localization Recommendation For Counts And Plurals

Grex relies on either two-key singular/plural pairs (the `TimeSecond*`,
`TimeMinute*`, `TimeHour*` family) or English-only "X match(es)" hacks
(`RegexBreakdownFoundMatches`). Both are inadequate for the 107 locale set
that Grex carries:

- Slavic languages (`ru-RU`, `uk-UA`, `pl-PL`, `cs-CZ`, `sk-SK`, `sl-SI`,
  `bg-BG`, `hr-HR`, `bs-BA`, `mk-MK`, `sr-Latn-RS`, `be-BY`) need at least
  three plural forms (one, few, many).
- Arabic (`ar-SA`) needs six.
- Welsh (`cy-GB`) needs five.
- East Asian languages (`zh-CN`, `zh-TW`, `ja-JP`, `ko-KR`, `vi-VN`, `th-TH`,
  `lo-LA`, `km-KH`, `my-MM`, `bo-CN`) collapse to one form, which the current
  `Singular`/`Plural` pair handles by accident but with wasted strings.

Grexa decision:

1. Replace every singular/plural pair and every `{0} match(es)`-style string
   with **ICU MessageFormat** plural rules in a single key. Example:
   ```
   FoundMatchesStatus = "Found {matches, plural,
     one {# match}
     other {# matches}
   } in {files, plural,
     one {# file}
     other {# files}
   } in {duration}"
   ```
2. Pick a Rust ICU implementation (`icu_messageformat` from `icu4x`) so the
   GUI side calls a single `format_with()` and never branches on `n == 1`.
3. Translation files keep the `.resw` key set during the import pass and gain
   ICU markup on first edit. Importer leaves untranslated strings as the
   English fallback rather than emitting broken plural skeletons.

Keys flagged "**needs plural ICU MessageFormat**" in the tables above:

- `FoundMatchesStatus`
- `FilteredMatchesStatus`
- `ReplacedMatchesStatus`
- `LocalizationTestSuccessMessage`
- `LocalizationTestLanguageFailure`
- `LocalizationTestMoreKeys`
- `RegexBreakdownFoundMatches` (folds in `RegexBreakdownFoundOneMatch`,
  `RegexBreakdownNoMatchFound`, `RegexBreakdownNoMatchesFound`)
- `TimeSecondSingular` + `TimeSecondPlural` (fold into a single key)
- `TimeMinuteSingular` + `TimeMinutePlural` (fold into a single key)
- `TimeHourSingular` + `TimeHourPlural` (fold into a single key)

Plural folding does not change the count totals in the per-area tables; the
old keys remain marked `keep` so the importer can still read them out of an
imported Grex backup, but the renderer is wired against the new
`*PluralFormat` keys.

## Out Of Scope For This Document

- Per-locale audit. The 107 locale directories are assumed to follow the
  canonical key set. Any locale-specific issues (text expansion, RTL layout,
  font fallback) belong in `docs/grex-culture-comparison-audit.md`.
- Translation memory tooling and the import tool itself. Those are tracked
  in the PLAN phase 10 (Grex-to-Grexa importer) and PLAN phase 11
  (translation workflow).
- The CLI strings table. Grex CLI strings live in code, not in
  `Resources.resw`. Those are captured in
  `docs/grex-search-tab-content-codebehind-audit.md` and not re-listed here.
