# Grex Context Preview Audit

This document records Grex `Controls/ContextPreviewDialog.xaml`,
`Controls/ContextPreviewDialog.xaml.cs`, `Services/ContextPreviewService.cs`,
and Search tab integration for context preview behavior.

Source evidence:

- `Controls/ContextPreviewDialog.xaml`
- `Controls/ContextPreviewDialog.xaml.cs`
- `Services/ContextPreviewService.cs`
- `Models/ContextPreviewResult.cs`
- `Controls/SearchTabContent.xaml`
- `Controls/SearchTabContent.xaml.cs`
- `Services/SettingsService.cs`
- `Tests/Services/ContextPreviewServiceTests.cs`
- `docs/features.md`
- `docs/usage.md`

## User Workflow

Context Preview is available from content search results.

User paths:

- select a content result and press Space
- right-click a content result and choose `Preview (Space)`

The dialog shows:

- file name and match line number
- surrounding context lines
- line numbers in a gutter
- the match line highlighted with a blue indicator strip and blue background
- `Open in Editor` primary button
- `Close` button

The context line count is configured in Settings:

- default lines before: 5
- default lines after: 5
- settings clamp each value to `1..20`

Grexa replacement:

- Preserve Space-to-preview and right-click Preview on content rows.
- Preserve configurable before/after line counts.
- Make preview target-aware for local, container-mirror, and direct-container
  result paths.

## Dialog Layout

`ContextPreviewDialog` is a WinUI `UserControl`.

Root sizing:

- minimum width 600
- maximum width 900
- maximum height 500

Rows:

1. file info header
2. scrollable context lines

Header:

- bordered area with card background
- 12x8 padding
- corner radius 4
- text is semi-bold, 13px, and character-ellipsis trimmed

Line list:

- `ItemsControl` inside a horizontal and vertical `ScrollViewer`
- each `ContextLine` row uses three columns:
  - 4px match indicator
  - line number gutter, minimum width 50
  - line content
- line numbers and content use `Consolas` at 12px
- line content does not wrap

Match styling:

- match indicator: `DodgerBlue`
- non-match indicator: transparent
- match line background: semi-transparent ARGB `40, 30, 144, 255`
- non-match line background: transparent

Data methods:

- `SetData(ContextPreviewResult)` sets header to
  `<FileName> : Line <MatchLineNumber>` and binds `LinesItemsControl.ItemsSource`
- `SetFileInfo(fileName, lineNumber, fileInfoFormat)` exists but is not used by
  current Search tab integration

Grexa replacement:

- Use a native dialog or side panel with monospace text, stable gutters, and a
  clear match-line highlight.
- Support long lines with horizontal scrolling.
- Localize the file info header instead of hard-coding `Line`.
- Scroll to the match line when ranges grow beyond the visible area.

## Service Contract

`ContextPreviewService.GetContextAsync` signature:

```csharp
Task<ContextPreviewResult> GetContextAsync(
    string filePath,
    int lineNumber,
    int linesBefore = 5,
    int linesAfter = 5)
```

Inputs:

- `filePath` is a full path
- `lineNumber` is 1-based
- `linesBefore` defaults to 5
- `linesAfter` defaults to 5

Validation:

- null or empty file path throws `ArgumentException`
- line number less than 1 throws `ArgumentException`
- `linesBefore` and `linesAfter` are not validated in the service

Result fields:

- `FileName`: `Path.GetFileName(filePath)`
- `FullPath`: original input path
- `MatchLineNumber`: requested line number
- `Lines`: context line list
- `MatchLineIndex`: index of the match line in `Lines`, or 0 when not found

Range calculation:

- `startLine = max(1, lineNumber - linesBefore)`
- `endLine = lineNumber + linesAfter`
- reads lines from `startLine` through `endLine`, inclusive
- stops reading once `currentLine > endLine`

Reading behavior:

- detects file encoding before opening the stream
- opens `StreamReader(filePath, detectedEncoding, detectEncodingFromByteOrderMarks: false)`
- reads line by line asynchronously
- skips earlier lines instead of storing them
- stops after the requested range

Important side effect:

- `EncodingDetectionService.DetectFileEncoding` reads the whole file into memory
  before the preview stream starts. Context preview itself streams only the
  requested line range, but encoding detection is not range-limited.

Error behavior:

- `IOException` becomes `InvalidOperationException("Failed to read file: ...")`
- `UnauthorizedAccessException` becomes
  `InvalidOperationException("Access denied to file: ...")`
- other exceptions are not wrapped by `ContextPreviewService`

Edge behavior:

- empty file returns an empty line list
- first-line previews only include available following lines
- last-line previews only include available preceding lines
- line numbers beyond end-of-file return whatever lines fall in the requested
  range and set `MatchLineIndex` to 0 if the requested line is absent

Grexa replacement:

- Preserve 1-based line numbers and inclusive before/after ranges.
- Validate or clamp before/after counts at the service boundary, not only in UI
  settings.
- Avoid full-file reads for encoding detection where possible.
- Preserve precise errors for missing files, permissions, and decode failures.

## WSL Detection

`ContextPreviewService.IsWslPath` returns true for:

- `\\wsl$\...`
- `\\wsl.localhost\...`
- `/mnt/...`
- `\mnt\...`
- any path that starts with `/` and is not a drive-letter path

The Search tab also has `ConvertWslPathToWindows`:

- already-Windows paths return unchanged
- `/mnt/<drive>/...` becomes `<DRIVE>:\...`
- WSL Unix paths are converted to `\\wsl.localhost\<distribution>\...`
- distribution is inferred from the current search path when it is WSL UNC
- otherwise the default WSL distribution is queried
- final fallback distribution is `Ubuntu-24.04`

Grexa replacement:

- Drop WSL runtime conversion from normal preview behavior.
- Keep Grex import/migration handling for old WSL-style saved paths.
- Treat Linux absolute paths as local native paths.

## Search Tab Integration

The Search tab owns:

```csharp
private readonly ContextPreviewService _contextPreviewService =
    new ContextPreviewService(new EncodingDetectionService());
```

Keyboard integration:

- `ResultsListView.PreviewKeyDown` handles Space
- requires the selected item to be a `SearchResult`
- marks the key event handled
- starts `ShowContextPreviewAsync(result)` without awaiting it

Right-click integration:

- content row right-click checks Docker mode first
- Docker mode shows a Docker-specific menu and returns
- local/WSL content rows show a custom `MenuFlyout`
- the first menu item is Preview with a view glyph
- clicking it starts `ShowContextPreviewAsync(result)`
- the menu then includes Show in Explorer, Copy Path, and Copy File Name

Dialog integration:

- invalid result or blank full path logs and returns
- converts WSL path to Windows path
- reads line counts from `SettingsService`
- loads context through `ContextPreviewService`
- creates `ContextPreviewDialog` and calls `SetData`
- localizes title, primary button, and close button with English fallbacks
- `Open in Editor` primary result calls `OpenFileInEditor(result)`
- errors show a localized `Preview Error` dialog with the exception message

Current Docker/container gap:

- right-click in Docker mode does not offer Preview
- Space handling does not explicitly guard Docker mode
- direct Docker results can have container paths that are not readable by
  `ContextPreviewService`
- mirror results may be readable locally, but the current code path is not
  explicit about direct vs mirrored preview behavior

Grexa replacement:

- Implement preview capability per target:
  - local files: direct read
  - mirrored container results: read mirror file while displaying container path
  - direct container results: execute runtime read command or disable with clear
    status if unavailable
- Disable or hide preview when a target cannot support it.
- Avoid fire-and-forget preview tasks; route errors through the UI model.

## Open In Editor

The Context Preview dialog primary button uses Search tab `OpenFileInEditor`.

Current behavior:

- converts WSL paths first
- checks `File.Exists` for local Windows paths
- assumes WSL UNC paths can be attempted without existence checks
- `.env` files try Notepad++ with `-n<line> -c<column>`
- PHP-family files try PhpStorm, but without line/column arguments
- other files use shell file association and do not pass line/column
- failures fall back to shell opening the file path

Grexa replacement:

- Use configured editor command templates.
- Ship presets for Kate/KWrite, VS Code, VSCodium, JetBrains IDEs, Sublime
  Text, GNOME Text Editor, Neovim terminal wrapper, and default `xdg-open`.
- Preserve line and column where the selected editor supports it.
- For container results, offer copy container path/runtime command when opening
  is not meaningful.

## Localization

Context Preview keys in `Strings/en-US/Resources.resw` include:

- `ContextPreviewTitle`
- `ContextPreviewOpenInEditorButton`
- `ContextPreviewMenuItem`
- `ContextPreviewLoadingText`
- `ContextPreviewErrorTitle`
- `ContextPreviewErrorText`
- `CloseButton`

Observed behavior:

- title and buttons use localized values with fallback strings
- context menu item uses localized value with fallback
- error dialog uses localized title/text with fallback
- `ContextPreviewLoadingText` exists but is not used in the current code path
- dialog header text from `SetData` is not localized
- error dialog close text is hard-coded to `OK`

Grexa replacement:

- Localize all visible text, including file info header and error close button.
- Use translation placeholders for file name, line number, and error message.
- Add localization tests for the Context Preview keys.

## Test Coverage

Existing tests cover:

- normal middle-of-file range
- first-line range clipping
- last-line range clipping
- default 5 before and 5 after
- empty file
- null file path
- invalid line number
- nonexistent file error
- WSL path detection for UNC, `/mnt`, Linux absolute path, drive-letter paths,
  and empty path

Current test gaps:

- no tests for UTF-8 with BOM, UTF-16, or other detected encodings
- no tests for permission denied
- no tests for line number beyond EOF
- no tests for negative `linesBefore` or `linesAfter`
- no UI tests for Space key, right-click Preview, dialog rendering, or Open in
  Editor
- no container preview tests
- no tests for localized header/error text

Grexa should add tests for all target types, encoding edge cases, and preview
task cancellation/error propagation.

## Current Grexa Status

Grexa now includes `grexa-core::context_preview`, `ContextPreviewDialog.qml`,
and `SearchController::preview_at`. The Search page opens the dialog from
result rows and keyboard flow, and Settings stores before/after line counts
with the core `1..20` clamp.

Remaining gaps:

- add mirrored/direct container preview support
- add localized error strings for preview failures
- broaden tests for permissions, missing files, encodings, and container paths
- add UI automation for keyboard and pointer preview entry points
