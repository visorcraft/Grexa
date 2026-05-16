# Grex SearchService Audit

This document records the behavior of Grex `Services/SearchService.cs` that
Grexa must either preserve, replace with a Linux-native equivalent, or document
as intentionally non-applicable.

Source evidence:

- `Services/SearchService.cs`
- `Models/SearchResult.cs`
- `Models/FileSearchResult.cs`
- `Models/SizeLimitType.cs`
- `Models/SizeUnit.cs`
- `Models/StringComparisonMode.cs`
- `Models/UnicodeNormalizationMode.cs`
- `Tests/Services/SearchServiceTests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
- `Tests/ViewModels/TabViewModelTests.cs`

## Public Search Contract

`SearchAsync` returns `List<SearchResult>` and accepts:

- `path` and `searchTerm`
- text or regex mode through `isRegex`
- filters for `.gitignore`, case sensitivity, system files, subfolders, hidden
  items, searchable binary/document files, symbolic links, file size, file name
  patterns, and excluded directories
- optional Windows Search candidate seeding
- text comparison controls: `StringComparisonMode`,
  `UnicodeNormalizationMode`, `diacriticSensitive`, and optional selected
  culture
- cancellation token

Empty or whitespace `path` or `searchTerm` returns an empty result list. A
non-empty local path that is not an existing directory throws
`DirectoryNotFoundException`. Invalid regex patterns throw `ArgumentException`
before any file scanning begins.

Search dispatch is path-based:

- WSL paths are delegated to `SearchWslPathAsync`.
- All other paths use local Windows enumeration in `SearchWindowsPathAsync`.

For Grexa, WSL and Windows path branches are non-applicable, but their user
visible search behavior still identifies parity requirements for native Linux
paths.

## WSL Path Detection And Conversion

`IsWslPath` returns true for:

- `\\wsl$\...`
- `\\wsl.localhost\...`
- `/mnt/...`
- `\mnt\...`
- any path beginning with `/` unless the second character is `:`

It returns false for empty strings, null, bare `C`, and Windows drive paths such
as `C:\Users\Test`.

WSL UNC paths are converted by stripping the `\\wsl$\<distro>` or
`\\wsl.localhost\<distro>` prefix and changing separators to `/`. Windows drive
paths can be converted to `/mnt/<drive>/...` by the helper, although
`SearchAsync` only sends paths already classified as WSL to that branch.

The distribution name is extracted from UNC paths and passed as `wsl -d
<distro>`.

Grexa replacement:

- Native Linux paths are the default.
- No WSL detection or `wsl.exe` delegation should remain.
- Windows paths from imported Grex data require migration behavior, not runtime
  search behavior.

## Local File Enumeration

Local search uses `Directory.EnumerateFiles(path, "*", EnumerationOptions)` with:

- `RecurseSubdirectories = includeSubfolders`
- `IgnoreInaccessible = true`
- `AttributesToSkip = None`, plus `System` when system files are excluded, plus
  `Hidden` when hidden items are excluded

Files are processed with maximum parallelism of 8. File read errors, encoding
errors, unsupported paths, and most per-file exceptions are swallowed and the
file is skipped. Cancellation is checked before dispatch, during parallel file
processing, while adding results, and after processing.

Results are sorted by `FileName`, then `LineNumber` before return.

Grexa parity requirements:

- Keep inaccessible/failed files non-fatal unless the root itself is invalid.
- Keep cancellation checkpoints throughout traversal and file scanning.
- Preserve stable returned ordering or document a different UI/model sort policy.
- Add progress counters separately; Grex `SearchService` does not expose them
  directly.

## Windows Search Candidate Seeding

Windows Search is used only when:

- `preferWindowsSearchIndex` is true
- the query is not regex
- the process is running on Windows

If Windows Search returns an available scope, its paths become the candidate
file list. If the scope is unavailable or query throws, Grex falls back to full
directory enumeration. Indexed candidates are still searched by Grex before they
are returned. If `includeSubfolders` is false, indexed candidates are filtered
back to files directly under the root.

Grexa replacement:

- Optional Baloo candidate seeding should follow the same rule: candidate source
  only, never source of truth.
- Regex searches should not use index seeding unless explicitly proven safe.
- Failed or unavailable index queries should fall back to native walking.

## Filter Order

Local search applies filters in this order:

1. Windows Search candidate seeding, when enabled.
2. Indexed candidate root-only adjustment when subfolders are disabled.
3. Hidden file and hidden directory filtering by file attribute and dot-name
   semantics.
4. System/dependency directory filtering.
5. Binary/searchable binary filtering.
6. Symbolic link filtering.
7. `.gitignore` filtering.
8. File name pattern filtering.
9. Excluded directory filtering.
10. Size limit filtering.
11. File content search.

Replace uses nearly the same filters, but `.gitignore`, match-file, and
exclude-dir filters are applied before binary and symlink filters. Grexa should
prefer one shared filtering pipeline unless there is a documented compatibility
reason to preserve this difference.

## Hidden And System Filters

When hidden items are excluded, Grex removes:

- files with the Windows hidden attribute
- dotfiles such as `.env`
- files under any dot-directory

When system files are excluded, local search removes files under these path
components:

- `.git`
- `vendor`
- `node_modules`
- `storage/framework`

Replace additionally excludes `bin` and `obj`; WSL search excludes `.git`,
`vendor`, `node_modules`, `storage/framework`, `bin`, `obj`, and Linux pseudo
paths `sys`, `proc`, and `dev`.

Grexa parity requirements:

- Linux system-path filtering should include `.git`, `vendor`, `node_modules`,
  `storage/framework`, `bin`, `obj`, `sys`, `proc`, and `dev`.
- Root searches also need Linux pseudo-filesystem guards for `/proc`, `/sys`,
  `/dev`, and related runtime mounts.

## Binary And Searchable Document Filters

Grex treats these extensions as binary by default:

`exe`, `dll`, `obj`, `bin`, `zip`, `tar`, `gz`, `7z`, `rar`, `png`, `jpg`,
`jpeg`, `gif`, `bmp`, `ico`, `svg`, `webp`, `mp3`, `mp4`, `avi`, `mkv`, `wav`,
`flac`, `ogg`, `pdf`, `doc`, `docx`, `xls`, `xlsx`, `ppt`, `pptx`, `pdb`,
`cache`, `lock`, `pack`, `idx`, `rtf`.

When `includeBinaryFiles` is false, all binary extensions are skipped.

When `includeBinaryFiles` is true, Grex includes normal text files plus only
these searchable binary/document extensions:

- `docx`, `xlsx`, `pptx`
- `odt`, `ods`, `odp`
- `zip`
- `pdf`
- `rtf`

Grex document extraction behavior:

- ZIP-based formats search entries ending in `.xml`, `.txt`, or `.rels` as
  UTF-8. Result file names and relative paths include `[entry name]`.
- PDF support is best-effort: read bytes as UTF-8, extract text-like content
  between `stream` and `endstream`, and also try `/Type /Text` object text.
- RTF support reads UTF-8 and strips a simple subset of RTF control words.
- Unsupported or unreadable document entries are skipped.

Grexa parity requirements:

- Preserve the same binary skip list unless replaced by a documented better
  classifier.
- Implement searchable document extraction for the listed formats before
  claiming document-search parity.
- Keep unreadable or malformed searchable binaries non-fatal.

## `.gitignore` Handling

Local search delegates `.gitignore` decisions to `GitIgnoreService.ShouldIgnoreFile`.
Search only applies it when `respectGitignore` is true.

WSL search attempts to use `git grep` when the root contains `.git`; otherwise
it uses `find` and filters output through `git check-ignore`.

Grexa should preserve Grex's `.gitignore` semantics through golden tests,
especially root-relative patterns, directory-only patterns, negations, `**`,
bracket patterns, and case behavior. The Rust `ignore` crate is a good baseline,
but Grex compatibility must be verified rather than assumed.

## Match File Name Patterns

`matchFileNames` is a `|`-separated pattern string. Each item is trimmed.

- Patterns beginning with `-` are exclusions.
- Non-prefixed patterns are inclusions.
- Exclusions are checked first.
- If one or more include patterns exist, at least one include pattern must
  match.
- If only exclusions exist, files not matching an exclusion are included.
- `*` and `?` wildcard patterns are converted to case-insensitive regex.
- Invalid wildcard regex falls back to case-insensitive exact file-name compare.

Grex tests cover `*.json`, `*.json|*.txt`, and `*.json|-*.txt`.

Plan note: `PLAN.md` calls for `|` or `;` separators. Grex currently documents
and tests `|` in `SearchService`; Grexa can support both for migration quality,
but should record this as an intentional superset.

## Excluded Directory Patterns

`excludeDirs` is interpreted as regex when it starts with `^` or contains `(`,
`[`, or `$`. Regex matching is case-insensitive and checks every relative path
component plus the full relative directory path.

If not regex, `excludeDirs` is split by comma only. Each name is trimmed, and
matching is case-insensitive against any relative path component. Semicolon
support is not present in Grex `SearchService`, even though the Grexa plan asks
for comma/semicolon compatibility.

WSL post-filtering uses the same regex detection but also treats comma-separated
directory names only.

Grexa should support comma and semicolon as an intentional compatibility
superset, but should retain the Grex regex detection and component/full-path
matching behavior.

## Size Limits

Size limits are skipped when `SizeLimitType.NoLimit` or no size value is
provided.

Grex converts the user value to bytes for local search:

- KB: `value * 1024`
- MB: `value * 1024 * 1024`
- GB: `value * 1024 * 1024 * 1024`

Tolerance:

- KB: 10 KB
- MB: 1 MB
- GB: 25 MB

Comparisons:

- LessThan: `fileSize < limit + tolerance`
- EqualTo: `abs(fileSize - limit) <= tolerance`
- GreaterThan: `fileSize > limit - tolerance`

WSL search converts to KB and uses `find -size` with the same intended
tolerance for search. WSL replace currently uses exact `find -size` predicates
without tolerance; this is a Grex inconsistency that should not be copied unless
explicitly required.

## Text Matching

Regex mode:

- Compiles one .NET `Regex` per search with `RegexOptions.Compiled`.
- Adds `RegexOptions.IgnoreCase` when search is not case-sensitive.
- Throws `ArgumentException` for invalid patterns.
- Regex mode ignores culture, Unicode normalization, and diacritic settings.
- Lines causing regex exceptions are skipped.

Plain text mode:

- Applies Unicode normalization to both text and search term when requested.
- If diacritic insensitive, decomposes to FormD, drops non-spacing marks, and
  recomposes to FormC.
- If `StringComparisonMode.CurrentCulture` and a valid explicit culture is
  supplied, Grex uses that culture's `CompareInfo.IndexOf` with `IgnoreCase`
  when needed.
- Invalid explicit cultures fall back to normal comparison handling.
- Otherwise comparisons use `StringComparison.Ordinal`,
  `CurrentCulture`, or `InvariantCulture`, with case-sensitive or
  case-insensitive variants.

Important limitation:

- `CalculateColumnNumber`, `CountMatchesOnLine`, and
  `BuildMatchPreviewSegments` use ordinal/ordinal-ignore-case matching for text
  previews and counts. They do not apply Unicode normalization, selected
  culture, or diacritic-insensitive transformations. Grexa should decide whether
  to preserve this exact behavior or fix it consistently and document the
  difference.

## Result Shape

Each matching line produces one `SearchResult`:

- `FileName`: file name, or `file [entry]` for ZIP-based entries
- `LineNumber`: 1-based
- `ColumnNumber`: 1-based first match index, or 1 if not found by the preview
  helper
- `LineContent`: sanitized display line, truncated to 500 characters plus `...`
- `MatchPreviewBefore`, `MatchPreviewMatch`, `MatchPreviewAfter`: centered
  around the first match, maximum 400 characters total before sanitization
- `FullPath`: source file path
- `RelativePath`: `Path.GetRelativePath(root, file)` where possible
- `MatchCount`: number of matches on that line, at least 1 for matched text
  lines

Preview behavior:

- If the match is longer than 400 characters, preview starts at match index.
- Otherwise preview is centered around the first match where possible.
- Leading whitespace is stripped from the preview snippet.
- `...` is added to before/after segments when content was clipped.
- Sanitization removes replacement characters, newlines/carriage returns,
  non-tab control characters, surrogate characters, and unassigned code points.

`SearchResult.DirectoryPath` derives the directory portion of `RelativePath`,
normalizing separators to `/`.

## Files Mode Aggregation

`SearchService.SearchAsync` returns line-level results. Files mode aggregation is
performed outside `SearchService` in ViewModel code, but the shape expected by
the rest of the app is `FileSearchResult`:

- file name
- size
- total match count
- first match line number
- first match preview segments
- preview match list
- full and relative paths
- extension
- encoding
- modified timestamp

Grexa's core currently aggregates content results into file results directly,
which is acceptable if the resulting fields stay compatible with the Grex UI
contract.

## Replace Contract

`ReplaceAsync` returns `List<FileSearchResult>`.

Empty or whitespace `path` or `searchTerm` returns an empty list. Non-empty local
missing root throws `DirectoryNotFoundException`. Invalid regex patterns throw
`ArgumentException`.

Local replace:

- Reuses most search filters.
- Detects the file encoding and reads the entire file into memory.
- Regex replace uses compiled regex and `Regex.Replace`.
- Text replace counts matches with `IndexOf(searchTerm, comparison)` and writes
  with `content.Replace(searchTerm, replaceWith, comparison)`.
- Text replace does not apply Unicode normalization or diacritic-insensitive
  transformations even though those parameters exist.
- Files with no matches are skipped.
- Modified files are written back using detected encoding.
- Results include file name, updated size, match count, full and relative path,
  extension, encoding name, and last write time.
- Per-file read/write errors are swallowed.

WSL replace:

- Uses `find` plus `grep -l` to find matching files.
- Counts matches with `grep -o ... | wc -l`.
- Replaces with `sed -i`, using `-E` for regex and `g`/`gi` flags for global
  case-sensitive or insensitive fixed-string replacement.
- Collects file metadata with `stat`.
- Applies match-file and exclude-dir post-filters.

Grexa parity requirements:

- Local Linux replace should preserve the same safety workflow at the UI level,
  but implementation should avoid WSL/sed-specific quoting hazards.
- Replace should be cancellable and should report partial/cancelled policy.
- Decide whether to intentionally fix normalization/diacritic replace mismatch.

## Existing Test Coverage

`Tests/Services/SearchServiceTests.cs` covers:

- basic text search
- empty path/search term
- regex search and invalid regex
- case-sensitive and case-insensitive text search
- missing directories
- WSL path detection
- binary and searchable RTF inclusion behavior
- hidden file filtering
- subfolder filtering
- `.gitignore`
- special characters in paths
- replace basics and multiple replacements
- match-file patterns
- exclude-dir name and regex filtering
- size tolerances for KB and MB
- Windows Search candidate use and fallback

`IntegrationTests/SearchWorkflowTests.cs` adds workflow-level coverage for tab
searches, gitignore scenarios, filters, binary exclusion, search within results,
and culture-related UI behavior.

`Tests/ViewModels/TabViewModelTests.cs` verifies that view-model search and
replace calls pass through the relevant SearchService parameters.

## Current Grexa Gaps Against This Audit

As of this audit:

- Grexa implements native walking, hidden filtering, system directory filtering,
  symlink selection, match-file filtering, exclude-dir filtering, size limits,
  text search, regex search, result fields, and file aggregation.
- Grexa does not yet implement Grex-compatible `.gitignore` golden tests.
- Grexa does not yet implement document extraction for ZIP/Office/OpenDocument,
  PDF, or RTF.
- Grexa does not yet implement selected-culture text matching or an ICU strategy.
- Grexa's result ordering and column/preview behavior need comparison fixtures.
- Grexa does not yet implement progress events or cancellation policy.
- Grexa does not yet implement replace.
- Grexa does not yet implement Baloo candidate seeding.
- Grexa CLI and core expose only a subset of the advanced comparison settings
  through command-line options.

These gaps should remain open in `PLAN.md` until covered by implementation and
tests.
