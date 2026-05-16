# Grex Windows Search Integration Audit

This document records Grex `Services/WindowsSearchIntegration.cs` behavior and
the Linux replacement contract for Grexa.

Source evidence:

- `Services/WindowsSearchIntegration.cs`
- `Services/SearchService.cs`
- `ViewModels/TabViewModel.cs`
- `Controls/SearchTabContent.xaml`
- `Controls/SettingsView.xaml`
- `Services/SettingsService.cs`
- `Tests/Services/SearchServiceTests.cs`
- `Tests/ViewModels/TabViewModelTests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `docs/usage.md`
- `docs/reference.md`
- `docs/features.md`

## Public Contract

`IWindowsSearchIntegration` exposes:

```csharp
Task<WindowsSearchQueryResult> QueryIndexedFilesAsync(
    string rootPath,
    string searchTerm,
    bool includeSubfolders);
```

`WindowsSearchQueryResult` contains:

- `ScopeAvailable`
- `Paths`

Factory behavior:

- `NotAvailable()` returns `ScopeAvailable = false` and an empty path list.
- `FromPaths(...)` removes null/blank paths and de-duplicates paths with
  case-insensitive comparison.

Grexa replacement:

- Define an index candidate provider contract with equivalent semantics:
  available/unavailable plus candidate paths.
- Treat candidate paths as input to the normal search pipeline, never as final
  results.

## Eligibility

The Windows integration itself returns unavailable when:

- the process is not running on Windows
- root path is blank
- search term is blank
- root path does not exist
- root path is not a supported Windows drive path
- OLE DB integration fails with `OleDbException`, `InvalidOperationException`,
  or `DllNotFoundException`

Supported Windows path check is intentionally narrow:

- path length at least 2
- second character is `:`

`SearchService` adds another gate before it calls the provider:

- user preference must request index use
- search must not be regex
- OS must be Windows

`TabViewModel` gates the UI option:

- disabled for regex search
- disabled when Docker mode is active
- disabled for WSL/Unix-style paths
- enabled only for drive-letter paths where first character is a letter and the
  second is `:`
- when eligibility becomes false, `UseWindowsSearchIndex` is automatically reset
  to false

Grexa replacement:

- Use a setting such as `Use KDE file index` or `Use file index`.
- Enable it only when the active target is a local Linux filesystem path and the
  selected search mode is plain text.
- Disable it for regex searches, container targets, unavailable index services,
  non-indexed paths, and abstract KIO URLs that cannot be opened as normal
  files.
- Clear the active toggle when eligibility becomes false.

## Query Behavior

Grex uses the Windows Search OLE DB provider:

```text
Provider=Search.CollatorDSO.2;Extended Properties='Application=Grex';
```

Scope construction:

- root path is expanded with `Path.GetFullPath`
- trailing separator is normalized
- a trailing backslash is added
- backslashes are escaped
- the final scope is `file:<escaped-path>`

Before querying for matches, Grex checks that the scope has at least one indexed
entry:

```sql
SELECT TOP 1 System.ItemPathDisplay
FROM SYSTEMINDEX
WHERE SCOPE='<scope>'
```

If the scope has no entries, the result is `NotAvailable`.

Candidate query:

```sql
SELECT System.ItemPathDisplay
FROM SYSTEMINDEX
WHERE SCOPE='<scope>'
AND CONTAINS('"searchTerm"')
```

Escaping:

- single quotes in scope are doubled
- search term is trimmed
- single quotes in search term are doubled
- double quotes in search term are doubled

Candidate rows are included only when the returned path is not blank and
`File.Exists(candidate)` is true.

When `includeSubfolders` is false, candidate paths are filtered to files whose
parent directory exactly equals the normalized root directory.

Grexa replacement:

- Baloo queries should be a short spike with a keep/defer/drop decision.
- If kept, Baloo must be treated as a candidate provider only.
- Every candidate must still pass Grexa's own filters and content matcher before
  appearing in results.
- Candidate paths must be normal filesystem paths that can be read by Grexa.
- If Baloo is disabled, unavailable, stale, unsupported for the path, or throws,
  Grexa must fall back to native walking.

## SearchService Integration

When the provider returns available candidates, `SearchService` uses those paths
as the file enumeration source and then continues through the normal pipeline:

- hidden filtering
- system/dependency filtering
- binary filtering
- symbolic link filtering
- `.gitignore`
- match-file patterns
- exclude-dir patterns
- size limits
- file content reading and matching

When the provider reports unavailable or throws, `SearchService` falls back to
`Directory.EnumerateFiles`.

Tests verify:

- indexed candidates limit the searched file set
- unavailable index falls back to full scan
- view model passes the preference flag to `SearchService`
- regex and non-Windows paths disable the UI flag
- integration workflow uses returned indexed candidates

Grexa replacement:

- The index provider must integrate before file filtering, matching Grex.
- The fallback walker must preserve identical search results when the index is
  not used.
- Diagnostics should state whether a search used the index or the walker.

## Settings And UI Text

Grex setting:

- `UseWindowsSearchIndex`, default false

Grex Search tab control:

- `UseWindowsSearchCheckBox`
- content: `Use Windows Search`
- tooltip: use the Windows Search index for faster text searches on indexed
  Windows folders; regex, WSL, and non-indexed locations use the traditional
  scanner.

Grex Settings control:

- `DefaultUseWindowsSearchCheckBox`
- content: `Use Windows Search`
- tooltip: use the Windows Search index for faster text searches on indexed
  folders.

Grexa replacement:

- Rename setting to a Linux-neutral name, for example `use_file_index`.
- User text should avoid promising completeness. Prefer wording such as `Use KDE
  file index` or `Use file index for candidates`.
- Imported Grex settings can map `UseWindowsSearchIndex` to the new setting only
  when a Linux index backend is implemented; otherwise ignore it with migration
  notes.

## Current Grexa Gaps Against This Audit

As of this audit:

- No index candidate provider trait exists.
- Baloo availability/path-indexed detection is not implemented.
- Search does not report whether it used an index.
- Settings include `use_file_index`, but there is no UI or backend behavior.
- Tests for index candidate fallback and verification are not implemented.

These gaps should remain open in `PLAN.md` until implementation and tests cover
them.
