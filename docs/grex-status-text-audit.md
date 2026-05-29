# Grex StatusText Audit

This document records the behavior of the status strip in Grex's search
tabs — `TabViewModel.StatusText`, the resource keys it formats, the
elapsed-time pluralization rules, and the search-within filtered summary.
Grexa must either preserve these strings 1:1, reword them for Linux
semantics, or replace `string.Format` placeholders with ICU MessageFormat
so plural forms render correctly in non-English locales.

Source evidence:

- `ViewModels/TabViewModel.cs`
- `Controls/SearchTabContent.xaml`
- `Controls/SearchTabContent.xaml.cs`
- `MainWindow.xaml` and `MainWindow.xaml.cs`
- `Services/LocalizationService.cs`
- `Strings/en-US/Resources.resw`
- `Tests/ViewModels/TabViewModelTests.cs`
- `UITests/SearchUITests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`

## StatusText Property And Update Pipeline

`TabViewModel` exposes a single `StatusText` string that the rest of the UI
observes through `INotifyPropertyChanged`. The relevant private state:

```csharp
private string _statusText = string.Empty;
private string _statusResourceKey = "ReadyStatus";
private object[] _statusResourceArgs = Array.Empty<object>();

private bool _hasResultsSummary;
private string _resultsSummaryKey = "ReadyStatus";
private int _resultsSummaryTotalMatches;
private int _resultsSummaryTotalFiles;
private string _resultsSummaryElapsed = string.Empty;
```

Every assignment flows through `SetStatus(key, args)` which formats the
resource string and caches the key + args so `RefreshLocalization()` can
rebuild it on a runtime language switch. `SetResultsSummary` captures the
unfiltered totals; `UpdateResultsStatusForFilter()` re-renders as either
the unfiltered summary or `FilteredMatchesStatus` when the
search-within-results filter is non-empty.

The single consumer is `MainWindow.xaml`'s
`<InfoBar x:Name="StatusInfoBar" x:Uid="StatusInfoBar"/>`;
`MainWindow.SelectedTab_PropertyChanged` mirrors the active tab's
`StatusText` into `StatusInfoBar.Message`. Severity is bucketed in
`UpdateStatusInfoBar` by **string-prefix sniffing**:

```csharp
if (statusText.StartsWith("Error:", StringComparison.OrdinalIgnoreCase))
    StatusInfoBar.Severity = InfoBarSeverity.Error;
else if (statusText.StartsWith("Found ", ...) || statusText.StartsWith("Replaced ", ...))
{
    var match = Regex.Match(statusText, @"(?:Found|Replaced)\s+(\d+)\s+matches");
    ...
}
```

This is a latent localization bug — the classifier only recognizes English
prefixes and the literal `"matches"` token. Any localized status breaks the
bucketing and falls through to `Informational`. Grexa must replace this
with a typed severity field rather than parsing the rendered string.

## 1. Every Status String During Search

All resource lookups below quote `Strings/en-US/Resources.resw` verbatim.

### 1.1 Idle (initial, after Clear, after cancellation)

`SetStatus("ReadyStatus")` is called from the constructor, `ClearResults()`,
and the `OperationCanceledException` branch in both `PerformSearchAsync` and
`PerformReplaceAsync`.

```xml
<data name="ReadyStatus"><value>Ready</value></data>
```

### 1.2 Searching (in flight)

`SetStatus("SearchingStatus")` at the top of `PerformSearchAsync` after the
result lists are cleared. The trailing ellipsis is ASCII three-dot, not
U+2026 — Grexa should canonicalize to `…`.

```xml
<data name="SearchingStatus"><value>Searching...</value></data>
```

### 1.3 Replacing (in flight)

`SetStatus("ReplacingStatus")` at the top of `PerformReplaceAsync`.

```xml
<data name="ReplacingStatus"><value>Replacing...</value></data>
```

### 1.4 No Matches

Present in the catalog but **never assigned** by `TabViewModel`. A
zero-match search renders `FoundMatchesStatus` with `{0}=0` (e.g.
`"Found 0 matches in 0 files in 0.04 seconds"`).

```xml
<data name="NoMatchesStatus"><value>No matches found</value></data>
```

### 1.5 Search Completed (totals)

`SetResultsSummary("FoundMatchesStatus", totalMatches, fileCount,
FormatElapsedTime(_searchStopwatch.Elapsed))` at the end of both the
Files-mode and Content-mode branches of `PerformSearchAsync`.

```xml
<data name="FoundMatchesStatus"><value>Found {0} matches in {1} files in {2}</value></data>
```

Args: `{0}` = match count (int), `{1}` = file count (int),
`{2}` = pre-formatted elapsed string (see section 4).

### 1.6 Filtered Summary (search-within-results narrows results)

`UpdateResultsStatusForFilter()` calls
`SetStatus("FilteredMatchesStatus", filteredMatches, filteredFiles,
_resultsSummaryTotalMatches, _resultsSummaryTotalFiles,
_resultsSummaryElapsed)` whenever the filter text is non-empty and
`_hasResultsSummary` is true.

```xml
<data name="FilteredMatchesStatus"><value>Showing {0} matches in {1} files (filtered from {2} matches in {3} files) in {4}</value></data>
```

Args: `{0}` filtered matches, `{1}` filtered files, `{2}` original matches,
`{3}` original files, `{4}` original elapsed string. The elapsed time is
**not** recomputed; the filter is a synchronous in-memory pass that reuses
the underlying search's wall-clock.

### 1.7 Replace Completed (totals)

`SetResultsSummary("ReplacedMatchesStatus", totalMatches, fileCount,
FormatElapsedTime(...))` at the end of `PerformReplaceAsync`.

```xml
<data name="ReplacedMatchesStatus"><value>Replaced {0} matches in {1} files in {2}</value></data>
```

### 1.8 Error

`SetStatus("ErrorStatus", ex.Message)` in three places: the docker symlink
branch (its `{0}` arg is itself the localized `DockerSymlinkErrorMessage`),
the generic `catch (Exception ex)` block in `PerformSearchAsync`, and the
same block in `PerformReplaceAsync`.

```xml
<data name="ErrorStatus"><value>Error: {0}</value></data>
```

The code-behind also writes `ViewModel.StatusText = $"Error: {ex.Message}";`
in three places (`Controls/SearchTabContent.xaml.cs:1536, 2442, 2965`),
bypassing localization. Grexa must route these through localization.

### 1.9 InfoBar Frame Strings (static)

```xml
<data name="StatusInfoBar.Title"><value>Search Status</value></data>
<data name="StatusInfoBar.Message"><value>Ready</value></data>
```

The fallback `UpdateStatusInfoBar("Ready")` at `MainWindow.xaml.cs:736` is
a hard-coded English literal and should read from `ReadyStatus` instead.

## 2. Format Strings

All user-facing format strings in canonical form:

```text
ReadyStatus            = "Ready"
SearchingStatus        = "Searching..."
ReplacingStatus        = "Replacing..."
NoMatchesStatus        = "No matches found"        (unused at runtime)
ErrorStatus            = "Error: {0}"               {0} = exception message
FoundMatchesStatus     = "Found {0} matches in {1} files in {2}"
ReplacedMatchesStatus  = "Replaced {0} matches in {1} files in {2}"
FilteredMatchesStatus  = "Showing {0} matches in {1} files (filtered from {2} matches in {3} files) in {4}"
```

The elapsed slot has no numeric format specifier; `{2}`/`{4}` receives a
pre-formatted string from `FormatElapsedTime` (section 4). Decimal
precision lives in C# (`$"{totalSeconds:F2}"`), not in the resource.

## 3. Filtered-Summary Phrasing

The English summary when the in-tab "Search within results..." box narrows
results:

```text
Showing {0} matches in {1} files (filtered from {2} matches in {3} files) in {4}
```

Rendered: `Showing 12 matches in 3 files (filtered from 137 matches in 24 files) in 0.84 seconds`.

The filter is purely client-side (`ApplyResultsFilter` over
`_allSearchResults` / `_allFileSearchResults`) and supports literal and
regex modes via `_resultsFilterIsRegex` / `_resultsFilterRegex`. Tests
assert prefix equality with `string.Empty` in the elapsed slot:

```csharp
// UITests/SearchUITests.cs:186, IntegrationTests/SearchWorkflowTests.cs:92,
// Tests/ViewModels/TabViewModelTests.cs:354
tabViewModel.StatusText.Should().StartWith(
    L("FilteredMatchesStatus", filteredMatches, filteredFiles,
      totalMatches, totalFiles, string.Empty));
```

The contract is "everything up to the elapsed token must match"; the
elapsed string is non-deterministic across runs.

Edge cases:

- When the filter text is whitespace, `UpdateResultsStatusForFilter` falls
  back to `SetStatus(_resultsSummaryKey, ...)` — trimming to empty restores
  `FoundMatchesStatus` rather than displaying `Showing X of X`.
- After a successful search, both result-mode branches call
  `ApplyResultsFilter()`, so a filter typed before pressing Search renders
  `FilteredMatchesStatus` immediately on completion.

## 4. Elapsed-Time Formatting

`FormatElapsedTime(TimeSpan elapsed)` in `TabViewModel.cs:1928-1976` buckets
the duration into four ranges. Unit words:

```xml
<data name="TimeSecondSingular"><value>second</value></data>
<data name="TimeSecondPlural"  ><value>seconds</value></data>
<data name="TimeMinuteSingular"><value>minute</value></data>
<data name="TimeMinutePlural"  ><value>minutes</value></data>
<data name="TimeHourSingular"  ><value>hour</value></data>
<data name="TimeHourPlural"    ><value>hours</value></data>
```

### Subsecond and Up-To-30-Second Range

```csharp
// Less than 30 seconds: show with milliseconds
string secondUnit = totalSeconds == 1.0 ? Singular : Plural;
return $"{totalSeconds:F2} {secondUnit}";
```

Examples: `"0.04 seconds"`, `"12.43 seconds"`, `"29.99 seconds"`. The
singular branch is effectively unreachable — floating-point equality against
`1.0` makes every subsecond render plural. Decimal precision is hard-coded
`F2`.

### 30 To 59 Seconds

```csharp
int seconds = (int)Math.Round(totalSeconds);
string secondUnit = seconds == 1 ? Singular : Plural;
return $"{seconds} {secondUnit}";
```

Examples: `"30 seconds"`, `"45 seconds"`. Always plural in practice
(bucket starts at 30).

### 60 Seconds To 59 Minutes

```csharp
int minutes = (int)Math.Floor(totalSeconds / 60);
int seconds = (int)Math.Floor(totalSeconds % 60);
if (seconds == 0) return $"{minutes} {minuteUnit}";
return $"{minutes} {minuteUnit} {seconds} {secondUnit}";
```

Examples: `"1 minute"`, `"1 minute 1 second"`, `"1 minute 9 seconds"`,
`"5 minutes 30 seconds"`.

### 60 Minutes Or More

```csharp
int hours = (int)Math.Floor(totalMinutes / 60);
int minutes = (int)Math.Floor(totalMinutes % 60);
if (minutes == 0) return $"{hours} {hourUnit}";
return $"{hours} {hourUnit} {minutes} {minuteUnit}";
```

Examples: `"1 hour"`, `"1 hour 1 minute"`, `"1 hour 9 minutes"`,
`"3 hours 45 minutes"`. No seconds component once `totalMinutes >= 60`.

### Pluralization Failure Modes

- Only two forms per unit (`Singular` / `Plural`). Languages with
  `one`/`few`/`many` (Polish, Russian, Arabic, Czech, Lithuanian, Welsh)
  cannot be rendered correctly; the `n != 1` rule is English-class only.
- `0 seconds` uses the plural branch (correct for English) but Arabic
  treats zero as its own category.
- The subsecond branch's `totalSeconds == 1.0` check is floating-point
  equality and effectively dead code.
- `$"{hours} {hourUnit} {minutes} {minuteUnit}"` is a hand-rolled
  conjunction that drops the word for "and" — locales such as DE/FR
  expect "1 Stunde und 9 Minuten" / "1 heure et 9 minutes". The bare-space
  form is incidentally fine for RU but wrong elsewhere.
- Numbers are interpolated with no `IFormatProvider`; `string.Format` in
  `LocalizationService.GetLocalizedString` likewise omits a provider, so
  large match counts render as `1,234` in en-US, `1.234` in de-DE,
  `1 234` (NBSP) in fr-FR. The InfoBar severity regex
  `(?:Found|Replaced)\s+(\d+)\s+matches` then fails to match.

## 5. Localization Shape

| Resource key            | Args | Plural-aware? | Numbers          |
|-------------------------|------|---------------|------------------|
| `ReadyStatus`           | 0    | n/a           | n/a              |
| `SearchingStatus`       | 0    | n/a           | n/a              |
| `ReplacingStatus`       | 0    | n/a           | n/a              |
| `NoMatchesStatus`       | 0    | n/a           | n/a              |
| `ErrorStatus`           | 1    | no            | string `{0}`     |
| `FoundMatchesStatus`    | 3    | **no** — `matches`/`files` baked in | culture-dep int |
| `ReplacedMatchesStatus` | 3    | **no** — same                       | culture-dep int |
| `FilteredMatchesStatus` | 5    | **no** — four count slots baked in  | culture-dep int |
| `TimeSecondSingular`    | 0    | chosen at call site                 | n/a              |
| `TimeSecondPlural`      | 0    | chosen at call site                 | n/a              |
| `TimeMinuteSingular`    | 0    | chosen at call site                 | n/a              |
| `TimeMinutePlural`      | 0    | chosen at call site                 | n/a              |
| `TimeHourSingular`      | 0    | chosen at call site                 | n/a              |
| `TimeHourPlural`        | 0    | chosen at call site                 | n/a              |

Composite keys all use positional placeholders.

Key observations:

- The three composite strings (`Found/Replaced/Filtered`) are positional
  (`{0}..{4}`) but bake the English plural `"matches"` / `"files"`
  literally into the format. Translators cannot adjust the noun form to
  match the count.
- The `Time*` family is the only locus where Grex acknowledges plurality,
  and it's a binary singular/plural switch driven from C#.
- No status key carries `{0:N0}` or `{0:#,##0}` — number formatting is
  whatever `string.Format(format, intValue)` defaults to under the current
  thread culture.
- `RefreshLocalization` re-evaluates from the cached `_statusResourceKey`
  and `_statusResourceArgs`, which works for static args but **loses
  fidelity** for the elapsed slot — the cached arg is the already-formatted
  English string `"12.43 seconds"`, not the underlying `TimeSpan`.
  Switching language after a search leaves a stale English duration
  embedded in an otherwise translated summary.

## 6. Grexa Replacement Plan

### 6.1 Translate 1:1 (Fluent / Qt-ts catalog)

These keys carry no plural variation and can be ported verbatim:

```fluent
status-ready = Ready
status-searching = Searching…
status-replacing = Replacing…
status-no-matches = No matches found
status-error = Error: { $message }
status-infobar-title = Search Status
```

Notes: promote ASCII `...` to U+2026 (`…`) per GNOME HIG; `status-error`
should drop the literal `Error:` prefix so severity comes from a typed
view-model enum, not `StartsWith("Error:")` sniffing.

### 6.2 Reword For Linux Semantics

- `NoMatchesStatus` is unreachable today. Wire it in for `totalMatches == 0`
  and drop the redundant "Found 0 matches" path.
- `StatusInfoBar.Title = "Search Status"` is Windows-y; consider
  `Search results` to match GNOME / KDE result-pane wording.
- Replace the three `$"Error: {ex.Message}"` literals in
  `Controls/SearchTabContent.xaml.cs` with localized lookups.
- Drop the InfoBar severity regex `(?:Found|Replaced)\s+(\d+)\s+matches`;
  surface severity from `TabViewModel` as a `StatusSeverity` enum and bind.

### 6.3 ICU MessageFormat (recommended for Grexa)

The three composite summaries and the elapsed-time formatter need ICU
MessageFormat plural selectors to handle non-English locales. Recommended
keys and shapes:

```icu
status-found = {fileCount, plural,
    one {Found {matchCount, plural, one {# match} other {# matches}} in # file}
    other {Found {matchCount, plural, one {# match} other {# matches}} in # files}
  } in {elapsed}

status-replaced = {fileCount, plural,
    one {Replaced {matchCount, plural, one {# match} other {# matches}} in # file}
    other {Replaced {matchCount, plural, one {# match} other {# matches}} in # files}
  } in {elapsed}

status-filtered = Showing {filteredMatches, plural,
    one {# match} other {# matches}
  } in {filteredFiles, plural,
    one {# file} other {# files}
  } (filtered from {totalMatches, plural,
    one {# match} other {# matches}
  } in {totalFiles, plural,
    one {# file} other {# files}
  }) in {elapsed}

elapsed-subsecond = {seconds, number, ::.00} {seconds, plural,
    one {second} other {seconds}}

elapsed-seconds = {seconds, plural, one {# second} other {# seconds}}

elapsed-minutes-seconds = {minutes, plural, one {# minute} other {# minutes}} {seconds, plural, one {# second} other {# seconds}}

elapsed-minutes-only = {minutes, plural, one {# minute} other {# minutes}}

elapsed-hours-minutes = {hours, plural, one {# hour} other {# hours}} {minutes, plural, one {# minute} other {# minutes}}

elapsed-hours-only = {hours, plural, one {# hour} other {# hours}}
```

Properties of this shape:

- Named arguments so translators can reorder phrases; several target
  languages put the verb at the end.
- `one`/`other` selectors are the minimum; ICU routes `zero`/`few`/`many`
  if the catalog supplies them, so Polish/Russian/Arabic translators can
  add categories without touching C# code.
- `# match` / `# matches` are inlined so the noun scales with the count.
- `{seconds, number, ::.00}` moves decimal precision into the catalog.
  Pair with an ICU `NumberFormat` for the active locale so thousands
  separators use a consistent provider, not implicit
  `CultureInfo.CurrentCulture`.
- Pass the raw `TimeSpan` (or `(hours, minutes, seconds, totalSeconds)`
  tuple) into the formatter rather than a pre-formatted string, so a
  language switch via `RefreshLocalization` re-renders the duration.

### 6.4 Tests To Port

`Tests/ViewModels/TabViewModelTests.cs`, `UITests/SearchUITests.cs`, and
`IntegrationTests/SearchWorkflowTests.cs` all share a `L(key, args...)`
helper that round-trips through `LocalizationService` and asserts
`StartsWith` against the elapsed cutoff. Grexa's test harness should:

1. Replace the prefix assertion with a structural assertion against a
   `StatusSummary` record (`matches`, `files`, `filteredMatches`,
   `filteredFiles`, `elapsed`) exposed alongside the rendered string.
2. Keep one rendered-text smoke assertion per status state to exercise
   the ICU template end-to-end without coupling to English wording.
3. Add cases for `count` of 0, 1, 2, 5 (Russian `few`), and 11 (Russian
   `many`) once ICU plural selectors are in place.
