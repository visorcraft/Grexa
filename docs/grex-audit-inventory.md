# Grex Audit Inventory

This inventory records the Grex files used as the behavioral source of truth for
Grexa. It was captured from `/work/repos/visorcraft/grex` with:

```bash
rg --files -g '!bin/**' -g '!obj/**' -g '!backup/**' -g '!__pycache__/**' | sort
```

Generated build outputs, local backup folders, and Python cache folders are not
part of this audit. The captured baseline contains 275 files. At capture time,
Grex was on `master` at `v1.1` / `da7cf68` with these modified files:

- `Controls/SearchTabContent.xaml` (modified)
- `Controls/SearchTabContent.xaml.cs` (modified)
- `UITests/AiSearchUiWiringTests.cs` (modified)

Those modified files are included in the inventory because they affect the AI
chat UI baseline.

## Repository And Build Files

- `AGENTS.md`
- `GEMINI.md`
- `LICENSE`
- `README.md`
- `grex.sln`
- `Grex.csproj`
- `Grex.Cli/Grex.Cli.csproj`
- `IntegrationTests/Grex.IntegrationTests.csproj`
- `Tests/Grex.Tests.csproj`
- `Tests/Grex.Cli.Tests/Grex.Cli.Tests.csproj`
- `UITests/Grex.UITests.csproj`
- `Package.appxmanifest`
- `app.manifest`
- `Properties/AssemblyInfo.cs`
- `IntegrationTests/Properties/AssemblyInfo.cs`
- `Tests/Properties/AssemblyInfo.cs`
- `Tests/Grex.Cli.Tests/Properties/AssemblyInfo.cs`
- `UITests/Properties/AssemblyInfo.cs`

## Application Shell

- `App.xaml`
- `App.xaml.cs`
- `MainWindow.xaml`
- `MainWindow.xaml.cs`

## Controls

- `Controls/AboutView.xaml`
- `Controls/AboutView.xaml.cs`
- `Controls/ContextPreviewDialog.xaml`
- `Controls/ContextPreviewDialog.xaml.cs`
- `Controls/RegexBuilderView.xaml`
- `Controls/RegexBuilderView.xaml.cs`
- `Controls/ResultsTemplateSelector.cs`
- `Controls/SearchTabContent.xaml`
- `Controls/SearchTabContent.xaml.cs`
- `Controls/SettingsView.xaml`
- `Controls/SettingsView.xaml.cs`

## Converters

- `Converters/BooleanToVisibilityConverter.cs`
- `Converters/SearchResultTooltipLinePrefixConverter.cs`

## Models

- `Models/AiChatMessage.cs`
- `Models/ContextPreviewResult.cs`
- `Models/DockerContainerInfo.cs`
- `Models/DockerContainerOption.cs`
- `Models/DockerMirrorInfo.cs`
- `Models/FileSearchResult.cs`
- `Models/PathSuggestion.cs`
- `Models/RecentSearch.cs`
- `Models/SearchProfile.cs`
- `Models/SearchResult.cs`
- `Models/SearchResultSortField.cs`
- `Models/SizeLimitType.cs`
- `Models/SizeUnit.cs`
- `Models/StringComparisonMode.cs`
- `Models/UnicodeNormalizationMode.cs`

## ViewModels

- `ViewModels/MainViewModel.cs`
- `ViewModels/TabViewModel.cs`

## Services

- `Services/AdminHelper.cs`
- `Services/AiSearchService.cs`
- `Services/ContextMenuService.cs`
- `Services/ContextPreviewService.cs`
- `Services/DockerSearchService.cs`
- `Services/EncodingDetectionService.cs`
- `Services/ExportService.cs`
- `Services/GitIgnoreService.cs`
- `Services/IEncodingDetectionService.cs`
- `Services/ILocalizationService.cs`
- `Services/ISearchService.cs`
- `Services/LocalizationService.cs`
- `Services/LocalizedToolTipRegistry.cs`
- `Services/NotificationService.cs`
- `Services/RecentPathsService.cs`
- `Services/RecentSearchesService.cs`
- `Services/SearchProfilesService.cs`
- `Services/SearchService.cs`
- `Services/SettingsService.cs`
- `Services/WindowsSearchIntegration.cs`
- `Services/WindowsSubsystemLinuxService.cs`

## CLI

- `Grex.Cli/CliSearchRunner.cs`
- `Grex.Cli/Commands/SearchCommand.cs`
- `Grex.Cli/Formatters/CsvOutputFormatter.cs`
- `Grex.Cli/Formatters/IOutputFormatter.cs`
- `Grex.Cli/Formatters/JsonOutputFormatter.cs`
- `Grex.Cli/Formatters/TextOutputFormatter.cs`
- `Grex.Cli/Options/SearchOptions.cs`
- `Grex.Cli/Program.cs`

## Scripts

- `Scripts/add_localization_entry.py`
- `Scripts/generate_translation_status.py`
- `Scripts/remove_localization_entry.py`
- `Scripts/test_add_localization_entry.py`
- `Scripts/test_remove_localization_entry.py`
- `Scripts/translate_remaining_entries.py`
- `Scripts/update_version.py`

## Assets

- `Assets/Grex.ico`
- `Assets/Grex.png`
- `Assets/Social1024x512.png`
- `Assets/SplashScreen.png`
- `Assets/Square1024x1024.png`
- `Assets/Square128x128Logo.png`
- `Assets/Square150x150Logo.png`
- `Assets/Square16x16Logo.png`
- `Assets/Square192x192Logo.png`
- `Assets/Square24x24Logo.png`
- `Assets/Square256x256Logo.png`
- `Assets/Square310x150Logo.png`
- `Assets/Square32x32Logo.png`
- `Assets/Square40x40Logo.png`
- `Assets/Square44x44Logo.png`
- `Assets/Square48x48Logo.png`
- `Assets/Square50x50Logo.png`
- `Assets/Square512x512Logo.png`
- `Assets/Square64x64Logo.png`
- `Assets/Square96x96Logo.png`

## Localization Resources

- `Strings/af-ZA/Resources.resw`
- `Strings/am-ET/Resources.resw`
- `Strings/ar-SA/Resources.resw`
- `Strings/as-IN/Resources.resw`
- `Strings/az-AZ/Resources.resw`
- `Strings/be-BY/Resources.resw`
- `Strings/bg-BG/Resources.resw`
- `Strings/bn-BD/Resources.resw`
- `Strings/bo-CN/Resources.resw`
- `Strings/bs-BA/Resources.resw`
- `Strings/ca-ES/Resources.resw`
- `Strings/ceb-PH/Resources.resw`
- `Strings/cs-CZ/Resources.resw`
- `Strings/cy-GB/Resources.resw`
- `Strings/da-DK/Resources.resw`
- `Strings/de-DE/Resources.resw`
- `Strings/el-GR/Resources.resw`
- `Strings/en-US/Resources.resw`
- `Strings/es-ES/Resources.resw`
- `Strings/et-EE/Resources.resw`
- `Strings/eu-ES/Resources.resw`
- `Strings/fa-IR/Resources.resw`
- `Strings/fi-FI/Resources.resw`
- `Strings/fil-PH/Resources.resw`
- `Strings/fj-FJ/Resources.resw`
- `Strings/fr-FR/Resources.resw`
- `Strings/ga-IE/Resources.resw`
- `Strings/gl-ES/Resources.resw`
- `Strings/gu-IN/Resources.resw`
- `Strings/ha-NG/Resources.resw`
- `Strings/haw-US/Resources.resw`
- `Strings/he-IL/Resources.resw`
- `Strings/hi-IN/Resources.resw`
- `Strings/hr-HR/Resources.resw`
- `Strings/hu-HU/Resources.resw`
- `Strings/hy-AM/Resources.resw`
- `Strings/id-ID/Resources.resw`
- `Strings/ig-NG/Resources.resw`
- `Strings/is-IS/Resources.resw`
- `Strings/it-IT/Resources.resw`
- `Strings/ja-JP/Resources.resw`
- `Strings/jv-Latn-ID/Resources.resw`
- `Strings/ka-GE/Resources.resw`
- `Strings/kk-KZ/Resources.resw`
- `Strings/km-KH/Resources.resw`
- `Strings/kn-IN/Resources.resw`
- `Strings/ko-KR/Resources.resw`
- `Strings/ky-KG/Resources.resw`
- `Strings/lb-LU/Resources.resw`
- `Strings/lo-LA/Resources.resw`
- `Strings/lt-LT/Resources.resw`
- `Strings/lv-LV/Resources.resw`
- `Strings/mg-MG/Resources.resw`
- `Strings/mi-NZ/Resources.resw`
- `Strings/mk-MK/Resources.resw`
- `Strings/ml-IN/Resources.resw`
- `Strings/mn-MN/Resources.resw`
- `Strings/mr-IN/Resources.resw`
- `Strings/ms-MY/Resources.resw`
- `Strings/mt-MT/Resources.resw`
- `Strings/my-MM/Resources.resw`
- `Strings/ne-NP/Resources.resw`
- `Strings/nl-NL/Resources.resw`
- `Strings/no-NO/Resources.resw`
- `Strings/nr-Latn-ZA/Resources.resw`
- `Strings/nso-Latn-ZA/Resources.resw`
- `Strings/or-IN/Resources.resw`
- `Strings/pa-IN/Resources.resw`
- `Strings/pl-PL/Resources.resw`
- `Strings/pt-BR/Resources.resw`
- `Strings/pt-PT/Resources.resw`
- `Strings/ro-RO/Resources.resw`
- `Strings/ru-RU/Resources.resw`
- `Strings/rw-RW/Resources.resw`
- `Strings/si-LK/Resources.resw`
- `Strings/sk-SK/Resources.resw`
- `Strings/sl-SI/Resources.resw`
- `Strings/sm-WS/Resources.resw`
- `Strings/sn-Latn-ZW/Resources.resw`
- `Strings/so-SO/Resources.resw`
- `Strings/sq-AL/Resources.resw`
- `Strings/sr-Latn-RS/Resources.resw`
- `Strings/ss-Latn-ZA/Resources.resw`
- `Strings/st-Latn-ZA/Resources.resw`
- `Strings/su-Latn-ID/Resources.resw`
- `Strings/sv-SE/Resources.resw`
- `Strings/sw-KE/Resources.resw`
- `Strings/ta-IN/Resources.resw`
- `Strings/te-IN/Resources.resw`
- `Strings/tg-TJ/Resources.resw`
- `Strings/th-TH/Resources.resw`
- `Strings/tk-TM/Resources.resw`
- `Strings/tn-Latn-ZA/Resources.resw`
- `Strings/to-TO/Resources.resw`
- `Strings/tr-TR/Resources.resw`
- `Strings/ts-Latn-ZA/Resources.resw`
- `Strings/ty-Latn-PF/Resources.resw`
- `Strings/ug-CN/Resources.resw`
- `Strings/uk-UA/Resources.resw`
- `Strings/ur-PK/Resources.resw`
- `Strings/uz-UZ/Resources.resw`
- `Strings/ve-Latn-ZA/Resources.resw`
- `Strings/vi-VN/Resources.resw`
- `Strings/xh-ZA/Resources.resw`
- `Strings/yo-NG/Resources.resw`
- `Strings/zh-CN/Resources.resw`
- `Strings/zh-TW/Resources.resw`
- `Strings/zu-ZA/Resources.resw`

## Unit Tests

- `Tests/ContextMenuServiceTests.cs`
- `Tests/Controls/AboutViewLocalizationTests.cs`
- `Tests/Controls/ExcludeDirsValidationTests.cs`
- `Tests/Controls/SettingsViewAiEndpointHelpersTests.cs`
- `Tests/Grex.Cli.Tests/CliSearchRunnerTests.cs`
- `Tests/Grex.Cli.Tests/OutputFormatterTests.cs`
- `Tests/SearchTabContentRightClickTests.cs`
- `Tests/Services/AdminHelperTests.cs`
- `Tests/Services/AiSearchServiceTests.cs`
- `Tests/Services/ContextPreviewServiceTests.cs`
- `Tests/Services/DockerSearchServiceTests.cs`
- `Tests/Services/EncodingDetectionServiceTests.cs`
- `Tests/Services/ExportServiceTests.cs`
- `Tests/Services/GitIgnoreServiceTests.cs`
- `Tests/Services/LocalizationServiceTestCollection.cs`
- `Tests/Services/LocalizationServiceTests.cs`
- `Tests/Services/NotificationServiceTests.cs`
- `Tests/Services/RecentPathsServiceTests.cs`
- `Tests/Services/RecentSearchesServiceTests.cs`
- `Tests/Services/RegexBuilderLanguageIntegrationTests.cs`
- `Tests/Services/RegexBuilderLanguageSwitchingTests.cs`
- `Tests/Services/RegexBuilderLocalizationKeysTests.cs`
- `Tests/Services/SearchProfilesServiceTests.cs`
- `Tests/Services/SearchServiceTests.cs`
- `Tests/Services/SettingsServiceTests.cs`
- `Tests/Services/SimpleTest.cs`
- `Tests/Services/WindowsSubsystemLinuxServiceTests.cs`
- `Tests/StandaloneTest.cs`
- `Tests/TestDataHelper.cs`
- `Tests/TestSettingsFixture.cs`
- `Tests/ViewModels/MainViewModelTests.cs`
- `Tests/ViewModels/TabViewModelTests.cs`

## Integration Tests

- `IntegrationTests/AboutPageTests.cs`
- `IntegrationTests/AiSearchLocalizationIntegrationTests.cs`
- `IntegrationTests/AiSearchSettingsIntegrationTests.cs`
- `IntegrationTests/RightClickContextMenuTests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `IntegrationTests/TestSettingsFixture.cs`

## UI Tests

- `UITests/AboutUITests.cs`
- `UITests/AiSearchUiWiringTests.cs`
- `UITests/SearchUITests.cs`
- `UITests/TestSettingsFixture.cs`
- `UITests/UITestMethodAttribute.cs`

## Documentation

- `docs/_config.yml`
- `docs/_data/navigation.yml`
- `docs/_includes/footer.html`
- `docs/architecture.md`
- `docs/build-and-test.md`
- `docs/features.md`
- `docs/index.md`
- `docs/reference.md`
- `docs/regex-localization.md`
- `docs/translations.md`
- `docs/usage.md`
- `docs/assets/img/logo.png`
- `docs/assets/img/screenshot_1.png`
- `docs/assets/img/screenshot_2.png`
- `docs/assets/img/screenshot_3.png`

## Immediate Audit Notes

- Search service: `Services/SearchService.cs` is the primary source for local/WSL search
  semantics, comparison behavior, file filtering, Windows Search seeding, and
  replace behavior.
- Docker service: `Services/DockerSearchService.cs` is the primary source for Docker direct
  grep, mirror fallback, container path preservation, and container-specific
  filtering behavior.
- Search tab controls: `Controls/SearchTabContent.xaml` and `Controls/SearchTabContent.xaml.cs`
  define the Search tab workflow surface, including active dirty changes for AI
  chat layout and filter pane collapse behavior.
- Settings controls: `Controls/SettingsView.xaml` and `Controls/SettingsView.xaml.cs` define
  settings persistence/editing workflows, including AI endpoint testing.
- Source locale: `Strings/en-US/Resources.resw` is the source locale for migration; all other
  `Strings/*/Resources.resw` files require placeholder/pluralization validation
  before conversion.
