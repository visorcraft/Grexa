# Grex Regex Builder Audit

This document records Grex `Controls/RegexBuilderView.xaml` and
`Controls/RegexBuilderView.xaml.cs` behavior that Grexa must preserve, replace,
or deliberately improve.

Source evidence:

- `Controls/RegexBuilderView.xaml`
- `Controls/RegexBuilderView.xaml.cs`
- `MainWindow.xaml`
- `MainWindow.xaml.cs`
- `Tests/Services/RegexBuilderLocalizationKeysTests.cs`
- `Tests/Services/RegexBuilderLanguageIntegrationTests.cs`
- `Tests/Services/RegexBuilderLanguageSwitchingTests.cs`
- `docs/usage.md`
- `docs/architecture.md`
- `docs/regex-localization.md`

## Role

Regex Builder is a standalone navigation page hosted by `MainWindow`, not part
of a search tab.

MainWindow hosts it under:

- `RegexBuilderNavItem`
- `RegexBuilderContentGrid`
- `RegexBuilderView`

There is no button that applies the pattern back to the active Search tab. Users
experiment with a sample text, presets, options, live match output, and a visual
syntax breakdown before manually using the pattern elsewhere.

Grexa replacement:

- Keep Regex Builder as a first-class tool page.
- Preserve standalone experimentation behavior.
- Consider adding explicit copy/apply actions later, but do not assume Grex has
  an apply-to-search workflow.

## Layout

`RegexBuilderView` is a WinUI `UserControl` named `RegexBuilderControl`.

The root layout is a padded grid with four rows:

1. top row: sample text and live match results
2. middle row: regex pattern input
3. presets row
4. bottom row: visual breakdown and options

Top row:

- left column contains `SampleTextTextBlock` and `SampleTextTextBox`
- center spacer is 24 pixels
- right column contains `LiveMatchResultsTextBlock` and a bordered
  scrollable `MatchResultsTextBlock`

Pattern row:

- `RegexPatternTextBlock`
- `RegexPatternTextBox`

Preset row:

- `PresetsTextBlock`
- five equal-width buttons:
  - `EmailPresetButton`
  - `PhonePresetButton`
  - `DatePresetButton`
  - `DigitsPresetButton`
  - `URLPresetButton`

Bottom row:

- left column contains `VisualRegexBreakdownTextBlock` and a bordered
  scrollable `BreakdownStackPanel`
- right column contains `OptionsTextBlock` and three check boxes:
  - `CaseInsensitiveCheckBox`
  - `MultilineCheckBox`
  - `GlobalMatchCheckBox`

Default option state:

- case insensitive: false
- multiline: false
- global match: true

Grexa replacement:

- Use a dense split layout with sample input, pattern input, presets, live
  result preview, syntax breakdown, and options visible without navigation.
- Preserve scrollable result and breakdown areas.
- Preserve the three options and the default global-match state.

## Live Evaluation

State:

- `_currentRegex` stores the last successfully compiled regex.
- `_isUpdating` suppresses recursive updates while setting text or refreshing
  localization.

Events:

- changing sample text updates match results
- changing pattern recompiles regex, updates match results, and updates
  breakdown
- changing any option recompiles regex and updates match results

Compilation:

- blank pattern clears `_currentRegex`
- invalid pattern catches `ArgumentException` and clears `_currentRegex`
- case-insensitive maps to `RegexOptions.IgnoreCase`
- multiline maps to `RegexOptions.Multiline`
- no other .NET regex options are exposed
- no timeout is supplied
- no debounce is used
- compilation and matching happen on the UI path

Match-result states:

- no valid pattern: localized `EnterValidPatternMessage`
- no sample text: localized `EnterSampleTextMessage`
- global match with zero matches: localized `RegexBreakdownNoMatchesFound`
- global match with matches: original sample text is emitted with each match
  shown as bold accent-colored text, followed by localized
  `RegexBreakdownFoundMatches(count)`
- single match mode with zero matches: localized `RegexBreakdownNoMatchFound`
- single match mode with a match: sample text is emitted with the first match
  shown as bold accent-colored text, followed by localized
  `RegexBreakdownFoundOneMatch`
- unexpected match error: localized `RegexBreakdownErrorMessage(error)`

Match highlighting:

- uses `SystemAccentColor` when available
- falls back to ARGB `0, 120, 215`
- match count uses green
- no-match messages use orange
- errors use red

Grexa replacement:

- Regex Builder must use the same regex engine and option semantics as Grexa
  search, or clearly label when the builder supports a different dialect.
- Add a timeout or cancellation guard for pathological regexes; Grex currently
  risks UI stalls from catastrophic backtracking.
- Debounce live evaluation enough to keep typing responsive.
- Preserve global vs first-match behavior and localized status messages.

## Regex Engine Parity

Grex uses .NET `System.Text.RegularExpressions.Regex`.

Exposed options:

- default .NET regex behavior
- ignore case
- multiline anchors

Unexposed options:

- singleline/dotall
- ignore pattern whitespace
- explicit capture
- culture invariant
- ECMAScript
- right-to-left
- compiled

Grexa implication:

- Rust `regex` does not support all .NET constructs, including backreferences
  and lookaround.
- If Grexa search uses Rust `regex`, Regex Builder should preview Rust-regex
  behavior and surface unsupported syntax clearly.
- If Grexa wants Grex/.NET-like Regex Builder compatibility, evaluate PCRE2,
  `fancy-regex`, or another engine and make search use the same semantics.

## Visual Breakdown

`UpdateBreakdown`:

- clears `BreakdownStackPanel`
- blank pattern shows localized `RegexBreakdownEnterPatternMessage` in gray
- validates the pattern with `new Regex(pattern)`
- invalid pattern shows localized `RegexBreakdownInvalidPatternMessage(error)`
  in red
- valid pattern is parsed by `ParseRegexBreakdown`

Each breakdown item displays:

- type text as `[type]`
- type in semi-bold accent/fallback blue
- content in `Consolas`
- optional description prefixed by ` - ` in gray
- row spacing 8 and vertical margin 4

Parser behavior:

- `[...]` becomes a character class using the next `]`
- `(...)` becomes a capturing group by matching raw parentheses
- `(?...)` becomes a non-capturing group; Grex treats all `(?...)` forms this
  way, including lookarounds and named groups
- `*`, `+`, `?`, and `{...}` become quantifiers
- `^` and `$` become anchors
- `\d`, `\D`, `\w`, `\W`, `\s`, `\S`, `\n`, `\t`, and `\r` get specific escape
  descriptions
- other two-character escapes get a generic escape-sequence description
- all other characters become literals

Known simplifications:

- character classes do not account for escaped `]`
- parenthesis matching does not ignore escaped parentheses or parentheses inside
  character classes
- lazy or possessive quantifier suffixes are not grouped with their base
  quantifier
- alternation, dot, word boundary, named captures, lookarounds, inline options,
  and backreferences are not specifically classified
- breakdown validation uses default regex options, not the selected check boxes,
  though those options do not affect syntax for the exposed settings

Grexa replacement:

- Preserve a readable visual breakdown, but do not copy the shallow parser as a
  false source of truth.
- Prefer an engine-backed parse tree or a documented lightweight explainer that
  clearly supports a limited subset.
- Add tests for classes, escaped brackets, nested groups, escaped parentheses,
  alternation, lazy quantifiers, lookarounds, named captures, and invalid
  patterns.

## Presets

Preset patterns:

```text
Email:  ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$
Phone:  ^(\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}$
Date:   ^\d{4}-\d{2}-\d{2}$
Digits: ^\d+$
URL:    ^https?://[^\s/$.?#].[^\s]*$
```

Apply behavior:

- if the pattern box is blank or whitespace, the preset is inserted without
  confirmation
- `_isUpdating` suppresses the `TextChanged` handler during insertion
- after insertion, regex, match results, and breakdown are updated explicitly
- if the pattern box has content, a confirmation dialog is shown
- dialog title: `RegexBreakdownOverwritePatternTitle`
- dialog message: `RegexBreakdownOverwritePatternMessage(presetName)`
- primary button: `ProceedButton`
- secondary button: `CancelButton`
- default button: secondary/cancel
- proceeding replaces the pattern and updates regex/results/breakdown

Grexa replacement:

- Preserve these five presets and overwrite confirmation behavior.
- Use native Kirigami dialogs.
- Add tests that inserting into an empty field is immediate and inserting over
  existing text asks for confirmation.

## Localization

Regex Builder strings use `LocalizationService.Instance` through local
`GetString` helpers.

Registered tooltips:

- sample text box
- regex pattern text box
- case-insensitive check box
- multiline check box
- global-match check box

`RefreshLocalization` manually updates:

- all text-block labels
- text-box placeholders
- preset button content
- check-box content
- dynamic match results
- dynamic breakdown rows
- layout

Fallback behavior:

- if a localized value is empty or equals the key, hard-coded English fallback
  text is used for static UI labels
- dynamic result and breakdown strings use localization service output directly

MainWindow behavior:

- `MainWindow.RefreshLocalization` updates navigation text and registered
  tooltips
- then `RefreshChildViews` calls `ReloadRegexBuilderView`
- `ReloadRegexBuilderView` clears `RegexBuilderContentGrid`, creates a new
  `RegexBuilderView`, assigns it to `RegexBuilderView`, and calls
  `RefreshLocalization`

Current consequence:

- a full application language refresh discards any transient Regex Builder
  sample text and pattern because MainWindow recreates the control.

Grexa replacement:

- Use Qt translation bindings or a model-driven refresh.
- Preserve localized tooltips and dynamic result/breakdown text.
- Refresh in place so changing language does not discard the current regex
  experiment.

## Theme Handling

Regex Builder subscribes to `MainWindow.ThemeChanged` while loaded and
unsubscribes when unloaded.

Custom high-contrast themes:

- Gentle Gecko
- Black Knight
- Diamond
- Dreams
- Paranoid
- Red Velvet
- Subspace
- Tiefling
- Vibes

Theme handling mirrors the Settings view:

- non-custom themes clear overrides
- custom themes clear local resources, walk the visual tree, set text/control
  foregrounds, set background, and populate local button/check-box/text-box
  resource keys
- check box visual states are forced through state transitions to apply colors

Grexa replacement:

- Use Qt/Kirigami palettes and KDE color roles.
- Do not port WinUI resource overrides or visual-tree brush walking.
- Keep Regex Builder readable under light, dark, system, and high-contrast
  palettes.

## Pointer Cursors

Buttons and check boxes handle pointer-enter and pointer-exit events.

Behavior:

- reflection is used to set WinUI `UIElement.ProtectedCursor`
- buttons and check boxes use hand cursor on hover
- cursor resets to arrow on exit

Grexa replacement:

- Drop WinUI reflection.
- Use native Qt cursor properties only where needed.

## Test Coverage

Existing Grex coverage is mostly localization-oriented:

- localization key lists for all Regex Builder messages, labels, placeholders,
  preset buttons, check boxes, and tooltips
- uniqueness and nonblank key assertions
- key naming convention checks
- fallback behavior for missing, empty, and null keys
- formatted localization calls for match count, errors, invalid patterns, and
  overwrite dialog messages
- culture switching for `en-US`, `de-DE`, `es-ES`, and `fr-FR`
- invalid culture fallback to `en-US`
- rapid culture switching sanity checks

Current test gaps:

- no direct tests for regex compilation options
- no direct tests for live match output
- no direct tests for global vs first-match mode
- no direct tests for preset overwrite behavior
- no direct tests for visual breakdown parsing
- no timeout/cancellation tests for expensive regexes

Grexa should add tests for all of the above, especially regex engine parity with
the actual search implementation.

## Current Grexa Status

Grexa now ships `RegexBuilderPage.qml` and
`apps/grexa-gui/src/qobjects/regex_builder.rs`. Live compilation and sample
match ranges use `grexa-core::PatternEngine`, so the preview follows the same
regex engine cascade as search. The page includes preset chips, sample text,
case-insensitive matching, match count, inline highlights, and apply-to-search
flow.

Remaining gaps:

- implement a visual breakdown/explainer if Grexa keeps that Grex surface
- add global/first-match and multiline controls if parity requires them
- add timeout/cancellation coverage for expensive regexes
- broaden tests for preset behavior and search/builder regex parity
- complete localization/accessibility coverage for all Regex Builder controls
