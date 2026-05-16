# Grex About View Audit

This document records Grex `Controls/AboutView.xaml`,
`Controls/AboutView.xaml.cs`, and About-page integration/localization behavior
that Grexa must preserve, replace, or deliberately improve.

Source evidence:

- `Controls/AboutView.xaml`
- `Controls/AboutView.xaml.cs`
- `MainWindow.xaml`
- `MainWindow.xaml.cs`
- `Properties/AssemblyInfo.cs`
- `Grex.csproj`
- `Tests/Controls/AboutViewLocalizationTests.cs`
- `IntegrationTests/AboutPageTests.cs`
- `UITests/AboutUITests.cs`
- `Strings/en-US/Resources.resw`
- `Assets/`

## Role And Navigation

About is a footer navigation item in `MainWindow`.

MainWindow wiring:

- `AboutNavItem` is inside `NavigationView.FooterMenuItems`
- item tag is `About`
- `AboutContentGrid` contains `AboutView`
- selecting About hides Search, Regex Builder, and Settings grids
- selecting About also hides `StatusInfoBar`
- `RootGrid_KeyDown` handles F1 and calls `NavigateToAbout`
- `NavigateToAbout` selects `AboutNavItem`

Grexa replacement:

- Keep About reachable from persistent navigation.
- Preserve F1 as the About shortcut if it does not conflict with KDE platform
  conventions.
- Hide search-status UI while About is active.

## Layout And Content

`AboutView` is a WinUI `UserControl` named `AboutControl`.

Root layout:

- padded grid
- vertical `ScrollViewer`
- centered `StackPanel`
- spacing 24
- max width 600

Visible content in order:

1. `AppLogoImage`
2. `AppNameTextBlock`
3. `VersionTextBlock`
4. `CreatedByTextBlock`
5. `LicenseTextBlock`
6. `GitHubLinkButton`
7. `KeyboardShortcutTextBlock`

Current values:

- app name is hard-coded to `Grex`
- logo is loaded from `Assets/Grex.png`
- created by text is localized, English value `Created by VisorCraft`
- license text is localized, English value `Licensed under GPL 3.0`
- GitHub button content is localized, English value `View Project on GitHub`
- GitHub URL is `https://github.com/visorcraft/Grex`
- keyboard shortcut text is localized, English value
  `Press F1 anytime to open this page`

Grexa replacement:

- Change app name to `Grexa`.
- Use Grexa icon/logo assets.
- Use the Visorcraft Grexa repository URL:
  `https://github.com/visorcraft/grexa`.
- Preserve VisorCraft attribution and GPL 3.0 license text unless project
  licensing changes.
- Consider adding build metadata, copyright, and third-party notices before
  release.

## Logo Loading

`LoadAppLogo`:

- builds `Path.Combine(AppContext.BaseDirectory, "Assets", "Grex.png")`
- checks `File.Exists`
- sets `AppLogoImage.Source` to a `BitmapImage(new Uri(logoPath))`
- logs failures to debug output only

`Grex.csproj` copies `Assets/*.png` and `Assets/*.ico` into the app output for
the main WinUI application.

Grexa replacement:

- Load the icon through Qt resources, AppStream metadata, or an installed asset
  path.
- Ensure package formats install the same icon used by the About page, desktop
  file, and AppStream metadata.

## Version Display

`LoadVersionInfo`:

- gets `AboutVersionLabel.Text` from `LocalizationService`
- reads `Assembly.GetExecutingAssembly().GetName().Version`
- displays only major and minor as `<label> <major>.<minor>`
- catches errors and falls back to `Version 1.1`

Current assembly metadata:

- `AssemblyVersion("1.1.0.0")`
- `AssemblyFileVersion("1.1.0.0")`
- `AssemblyInformationalVersion("1.1")`

Current behavior quirk:

- `LoadVersionInfo` runs in the constructor.
- `RefreshLocalization` does not update `VersionTextBlock`.
- Changing language at runtime refreshes created-by, license, GitHub, and F1
  text, but not the version label.

Grexa replacement:

- Source version from Cargo package metadata or build-time generated constants.
- Display semantic version consistently with package/AppStream metadata.
- Refresh the version label when language changes.

## Localization

About localization keys:

- `AboutNavItem.Content`
- `AboutCreatedByTextBlock.Text`
- `AboutLicenseTextBlock.Text`
- `AboutGitHubLinkButton.Content`
- `AboutKeyboardShortcutTextBlock.Text`
- `AboutVersionLabel.Text`

`AboutView.RefreshLocalization` updates:

- created-by text
- license text
- GitHub button content
- keyboard shortcut text

`MainWindow.RefreshLocalization` updates:

- window title
- Search navigation item
- Regex Builder navigation item
- Settings navigation item
- About navigation item
- `StatusInfoBar.Title`
- child views, including `AboutView.RefreshLocalization`

Grexa replacement:

- Localize all visible About page text.
- Include the version label in runtime refresh.
- Prefer declarative QML translation bindings rather than manual per-control
  refresh code.
- Keep tests that ensure About keys exist across translation catalogs.

## Theme Handling

About subscribes to `MainWindow.ThemeChanged` on load and unsubscribes on
unload.

Custom theme behavior:

- non-custom themes clear foreground/background overrides
- custom themes set text foreground and control foregrounds through visual-tree
  traversal
- background is set to the theme background brush

High-contrast/custom themes recognized by About:

- Black Knight
- Paranoid
- Diamond
- Subspace
- Red Velvet
- Dreams
- Tiefling
- Vibes

Current discrepancy:

- About does not include `GentleGecko` in `IsHighContrastTheme`, while Settings
  and Regex Builder do.

Grexa replacement:

- Centralize theme classification.
- Use Qt/Kirigami palette roles instead of visual-tree brush mutation.
- Ensure About renders correctly in system, light, dark, and high-contrast
  themes.

## Pointer Cursor

`GitHubLinkButton` has pointer-enter and pointer-exit handlers.

Behavior:

- reflection sets WinUI `UIElement.ProtectedCursor`
- hover cursor is hand
- exit cursor is arrow
- reflection failures are ignored

Grexa replacement:

- Use native Qt cursor handling or default hyperlink behavior.
- Drop WinUI reflection.

## Tests

`Tests/Controls/AboutViewLocalizationTests.cs` covers:

- English resource keys exist
- created-by text contains `VisorCraft`
- license text contains `GPL`
- GitHub button content contains `GitHub`
- keyboard shortcut text contains `F1`
- version label key exists and contains `Version`

`IntegrationTests/AboutPageTests.cs` covers:

- expected About keys exist in every language resource file
- English resource values exactly match current expected strings
- localization service returns nonempty values for About keys
- About XAML and code-behind files exist

`UITests/AboutUITests.cs` documents:

- `MainViewModel` initializes with tabs
- expected localization keys are nonempty
- expected GitHub URL is valid HTTPS GitHub URL
- F1 is the expected virtual key
- About should live in footer menu
- About page should have seven main elements

Current test limitations:

- UI tests mostly document expected behavior; they do not assert actual XAML
  structure through a UI automation session.
- No test catches that `VersionTextBlock` is not refreshed on language change.
- No test catches the missing `GentleGecko` custom theme handling in About.

Grexa replacement:

- Add tests for About content, repository URL, version formatting, translation
  keys, F1 navigation, and theme rendering.
- Add a test that runtime language refresh updates the version label.

## Current Grexa Gaps

Grexa does not yet have an About page.

Required work:

- add an About page to the native shell/navigation
- add F1 navigation if appropriate
- add Grexa branding and logo assets
- show package/build version
- link to `https://github.com/visorcraft/grexa`
- show GPL 3.0 licensing
- add localized strings and translation tests
- support system/light/dark/high-contrast themes
- include About in UI smoke tests
