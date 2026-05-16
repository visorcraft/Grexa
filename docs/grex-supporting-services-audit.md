# Grex Supporting Services Audit

This document records the behavior of five Grex services that sit alongside the
search pipeline and provide cross-cutting infrastructure: result export,
filesystem context menus, user notifications, string localization, and
localized tooltip wiring. Grexa must either preserve, replace with a
Linux-native equivalent, or document the behavior as intentionally
non-applicable.

Source evidence:

- `Services/ExportService.cs`
- `Services/ContextMenuService.cs`
- `Services/NotificationService.cs`
- `Services/LocalizationService.cs`
- `Services/LocalizedToolTipRegistry.cs`
- `Tests/Services/ExportServiceTests.cs`
- `Tests/Services/NotificationServiceTests.cs`
- `Tests/Services/LocalizationServiceTests.cs`
- `Tests/Services/LocalizationServiceTestCollection.cs`
- `Tests/ContextMenuServiceTests.cs`

## ExportService

### Purpose

`Services/ExportService.cs` converts search results into export-friendly text
formats (CSV, JSON, tab-separated clipboard payloads) and writes those payloads
to disk or to the system clipboard. It is the integration point between the
in-memory result models (`SearchResult`, `FileSearchResult`) and the user's
external tools (spreadsheets, scripts, other editors).

### Public API

All methods are instance members on `Grex.Services.ExportService`. The service
has no constructor dependencies and is safe to instantiate per call.

- `string ExportContentResultsToCsv(IEnumerable<SearchResult> results)` returns
  a CSV string with header
  `FileName,LineNumber,ColumnNumber,LineContent,FullPath,RelativePath`.
  Individual fields are escaped via `EscapeCsvField`. Returns header only for
  an empty input.
- `string ExportFileResultsToCsv(IEnumerable<FileSearchResult> results)`
  returns a CSV string with header
  `FileName,Size,MatchCount,Extension,Encoding,DateModified,FullPath,RelativePath`.
  `DateModified` is formatted as `yyyy-MM-dd HH:mm:ss`.
- `string ExportContentResultsToJson(IEnumerable<SearchResult> results)`
  returns a `WriteIndented = true` JSON array using `System.Text.Json`.
- `string ExportFileResultsToJson(IEnumerable<FileSearchResult> results)`
  returns a `WriteIndented = true` JSON array including `FormattedSize` plus
  the raw `Size` and `DateModified` values.
- `string ExportContentResultsToClipboard(IEnumerable<SearchResult> results)`
  returns a tab-separated string with header
  `FileName\tLine\tColumn\tContent\tPath` using `TrimmedLineContent` and
  `RelativePath`.
- `string ExportFileResultsToClipboard(IEnumerable<FileSearchResult> results)`
  returns a tab-separated string with header
  `FileName\tSize\tMatches\tExtension\tEncoding\tDateModified\tPath`.
- `void CopyToClipboard(string content)` sets the system clipboard via
  `Windows.ApplicationModel.DataTransfer.DataPackage` and
  `Clipboard.SetContent`. No return value, no exception path; clipboard
  failures will propagate from the WinRT runtime.
- `Task<bool> SaveToFileAsync(string content, string suggestedFileName, string fileTypeDescription, string fileExtension, IntPtr hwnd)`
  opens a `FileSavePicker`, writes UTF-8 text via `FileIO.WriteTextAsync`,
  returns `true` on save, `false` on cancel or error. All exceptions are
  swallowed and converted to `false`.
- `private string EscapeCsvField(string field)` wraps fields that contain
  commas, quotes, newlines, or carriage returns in double quotes and escapes
  internal quotes by doubling them.

### Side Effects

- System clipboard mutation through WinRT `Clipboard.SetContent` in
  `CopyToClipboard`.
- Filesystem write through `FileIO.WriteTextAsync` in `SaveToFileAsync`. The
  destination is chosen by the user through the picker; the service only
  consumes the resulting `StorageFile`.
- A modal Windows file picker dialog through `FileSavePicker` plus
  `InitializeWithWindow.Initialize(savePicker, hwnd)` to anchor it to the
  caller's HWND.
- No timers, no background threads, no logging.

### Windows-specific Behavior and Linux Equivalents

- `Windows.ApplicationModel.DataTransfer.DataPackage` and
  `Clipboard.SetContent` must be replaced. On Linux prefer
  `QGuiApplication::clipboard()` (or the KDE `KSystemClipboard`); fall back to
  `wl-copy` on Wayland and `xclip`/`xsel` on X11 by spawning a child process
  and piping the payload through stdin.
- `FileSavePicker` plus `InitializeWithWindow` is WinUI/WinRT specific. Replace
  with `QFileDialog::getSaveFileName` (Qt) or `Gtk.FileChooserNative` (GTK).
  Both honor `xdg-desktop-portal` so portal/Flatpak sandboxing keeps working.
- `FileIO.WriteTextAsync` may emit UTF-8 with a BOM. Use
  `File.WriteAllTextAsync(path, content, new UTF8Encoding(false))` instead so
  the file stays friendly to Linux CLIs.
- The `IntPtr hwnd` parameter has no Linux analog; replace it with a
  parent-window abstraction or drop it entirely (the portal needs no parent).
- `DateModified` is formatted `yyyy-MM-dd HH:mm:ss`, which is portable but
  emitted in local time. Document this explicitly in Grexa.

### Test Coverage Observations

`Tests/Services/ExportServiceTests.cs` covers:

- CSV/JSON/clipboard happy paths for both content and file result shapes.
- Empty input cases (`ExportContentResultsToCsv_WithEmptyResults_ReturnsHeaderOnly`,
  `ExportContentResultsToJson_WithEmptyResults_ReturnsEmptyArray`,
  clipboard header-only case).
- CSV escaping for commas, quotes, and newlines.
- JSON serialization with embedded quotes and backslashes.
- Null `LineContent` is treated as empty by `EscapeCsvField`.
- Zero `Size` and `MatchCount` for file results.

Gaps:

- No test exercises `CopyToClipboard` (the WinRT API requires a UI thread).
- No test exercises `SaveToFileAsync` (requires a real HWND and picker).
- No assertion on the literal `yyyy-MM-dd HH:mm:ss` formatting or on
  locale-sensitivity of CSV decimal output.

### Grexa Replacement

- Keep the pure-text formatting methods (`ExportContent*ToCsv`,
  `ExportFile*ToCsv`, `ExportContent*ToJson`, `ExportFile*ToJson`,
  `ExportContent*ToClipboard`, `ExportFile*ToClipboard`, and `EscapeCsvField`)
  unchanged — they have no Windows dependencies and are already covered by
  unit tests.
- Define an `IClipboard` interface and inject a Linux implementation that
  prefers a Qt `QClipboard` or `KSystemClipboard` when running inside the
  desktop process, and falls back to a process-spawn helper that tries
  `wl-copy` (Wayland) then `xclip -selection clipboard` then `xsel -b -i` (X11).
- Define an `ISaveFileDialog` interface that returns a `Task<string?>` (chosen
  path) and have the Linux implementation use the host UI toolkit's native
  dialog. Drop the `IntPtr hwnd` parameter; pass a typed parent token
  instead. Have `SaveToFileAsync` orchestrate "ask for path → write bytes"
  rather than coupling the picker and writer.
- Port `EscapeCsvField` verbatim and reuse the existing xUnit fixtures from
  Grex (`ExportServiceTests`) as the regression baseline.
- Add at least one integration test that round-trips through the Linux
  clipboard backend on both Wayland and X11 sessions.

## ContextMenuService

### Purpose

`Services/ContextMenuService.cs` opens a context menu over a file or folder
discovered by the search UI. The current Windows implementation uses
WinRT `StorageFolder` plus a `MenuFlyout` with hard-coded items
("Open", "Open with...", "Copy path", "Copy", "Rename", "Delete",
"Properties"), and it shells out to `explorer.exe`, `rundll32.exe`, and
`wsl.exe wslpath -w` to talk to the Windows shell and to WSL.

### Public API

- `ContextMenuService()` parameterless constructor; obtains
  `NotificationService.Instance` for surfacing error toasts.
- `void ShowContextMenu(string filePath, int screenX, int screenY, UIElement? targetElement = null)`
  fire-and-forget wrapper that calls `ShowContextMenuAsync` and ignores the
  returned task.
- `Task ShowContextMenuAsync(string filePath, int screenX, int screenY, UIElement? targetElement = null)`
  normalizes WSL paths, validates that the target exists, resolves a
  `XamlRoot` (from the supplied element, then `Window.Current`, then by
  walking up the visual tree), builds a `MenuFlyout`, and calls
  `ShowAt(null, new Point(screenX, screenY))`. On any unrecoverable failure
  it calls `ShowCustomMenu` (a slimmer fallback flyout with just "Open" and
  "Copy Path") and may surface a `NotificationService.ShowError` toast.

Private helpers wired into the menu items:

- `OpenFile(string)` spawns `Process.Start` with `UseShellExecute = true` so
  Windows uses the default verb.
- `OpenFileWith(string)` shells out to
  `rundll32.exe shell32.dll,OpenAs_RunDLL <path>` to invoke the
  Windows "Open with" picker.
- `CopyPath(string)` writes the path string to the WinRT clipboard.
- `CopyFile(string)` uses `StorageFile.GetFileFromPathAsync` or
  `StorageFolder.GetFolderFromPathAsync` synchronously (via
  `.GetAwaiter().GetResult()`) and puts the storage item into the clipboard
  as a CF_HDROP-style payload.
- `RenameFile(string)` launches `explorer.exe /select,"<path>"` — note this
  does not actually trigger F2/rename, it only highlights the file.
- `DeleteFile(string)` calls `File.Delete` or `Directory.Delete(_, true)`.
  There is no Recycle Bin / trash redirection and no confirmation dialog.
- `ShowProperties(string)` is identical to `RenameFile` — it shells to
  `explorer.exe /select` and only highlights the file; the comment in the
  code acknowledges that `shell32.dll,ShellExec_RunDLL` would be required to
  show a real properties dialog.
- `NormalizeWslPath(string)` accepts `\\wsl.localhost\<distro>\...`,
  `\\wsl$\<distro>\...`, raw Linux paths starting with `/home/` or `/mnt/`,
  and the same paths with backslashes. It throws `NotSupportedException` for
  WSL1 paths containing `\AppData\Local\lxss\`.
- `ConvertLinuxPathToWsl(string)` runs `wsl wslpath -w "<path>"`, captures
  stdout, and falls back to
  `\\wsl$\<WindowsSubsystemLinuxService.GetDefaultDistributionName()>\...`
  (defaulting to `Ubuntu-24.04`) when the conversion command fails.
- `static void Log(string)` appends timestamped lines to
  `Path.Combine(Path.GetTempPath(), "Grex.log")`.

### Side Effects

- Spawns multiple external processes: `explorer.exe`, `rundll32.exe`, and
  `wsl.exe`.
- Writes to `%TEMP%\Grex.log` on every public call (and on most error paths).
- Mutates the Windows clipboard with a path string or with `IStorageItem`
  references.
- Performs filesystem mutations: `File.Delete` and recursive
  `Directory.Delete`. No undo, no confirmation, no trash.
- Surfaces error toasts through `NotificationService`.
- Renders a UI flyout anchored to the WinUI `XamlRoot`.

### Windows-specific Behavior and Linux Equivalents

This service is the most Windows-coupled of the five — almost every concrete
operation needs replacement.

- `Microsoft.UI.Xaml` `MenuFlyout` is WinUI-only. Replace with a native
  context menu in the host toolkit: Qt `QMenu`, or KDE's
  `KFileItemActions::addOpenWithActionsTo` for closest parity (it also
  injects installed ServiceMenus).
- `Process.Start(... UseShellExecute = true ...)` for "Open" must be replaced
  with `xdg-open` (preferred), or `gio open`/`kde-open5` when present.
- `rundll32.exe shell32.dll,OpenAs_RunDLL` for "Open with..." has no direct
  CLI replacement. Use KIO `KOpenWithDialog` via a Qt bridge, or build a
  custom submenu populated from `xdg-mime query filetype` plus the desktop
  entries discoverable through `gio info` / `gtk-launch -l`.
- `explorer.exe /select,"<path>"` for Rename/Properties must go. Use
  `xdg-open "$(dirname <path>)"` to highlight the folder, or call
  `org.freedesktop.FileManager1.ShowItems` /
  `org.freedesktop.FileManager1.ShowItemProperties` over D-Bus for a real
  "show in file manager" / "Properties" experience.
- `Process.Start("wsl", "wslpath -w ...")` and the entire WSL normalization
  branch are not applicable on Linux. Drop `NormalizeWslPath` and
  `ConvertLinuxPathToWsl` entirely.
- `File.Delete` / `Directory.Delete` must move to trash-aware deletion via
  `gio trash <path>` or the freedesktop trash spec
  (`$XDG_DATA_HOME/Trash`). Hard delete should require explicit user
  confirmation.
- Clipboard for `CopyPath` / `CopyFile` should funnel through the shared
  `IClipboard` shim. For file copy, put a `text/uri-list` payload of
  `file:///...` URIs plus `x-special/gnome-copied-files` onto the
  clipboard — Dolphin/Nautilus/Thunar all accept this as a file copy.
- The log file should move out of `Path.GetTempPath()` to
  `$XDG_STATE_HOME/grexa/grexa.log` (falling back to
  `$HOME/.local/state/grexa/grexa.log`).
- The synchronous `.GetAwaiter().GetResult()` calls inside `CopyFile` are
  deadlock-prone on a UI thread. The Linux port should fully `await`
  through `IClipboard.SetUrisAsync`.

### Test Coverage Observations

`Tests/ContextMenuServiceTests.cs` covers:

- Instance construction.
- "Does not throw" smoke tests for Windows paths, WSL paths, and invalid
  inputs (empty/null/whitespace).
- A single async happy path against a real temp file.

`Tests/SearchTabContentRightClickTests.cs` and
`IntegrationTests/RightClickContextMenuTests.cs` exist as additional
references (not part of this audit's scope, but they should be reviewed
when porting).

Gaps:

- No test verifies which menu items appear, no test exercises any item's
  click handler.
- `Process.Start` invocations, clipboard writes, and `File.Delete` paths are
  unmocked and unverified.
- WSL normalization has no negative-path coverage beyond the
  `NotSupportedException` constant.

### Grexa Replacement

- Treat `ContextMenuService` as a near-total rewrite. Define a new
  `IFileContextMenu` interface with verbs: `Open`, `OpenWith`, `CopyPath`,
  `CopyToClipboard`, `MoveToTrash`, `ShowInFileManager`, `ShowProperties`.
- Provide a Linux implementation backed by a thin shell over `xdg-open`,
  `gio trash`, `org.freedesktop.FileManager1` D-Bus, and the `IClipboard`
  shim. Prefer D-Bus over shelling out where reasonable, since spawning
  subprocesses is slower and harder to mock.
- Drop `NormalizeWslPath`, `ConvertLinuxPathToWsl`, and any references to
  `WindowsSubsystemLinuxService` from the port.
- Move the log file to an XDG-compliant location and add structured logging
  (e.g., `Microsoft.Extensions.Logging`) rather than ad-hoc
  `File.AppendAllText`.
- Add unit tests around the verb dispatcher using an `IProcessRunner`
  / `IDBusClient` fake so menu invocations can be asserted without spawning
  real processes.
- Add an integration test that exercises `gio trash` on a temp file inside a
  containerized Linux runner.

## NotificationService

### Purpose

`Services/NotificationService.cs` is a singleton that sends transient
notifications ("toasts") to the user for errors, warnings, info, and success
states. It is the only service in the supporting set that has no return value
flowing back to the UI — it is a fire-and-forget side channel.

### Public API

- `static NotificationService Instance` is a thread-safe lazy singleton.
- `void ShowError(string title, string message)` builds a `ToastGeneric`
  XML payload, constructs a `Microsoft.Windows.AppNotifications.AppNotification`,
  and calls `AppNotificationManager.Default.Show`. Catches all exceptions and
  logs them.
- `void ShowError(string title, Exception exception)` truncates the exception
  message to 200 chars (suffix `...`), defers to `ShowError(string, string)`,
  and always logs full exception details to file. Null `exception` is treated
  as `"An unknown error occurred."`.
- `void ShowInfo(string title, string message)` same shape as `ShowError`.
- `void ShowWarning(string title, string message)` same shape.
- `void ShowSuccess(string title, string message)` same shape.
- `void Initialize()` triggers `EnsureSupport` + `EnsureRegistration` at
  startup.
- `bool CheckSupport()` invokes `EnsureSupport` and returns its bool.
- `bool CheckRegistration()` invokes `EnsureRegistration` and returns
  `_isRegistered`.
- `string NotificationLogFilePath` returns
  `Path.Combine(Path.GetTempPath(), "Grex.log")`.
- `string? SupportFailureDetails` and `string? RegistrationFailureDetails`
  expose the most recent failure reason.

Private members:

- `EnsureSupport()` calls `AppNotificationManager.IsSupported()` once and
  memoizes the result plus a failure message.
- `EnsureRegistration()` calls `AppNotificationManager.Default.Register()`
  once, guarded by a lock. It intentionally attempts registration even when
  `IsSupported` returns `false` because unpackaged apps can return false
  even when toasts work; on success it back-fills `_isSupported = true`.
- `EscapeXml(string)` escapes `&`, `<`, `>`, `"`, `'`.
- `LogToFile(string)` writes timestamped lines to
  `Path.Combine(Path.GetTempPath(), "Grex.log")`.

### Side Effects

- Calls into `Microsoft.Windows.AppNotifications` (Windows App SDK / WinRT)
  to register an `AppNotificationManager` and to display toasts.
- Mutates the system notification queue (toasts visible to the user in the
  Windows Action Center).
- Appends entries to `%TEMP%\Grex.log` for every notification attempt,
  success, and failure path.
- Holds two locks: `_lock` for the singleton and `_registrationLock` for
  one-time `Register()`.

### Windows-specific Behavior and Linux Equivalents

- `AppNotificationManager.Register()` / `Show(AppNotification)` are Windows
  App SDK only. The `ToastGeneric` XML payload and the `IsSupported`
  heuristic should be discarded entirely.
- Linux replacement: emit notifications via the
  `org.freedesktop.Notifications` D-Bus interface (implemented by GNOME
  Shell, KDE Plasma, dunst, mako, swaync). In-process, the cleanest binding
  is `Tmds.DBus`; `libnotify` via P/Invoke is acceptable too.
- For KDE-native integration prefer KNotifications (`KNotification::event`)
  so notifications honor per-event policy from System Settings and gain
  sound, urgency, and history behaviors the freedesktop spec omits.
- Drop the XML payload. The freedesktop API takes `app_name`,
  `replaces_id`, `app_icon`, `summary`, `body`, `actions[]`, `hints{}`,
  `expire_timeout` — strings, not XML.
- Remove the unpackaged-app workaround in `EnsureRegistration`; D-Bus has
  no analogous registration step.
- Keep `EscapeXml` only if Grexa opts into KDE's small HTML-like body
  markup, and gate it on the server's `body-markup` capability.
- Move the log file to the XDG state directory (see `ContextMenuService`).

### Test Coverage Observations

`Tests/Services/NotificationServiceTests.cs` covers:

- Singleton identity (`Instance` returns the same reference).
- "Does not throw" smoke tests for `ShowError(string, string)`,
  `ShowError(string, Exception)`, `ShowError(string, null Exception)`,
  long exception messages, `ShowInfo`, `ShowWarning`, `ShowSuccess`, empty
  strings, special characters, and exceptions with inner exceptions.

Gaps:

- No test verifies that the rendered XML payload escapes special characters
  (it only verifies the call does not throw).
- No test asserts the 200-character truncation behavior.
- No test exercises `EnsureRegistration` failure modes — the registration
  is global state and cannot easily be reset between tests.
- No assertion on what is written to `Grex.log`.

### Grexa Replacement

- Define `INotificationService` with `ShowError`, `ShowInfo`, `ShowWarning`,
  `ShowSuccess` methods that take `string title, string message` (and an
  `ShowError(string, Exception)` convenience overload, with the same
  200-char message truncation).
- Implement a primary backend using `KNotifications` when running inside a
  KDE session and a freedesktop D-Bus fallback elsewhere. Detect at
  startup via `XDG_CURRENT_DESKTOP`, but always be ready to fall back to
  D-Bus when KNotifications is unavailable.
- Remove `IsSupported` / `Register` / `EnsureRegistration`. Replace with a
  one-shot D-Bus connection that is created lazily on first
  `Show*` call and cached.
- Keep singleton semantics but inject the backend so tests can swap in an
  in-memory recorder.
- Add unit tests that assert exact body, summary, and urgency hints for
  each level (`low`, `normal`, `critical` map to info/warning, info,
  error in the freedesktop spec).
- Switch logging to `ILogger<NotificationService>` with category-level
  filters; do not write directly to `/tmp` from this service.

## LocalizationService

### Purpose

`Services/LocalizationService.cs` is a singleton that resolves string
resources by key. It wraps `Microsoft.Windows.ApplicationModel.Resources`
(the modern WinUI 3 resource API backed by `.resw`/PRI files) and exposes a
small surface to the rest of the app: `GetLocalizedString(key)`,
`GetLocalizedString(key, args)`, `SetCulture(culture)`, and a
`PropertyChanged` event so observers (notably `LocalizedToolTipRegistry`)
can refresh themselves when culture changes.

### Public API

- `static LocalizationService Instance` is a singleton; lazy initialization
  is unsynchronized (`?? = new(...)`) which is a latent race in
  multi-threaded startup, although the inner constructor is defensive enough
  that the race is benign.
- `string CurrentCulture { get; private set; }` defaults to system
  `CultureInfo.CurrentUICulture.Name` if valid, otherwise `"en-US"`.
- `event PropertyChangedEventHandler? PropertyChanged` fires when
  `CurrentCulture` changes.
- `void Initialize()` is a no-op (the constructor does the work).
- `string GetLocalizedString(string key)` returns the resolved string. If
  the key is empty/null returns `string.Empty`; if the key cannot be
  resolved in `CurrentCulture` or in `en-US`, returns the key itself. Any
  exception is swallowed and the key is returned.
- `string GetLocalizedString(string key, params object[] args)` calls the
  single-arg overload, then `string.Format(format, args)`. On
  `FormatException` returns the unformatted format string.
- `void SetCulture(string culture)` validates with
  `CultureInfo.GetCultureInfo`, falls back to `"en-US"` on
  `CultureNotFoundException`, updates the current culture, sets
  `Thread.CurrentThread.CurrentCulture` /
  `Thread.CurrentThread.CurrentUICulture`,
  `CultureInfo.DefaultThreadCurrentCulture` /
  `DefaultThreadCurrentUICulture`, calls
  `ApplicationLanguages.PrimaryLanguageOverride = culture`, and clears the
  resource-context cache.

Private helpers:

- `GetStringForCulture(key, culture)` resolves through the cached
  `ResourceContext`. It tries the key, `key.Replace('.', '/')`,
  `key.Replace('.', '_')`, against both the `Resources` subtree and the
  prefixed `Resources/<key>` path of the main map.
- `GetResourceContext(culture)` lazily creates and caches a `ResourceContext`
  per culture, with `QualifierValues["Language"] = culture`. Throws
  `InvalidOperationException` if `_resourceManager` is null.
- `IsValidCulture(string)` is a `try`/`catch` around
  `CultureInfo.GetCultureInfo`.
- `ClearResourceContextCache()` empties the per-culture context dictionary
  whenever the culture is changed.
- `UpdateDefaultResourceContext(string)` exists but is effectively a
  no-op — it acknowledges in a comment that WinUI 3 has no public API to
  set a process-wide default `ResourceContext` for `x:Uid` resolution.

### Side Effects

- Creates a `Microsoft.Windows.ApplicationModel.Resources.ResourceManager`
  and indexes into its main resource map. Constructor failures are
  swallowed (twice) and the service degrades to "every key returns the
  key".
- Mutates `Thread.CurrentThread.CurrentCulture`,
  `Thread.CurrentThread.CurrentUICulture`,
  `CultureInfo.DefaultThreadCurrentCulture`, and
  `CultureInfo.DefaultThreadCurrentUICulture` on culture change.
- Mutates `Windows.Globalization.ApplicationLanguages.PrimaryLanguageOverride`.
- Raises `PropertyChanged("CurrentCulture")` on change.
- Writes debug-only diagnostics via `Debug.WriteLine`.
- Holds an internal lock (`_resourceContextLock`) around the context
  dictionary; the `Instance` getter itself is unsynchronized.

### Windows-specific Behavior and Linux Equivalents

- `ResourceManager` / `ResourceMap` / `ResourceContext` / `ResourceCandidate`
  are WinUI 3 PRI-backed APIs with no Linux port.
- `Windows.Globalization.ApplicationLanguages.PrimaryLanguageOverride` is a
  WinRT API. Linux honors `LANG` / `LC_*` env vars; in-process overrides
  are toolkit-specific (Qt `QTranslator`, GTK `gettext`).
- Linux replacement: GNU `gettext` (`.po`/`.mo`) via `NGettext`, which
  provides `Catalog.GetString(msgid)`, `GetPluralString`, and
  context-aware variants and integrates with `xgettext`/Weblate/Transifex.
- Mapping:
  - `GetLocalizedString(key)` → `catalog.GetString(key)`; missing
    translations return the msgid (key), matching Grex semantics.
  - `GetLocalizedString(key, args)` → `catalog.GetString(key)` then
    `string.Format`; preserve the `FormatException` fallback.
  - `SetCulture(culture)` → reload the `Catalog`, raise
    `PropertyChanged`, and continue setting
    `Thread.CurrentThread.CurrentCulture` so .NET date/number formatting
    follows the same locale.
  - Drop `ApplicationLanguages.PrimaryLanguageOverride` entirely.
- Drop the BCL key-variant logic (`key`, `'.'→'/'`, `'.'→'_'`). `gettext`
  keys are opaque strings. Keep only the final "return the key when not
  found" fallback.
- Fix the unsynchronized singleton initialization with `Lazy<T>` or
  `LazyInitializer.EnsureInitialized`.

### Test Coverage Observations

`Tests/Services/LocalizationServiceTests.cs` is mostly skipped — the
fixture comments call this out explicitly:

> Note: LocalizationService tests are limited because Windows ResourceLoader
> requires app context which may not be available in test environments.

Active tests:

- Empty key returns empty string (tested at the language level, not via
  the service).
- Null key returns empty string (same).
- Invalid culture string falls back to `"en-US"` (tested at
  `CultureInfo.GetCultureInfo` level, not via the service).
- Empty culture leaves current culture unchanged (logic-level test).

`Tests/Services/LocalizationServiceTestCollection.cs` exists only to mark
the collection so xUnit runs the tests sequentially, avoiding races on the
singleton.

Other related tests (out of scope for direct audit) include
`RegexBuilderLocalizationKeysTests`, `RegexBuilderLanguageSwitchingTests`,
and `RegexBuilderLanguageIntegrationTests` which exercise localization
indirectly.

Gaps:

- No active integration test loads a real resource file and asserts a
  translation round-trip — this is a major coverage gap that the Linux
  port should fix.
- No test exercises the `BuildKeyVariants` fallback chain.
- No test exercises `SetCulture` actually changing returned strings.

### Grexa Replacement

- Replace `ResourceManager` plumbing with `NGettext.Catalog` keyed on the
  current locale. Store `.po` files under `share/locale/<lang>/LC_MESSAGES/grexa.po`.
- Keep the `ILocalizationService` interface intact so the rest of the app
  does not change.
- Drop `BuildKeyVariants`, `_resourceMap`, `_resourceContexts`,
  `GetStringForCulture`, `TryGetCandidate`, `GetResourceContext`, and
  `UpdateDefaultResourceContext`.
- Continue to raise `PropertyChanged(nameof(CurrentCulture))` on
  `SetCulture` so `LocalizedToolTipRegistry` (and any future
  observers) refresh.
- Synchronize singleton creation with `Lazy<T>` to remove the latent race.
- Write proper integration tests that assert: known key → translated
  value; missing key → key as fallback; format-args-with-bad-format →
  unformatted fallback; culture change → re-resolved value.

## LocalizedToolTipRegistry

### Purpose

`Services/LocalizedToolTipRegistry.cs` is a static helper that keeps WinUI
`FrameworkElement` tooltips and `AutomationProperties.HelpText` in sync
with the current culture. Each registration links an element (held as a
`WeakReference`) to a localization key; whenever
`LocalizationService.CurrentCulture` changes, every live registration is
re-resolved and re-applied on the element's `DispatcherQueue`.

### Public API

- `static void Register(FrameworkElement? element, string resourceKey)`
  registers (or updates) a weak reference to `element` keyed by
  `resourceKey`. Calls `EnsureSubscribed` to attach a one-time
  `LocalizationService.Instance.PropertyChanged` handler, then applies the
  tooltip immediately.
- `static void RefreshRegisteredToolTips()` snapshots the registration list
  under the `_syncRoot` lock, cleans up dead weak references, and re-applies
  each registration's tooltip text.

Private members:

- `ToolTipRegistration` is a sealed nested type holding the weak reference
  plus the (mutable) resource key.
- `_registrations` is the shared backing list, guarded by `_syncRoot`.
- `_subscribedToLocalization` is a once-only flag.
- `EnsureSubscribed()` attaches `LocalizationServiceOnPropertyChanged`.
- `LocalizationServiceOnPropertyChanged(...)` calls
  `RefreshRegisteredToolTips()` only when `CurrentCulture` changes.
- `ApplyToolTip(element, key)` resolves the key, defaults the text back to
  the raw key when resolution returns empty or the same key, and routes to
  `SetToolTipText`.
- `SetToolTipText(element, text)` dispatches onto the element's
  `DispatcherQueue` if not already on the UI thread, else applies
  synchronously.
- `ApplyToolTipValues(element, text)` calls
  `ToolTipService.SetToolTip(element, text)` and
  `AutomationProperties.SetHelpText(element, text)`.
- `CleanupDeadRegistrationsLocked()` reverse-iterates the list and removes
  entries whose weak reference no longer points to a live element.

### Side Effects

- Mutates `ToolTipService` and `AutomationProperties` attached properties on
  registered elements.
- Subscribes (once, never unsubscribes) to
  `LocalizationService.Instance.PropertyChanged`.
- Marshals work onto the element's `Microsoft.UI.Dispatching.DispatcherQueue`
  when needed.
- Holds a static `List<ToolTipRegistration>` that grows over the app's
  lifetime, pruned only when `RefreshRegisteredToolTips` runs.
- Writes diagnostics through `Debug.WriteLine` on failure (no log file).

### Windows-specific Behavior and Linux Equivalents

- `Microsoft.UI.Xaml.FrameworkElement`,
  `Microsoft.UI.Xaml.Controls.ToolTipService`,
  `Microsoft.UI.Xaml.Automation.AutomationProperties`, and
  `Microsoft.UI.Dispatching.DispatcherQueue` are all WinUI 3 APIs.
- Linux replacement depends on the chosen UI toolkit:
  - For **Qt/QML or Qt Widgets**, swap `FrameworkElement` for `QWidget*`
    or `QObject*` (QML items), `ToolTipService.SetToolTip` for
    `QWidget::setToolTip` (Widgets) or the `ToolTip.text` attached
    property (QML), and `AutomationProperties.SetHelpText` for
    `QAccessibleWidget::setDescription` or QML's
    `Accessible.description`.
  - For **GTK4**, swap to `Gtk.Widget.SetTooltipText` and the
    `gtk_accessible_update_property` API for accessibility metadata.
- `DispatcherQueue.HasThreadAccess` and `TryEnqueue(...)` become Qt's
  `QMetaObject::invokeMethod(target, ..., Qt::QueuedConnection)` (or
  `QCoreApplication::instance()->thread() == QThread::currentThread()` for
  the thread-affinity check). GTK uses `g_main_context_invoke`.
- Weak references should continue to use
  `System.WeakReference<T>` — that is BCL, not Windows-specific. No change
  needed there.
- The registry is currently a static class; Grexa should keep it static
  (or at minimum keep it process-singleton) so that XAML/QML code-behind
  can register without DI plumbing.

### Test Coverage Observations

There are no tests for `LocalizedToolTipRegistry`. The closest indirect
coverage comes from `RegexBuilderLocalizationKeysTests` and
`RegexBuilderLanguageSwitchingTests`, which assert that the keys used at
the call sites exist in the resource files. Those tests do not verify
that the registry actually mutates tooltip text on culture change.

Gaps:

- Registration churn (registering many elements then letting them be
  collected) is untested. The registry only prunes on `RefreshRegisteredToolTips`,
  so if culture never changes the list grows unbounded.
- Re-registration of the same element with a new key is untested.
- Thread-safety of the static `_syncRoot` is asserted by inspection only.
- The `EnsureSubscribed` once-flag has no test guarding against duplicate
  subscriptions.

### Grexa Replacement

- Define `ILocalizedToolTipRegistry` (or keep it static if the host
  toolkit code can call directly) with `Register(object element, string
  resourceKey)` and `RefreshAll()` methods. Type the element parameter to
  whatever the chosen toolkit's base widget type is.
- Replace `ToolTipService` + `AutomationProperties` with the equivalent
  calls from the chosen toolkit (Qt: `setToolTip`, `setAccessibleDescription`;
  GTK: `set_tooltip_text`, accessibility property update).
- Replace `DispatcherQueue` with `Qt::QueuedConnection` /
  `g_main_context_invoke` equivalent. Keep the "if already on UI thread,
  apply synchronously" optimization.
- Keep the weak-reference + lock model. Add a periodic prune that runs
  even when culture never changes — e.g., trigger a cleanup pass every
  `N` registrations to keep the static list bounded.
- Add tests: register an element, observe tooltip text changes when
  `LocalizationService.SetCulture` is called; let the element go out of
  scope and assert it is pruned on the next refresh; assert that
  re-registering with a new key updates the stored key rather than
  duplicating the entry.

## Cross-Cutting Notes

A few patterns repeat across these five services and deserve a single
Grexa-wide answer rather than per-service rework:

- **Clipboard access** in `ExportService.CopyToClipboard`,
  `ContextMenuService.CopyPath`, and `ContextMenuService.CopyFile` should
  flow through one `IClipboard` abstraction.
- **Log destination** is hardcoded to `Path.GetTempPath()/Grex.log` in
  `ContextMenuService` and `NotificationService`. Move everything to
  `ILogger<T>` and configure an XDG-state sink at startup.
- **Process spawning** in `ContextMenuService` should hide behind an
  `IProcessRunner` so D-Bus calls and CLI fallbacks share one test surface.
- **Singletons** in `NotificationService` and `LocalizationService` should
  migrate to DI-registered single-instance services; this also removes the
  unsynchronized-instance race in `LocalizationService`.
- **UI thread affinity** in `LocalizedToolTipRegistry` (and implicitly in
  `ContextMenuService.ShowContextMenuAsync`) should funnel through an
  `IUIDispatcher` so service code stays toolkit-agnostic.
