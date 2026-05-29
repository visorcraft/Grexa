# Grex GitIgnoreService Audit

This document records the behavior of Grex `Services/GitIgnoreService.cs`
that Grexa must either preserve through the `ignore` crate, replace with a
Linux-native equivalent, or document as intentionally non-applicable. The
Grex implementation hand-rolls gitignore semantics on top of regex; Grexa
delegates to BurntSushi's `ignore` crate via `WalkBuilder`, so this audit
also enumerates the gaps where the two implementations diverge.

Source evidence:

- `Services/GitIgnoreService.cs`
- `Tests/Services/GitIgnoreServiceTests.cs`
- `IntegrationTests/SearchWorkflowTests.cs`
  (`SearchWorkflow_WithGitIgnore_RespectsIgnoreRules`,
  `SearchWorkflow_WithRootRelativeGitIgnorePattern_OnlyMatchesFromRoot`)
- `Tests/Services/SearchServiceTests.cs`
  (`SearchAsync_WithGitIgnoreEnabled_RespectsGitIgnoreRules`)
- `Services/SearchService.cs` (line 341, 1286, 2064 — gitignore integration
  call sites)
- `crates/grexa-core/src/search.rs` lines 158-167 (WalkBuilder wiring)
- `crates/grexa-core/Cargo.toml` declares `ignore = "0.4"` indirectly via
  the workspace `Cargo.toml`

## Public Contract

`GitIgnoreService.ShouldIgnoreFile(filePath, rootPath)` returns `bool`:

- `false` when either argument is null, empty, or whitespace.
- `false` when the absolute `filePath` does not begin with the absolute
  `rootPath` (case-insensitive prefix check).
- `false` when there is no `.gitignore` under `rootPath` or any ancestor of
  the file inside the root.
- Otherwise the result of the last rule that matched, walking from root
  `.gitignore` down to the file's parent `.gitignore`, with negation
  toggling the inherited verdict.

The service caches parsed rules per `.gitignore` path in a private
dictionary keyed by absolute path. Cache invalidation is not implemented;
the service is intended to live for the duration of a single search.

There is no per-directory ignore beyond `.gitignore`. Grex does not honor
`.git/info/exclude`, `core.excludesFile`, `.ignore`, or `.gitignore_global`.

For Grexa, the public contract is satisfied implicitly by
`WalkBuilder::git_ignore(true).git_exclude(true).git_global(true).ignore(true)`
in `search.rs`. The crate returns a filtered iterator rather than a
per-file boolean, so Grexa never needs to expose a `should_ignore_file`
function — but if the GUI ever needs to render an ignored-but-revealed
diagnostic, a wrapper around `ignore::gitignore::GitignoreBuilder` is the
intended fit.

## Path Normalization And Anchoring

Both `filePath` and `rootPath` are passed through `Path.GetFullPath` and
then `Replace('\\', '/')`. The relative path used for matching is
`filePath` minus the `rootPath` prefix, with any leading `/` trimmed.

Implication: Grex treats `D:\Repos\proj\src\a.cs` against root
`D:\Repos\proj` as the relative path `src/a.cs`. Forward slashes are the
canonical internal representation regardless of Windows or WSL origin.

Grexa's `ignore` crate operates on `Path` objects, internally
normalizing separators per platform. On Linux there is nothing to
normalize, so parity is automatic.

## Rule Parsing

`GetGitIgnoreRules` reads the `.gitignore` file with `File.ReadAllLines`
and constructs one `GitIgnoreRule` per non-comment, non-blank line:

- Lines are `Trim()`ed, then empty lines are skipped.
- Lines starting with `#` (after trim) are skipped.
- Leading `!` marks the rule as a negation; the `!` is stripped, the
  remainder is `Trim()`ed.
- Trailing `/` flags `IsDirectoryOnly`.
- Leading `/` flags `IsRootRelative`.
- `GitIgnoreDirectory` is the directory that owns the `.gitignore`,
  enabling per-file relativization for nested ignore files.

There is no support for:

- Lines escaped with leading backslash (`\#literal` → a literal `#file`).
- Trailing-whitespace escaping with `\ ` to keep a space in the pattern.
- Mid-pattern escape sequences such as `\[` to escape a bracket character
  class opening (the bracket parser in `ConvertGitIgnorePatternToRegex`
  always treats `[` as the start of a character class).

These omissions are deviations from canonical git semantics. Grexa, via
the `ignore` crate, supports all three escape forms.

## Rule Storage Model

```
class GitIgnoreRule
{
    string Pattern;             // post-strip pattern body
    bool   IsNegation;          // had leading !
    bool   IsDirectoryOnly;     // had trailing /
    bool   IsRootRelative;      // had leading /
    string GitIgnoreDirectory;  // owning .gitignore folder
}
```

The pattern body retains its leading `/` and trailing `/`; the booleans
are derived from but redundant with the raw text. Consumers strip the
slashes at match time (`pattern.TrimEnd('/')`, conditional substring on
`'/'`).

## Rule Resolution Order

`ShouldIgnoreFile` evaluates rules in two passes:

1. Apply the root `.gitignore` (if present) to the entire relative path.
2. Walk each intermediate directory from root toward the file's parent.
   Each `.gitignore` is evaluated against the path *relative to that
   `.gitignore`'s directory*.

Within a single `.gitignore`, `CheckRules` iterates rules in source order
and records the last decision in a nullable `bool`. The final boolean is
read out as `shouldIgnore = result.Value` and then potentially overwritten
by deeper `.gitignore` files. This matches canonical git: more deeply
nested rules win, and within a file later rules win.

Grexa parity: the `ignore` crate evaluates per-directory `.gitignore`
files in the same precedence order. Negation also follows the same
"last matching rule wins" semantics.

## Matching Algorithm

`MatchesPattern` builds a regex via `ConvertGitIgnorePatternToRegex` and
tests it against three subjects in this order:

1. The full relative path (always).
2. The bare filename (only when the rule is not root-relative).
3. Each individual path segment (only when the rule is not
   root-relative).

If regex compilation throws, the fallback is `SimpleMatch`, which uses
substring/equality logic. The fallback is only ever reached when a
malformed bracket class survives the pattern translator.

`ConvertGitIgnorePatternToRegex` walks the pattern once and produces:

- `[abc]` → preserved as a regex character class.
  Inside brackets, `\`, `^`, and `-` are backslash-escaped.
- `**` → `.*` (matches across `/`).
- `*` → `[^/]*` (single-segment glob).
- `?` → `[^/]` (single character, never `/`).
- Other characters → `Regex.Escape`.

The pattern is then anchored:

- Patterns starting with `**/` (translated to `.*/`) become
  `^(<tail>$|.*/<tail>$)` — match at root or anywhere below.
- Patterns starting with `/` become `^<rest>$` — strict root anchor.
- Patterns with no wildcards become `(^<lit>$|/<lit>$)` — match the
  whole relative path or any path ending with that literal at a path
  boundary. This is what stops `.env` from matching `.env.docker`.
- Patterns with wildcards but no leading slash become `.*<body>$` —
  trailing anchor only.

Matching is case-insensitive (`RegexOptions.IgnoreCase`). On Windows and
WSL paths this is the safer default; on Linux it diverges from canonical
git, which is case-sensitive on case-sensitive filesystems.

## Directory-Only Pattern Handling

When `IsDirectoryOnly` is true, `CheckRules` short-circuits the regex
path entirely:

```
relativePath.Contains(pattern + "/")     ||
relativePath.StartsWith(pattern + "/")   ||
(relativePath.Equals(pattern) && isDirectory)
```

For root-relative directory-only rules, the leading `/` is stripped before
the check. The `isDirectory` argument is always `false` at the public
entry point because `ShouldIgnoreFile` only handles file paths; the third
clause is therefore dead in practice and only present for future use.

Implication: `build/` matches any file with `build/` anywhere in its
relative path. `/storage/app/` only matches files whose relative path
starts with `storage/app/`.

The `ignore` crate handles directory-only patterns through `WalkBuilder`'s
directory entry callback; the directory is excluded as a whole and its
descendants are never visited. This is faster than Grex's per-file check
but produces the same final filtered set.

## Cache Behavior

`_gitignoreCache` is a `Dictionary<string, List<GitIgnoreRule>>` keyed by
the absolute path of each `.gitignore` file. Once a `.gitignore` has been
parsed, subsequent calls reuse the rules. There is no invalidation by
file mtime, file size, or content hash. Modifying a `.gitignore` during a
running search yields stale rules.

Grexa equivalent: `Gitignore` builders inside the `ignore` crate read
their files once when `WalkBuilder::build()` is called. The walker is
re-created per search in `search.rs:158`, so each search picks up fresh
ignore files automatically.

## Edge Cases Asserted By Tests

| # | Behavior | Test method |
|---|----------|-------------|
| 1 | No `.gitignore` present ⇒ no file is ignored | `ShouldIgnoreFile_WithNoGitIgnoreFile_ReturnsFalse` |
| 2 | Plain extension glob `*.log` ⇒ `.log` files ignored, others kept | `ShouldIgnoreFile_WithGitIgnoreFile_ReturnsExpectedResult` |
| 3 | Multi-pattern file with `*.log`, `build/`, `.DS_Store` mixed | same test |
| 4 | Negation `!important.log` overrides earlier `*.log` | `ShouldIgnoreFile_WithNegationPattern_ReturnsExpectedResult` |
| 5 | Directory-only `build/` ignores everything beneath `build/` | `ShouldIgnoreFile_WithDirectoryPattern_ReturnsExpectedResult` |
| 6 | Nested `.gitignore` with `!subdir.txt` overrides outer `*.txt` | `ShouldIgnoreFile_WithNestedGitIgnore_ReturnsExpectedResult` |
| 7 | Null/empty arguments are non-fatal and return `false` | `ShouldIgnoreFile_WithEmptyOrNullParameters_ReturnsFalse` |
| 8 | Wildcard prefix `test*.txt` combined with `!test_backup.txt` | `ShouldIgnoreFile_WithComplexWildcardPatterns_ReturnsExpectedResult` |
| 9 | Double-asterisk `**/test.txt` matches at root and in subdirs | `ShouldIgnoreFile_WithDoubleAsteriskPattern_ReturnsExpectedResult` |
| 10 | Single-character `?` glob matches exactly one non-slash char | `ShouldIgnoreFile_WithQuestionMarkPattern_ReturnsExpectedResult` |
| 11 | Character class `test[12].txt` matches `test1.txt`/`test2.txt` | `ShouldIgnoreFile_WithBracketPatterns_ReturnsExpectedResult` |
| 12 | Empty `.gitignore` file ⇒ no file is ignored | `ShouldIgnoreFile_WithEmptyGitIgnoreFile_ReturnsFalse` |
| 13 | Comments-only `.gitignore` ⇒ no file is ignored | `ShouldIgnoreFile_WithCommentsOnlyInGitIgnoreFile_ReturnsFalse` |
| 14 | Malformed pattern (`[invalid`) does not throw | `ShouldIgnoreFile_WithMalformedPatterns_HandlesGracefully` |
| 15 | Caching is stable: repeated calls return identical results | `ShouldIgnoreFile_WithCachingBehavior_ReturnsConsistentResults` |
| 16 | Paths outside `rootPath` always return `false` | `ShouldIgnoreFile_WithAbsolutePath_ReturnsFalse` |
| 17 | Root-relative `/storage/app/` does NOT match `/app` segment | `ShouldIgnoreFile_WithRootRelativePattern_OnlyMatchesFromRoot` |
| 18 | Root-relative directory pattern matches files in subdirectories | `ShouldIgnoreFile_WithRootRelativeDirectoryPattern_MatchesFilesInsideDirectory` |
| 19 | Root-relative without trailing slash does not leak to non-root segments | `ShouldIgnoreFile_WithRootRelativePattern_DoesNotMatchSegmentInPath` |
| 20 | Mixed root-relative and non-root-relative `app/` patterns coexist | `ShouldIgnoreFile_WithRootRelativePatternAndNonRootRelativePattern_HandlesBothCorrectly` |
| 21 | `SearchService` honors `RespectGitignore=true` end-to-end | `SearchAsync_WithGitIgnoreEnabled_RespectsGitIgnoreRules` (`Tests/Services/SearchServiceTests.cs`) |
| 22 | Integrated UI search workflow respects gitignore | `SearchWorkflow_WithGitIgnore_RespectsIgnoreRules` (`IntegrationTests/SearchWorkflowTests.cs`) |
| 23 | UI workflow respects `/storage/app/` root-relative anchor | `SearchWorkflow_WithRootRelativeGitIgnorePattern_OnlyMatchesFromRoot` (`IntegrationTests/SearchWorkflowTests.cs`) |

That is 23 asserted edge cases, of which 20 sit directly inside
`GitIgnoreServiceTests`, two inside `IntegrationTests/SearchWorkflowTests`,
and one inside `Tests/Services/SearchServiceTests`.

## Deviations From Canonical Git

The features Grex tests assert are real git features. The list of Grex
deviations from upstream git behavior is:

1. **Case-insensitive matching always**. Canonical git matches against the
   filesystem's case-sensitivity (case-sensitive on ext4, case-insensitive
   on NTFS/APFS). Grex always uses `RegexOptions.IgnoreCase`. On Linux,
   the `ignore` crate is case-sensitive by default; Grexa must decide
   whether to flip the `case_insensitive` builder flag to preserve Grex
   behavior or to follow Linux convention.
2. **No backslash-escapes**. Grex cannot ignore a file literally named
   `#config.txt` because the `#` line is stripped. The `ignore` crate
   supports `\#config.txt`.
3. **No trailing-whitespace escape**. Grex `Trim()`s, dropping any
   `pattern\ ` form that git uses to retain a trailing space.
4. **No `\[` literal-bracket escape**. The Grex bracket parser does not
   recognize escape sequences inside or before `[`; the `ignore` crate
   does.
5. **No `.git/info/exclude` or global excludes**. Grex only reads
   `.gitignore` files; it never looks at `.git/info/exclude`,
   `core.excludesFile`, or `$XDG_CONFIG_HOME/git/ignore`. The `ignore`
   crate honors all three by default (and Grexa's `search.rs` opts in
   via `git_exclude(true)` and `git_global(true)`).
6. **Directory-only patterns use substring, not glob, semantics**. Grex
   matches `build/` by checking whether the relative path *contains*
   `build/`. A literal pattern like `bu*ld/` would not match anything,
   because `IsDirectoryOnly` skips the regex translator entirely. The
   `ignore` crate compiles globs for directory patterns the same way as
   file patterns.
7. **No range or negated character classes**. `[a-z]` and `[!a-z]` are
   not tested. Inspection of `ConvertGitIgnorePatternToRegex` shows
   `[a-z]` works coincidentally (regex preserves the range), but `[!abc]`
   becomes a regex character class with literal `!`, not the git-style
   negation. The `ignore` crate translates `[!...]` to `[^...]`.
8. **Wildcard fallthrough to segment match**. Grex tries the bare filename
   and each path segment against any non-root-relative pattern. Git only
   matches the pattern at one of the documented locations (relative to
   the `.gitignore` directory unless `**/` is used). In practice, the
   non-canonical fall-through usually produces the same answer because
   most user patterns are either filename globs or directory anchors;
   pathological cases with multi-segment patterns like `a/b` may match
   more aggressively in Grex than in canonical git.

## Cross-reference: `ignore` Crate Coverage

The `ignore` crate (BurntSushi, the same author as ripgrep) covers
gitignore semantics with very high fidelity. Empirically, ripgrep is
trusted as a reference implementation of gitignore behavior across the
Rust ecosystem.

Features that match Grex 1-for-1 in the `ignore` crate (or exceed it):

- Root-relative `/pattern`.
- Directory-only `pattern/`.
- Negation `!pattern`.
- Double-star `**/`, `/**/`, trailing `/**`.
- Character classes `[abc]`, ranges `[a-z]`, negation `[!abc]`.
- Comment lines `#`.
- Multiple `.gitignore` files combined per-directory.
- Backslash escapes (`\#`, `\!`, `\[`, `\ `).
- `.git/info/exclude` and global excludes.
- `.ignore` and `.rgignore` fallback files.

Features where Grex and the `ignore` crate diverge — Grexa must pick a
side:

- **Case sensitivity**. Grex is unconditionally case-insensitive. The
  crate is case-sensitive by default. Grexa currently does not flip the
  flag, so `*.LOG` in `.gitignore` does NOT match `app.log` on Linux. If
  Grex parity matters for migrated Windows projects, configure
  `GitignoreBuilder::case_insensitive(true)` or expose a runtime option.
- **Pattern-without-wildcard substring match**. Grex matches the literal
  pattern at any `/`-delimited boundary, so `.env` in the root `.gitignore`
  ignores `nested/dir/.env`. The `ignore` crate follows canonical git: a
  bare `.env` is only matched against the filename anywhere below the
  `.gitignore` directory. The two produce identical results for this
  case, but the algorithms differ — Grex would also match `nested/.env`
  on a pattern of `dir/.env`, which the `ignore` crate would not.
- **Empty pattern body after `!`**. Grex skips `!` (the negation marker
  with empty body) silently after the trim. The `ignore` crate also
  skips it, so this is consistent — but it deserves a regression test.
- **Malformed character class graceful fallback**. Grex falls back to
  `SimpleMatch`. The `ignore` crate returns a parse error from
  `GitignoreBuilder::add_line`; `WalkBuilder` then logs the error and
  carries on. End-to-end behavior is the same (the pattern is silently
  inert), but a future Grexa diagnostic surface should expose ignore
  parse errors instead of swallowing them.

Features where the `ignore` crate exceeds Grex:

- Walking respects `.git/info/exclude` and `core.excludesFile`.
- `.ignore` and `.rgignore` are honored (ripgrep convention).
- `parents(true)` walks up the directory tree above `root` looking for an
  enclosing `.gitignore`. Grex does not; it only sees `.gitignore` files
  at or below `rootPath`. Grexa's `search.rs` does not set `parents`, so
  current behavior matches Grex here.

## Grexa Implementation Hooks

In `crates/grexa-core/src/search.rs` lines 158-167, the wiring is:

```rust
let mut walker = WalkBuilder::new(&options.path);
walker
    .hidden(!options.include_hidden)
    .git_ignore(options.respect_gitignore)
    .git_exclude(options.respect_gitignore)
    .git_global(options.respect_gitignore)
    .ignore(options.respect_gitignore)
    .follow_links(options.include_symlinks)
    .same_file_system(false);
```

Observations:

- The four ignore flags are tied to a single `respect_gitignore` bool.
  Grex's `SearchService` only has one toggle too, so this is fine.
- `parents` is not set. Default is `false`, matching Grex.
- `case_insensitive` is not set on the underlying `Gitignore` builders.
  This is the largest behavioral gap with Grex.
- `.ignore` and `.rgignore` are enabled by `ignore(true)`. Grex has no
  equivalent — files that are excluded by `.ignore` but not `.gitignore`
  would be filtered in Grexa but not in Grex. Mostly a non-issue because
  almost no project ships a `.ignore` without also covering it in
  `.gitignore`.

## Golden-Test Plan

The following numbered test cases lock parity between Grex and Grexa.
Each case is shaped as `(.gitignore body, file path under root,
expected_ignored)`. All paths are forward-slash relative paths beneath
the search root. Tests should be implemented as Rust unit tests inside
`crates/grexa-core/src/search.rs` (or a sibling `gitignore_parity.rs`
module) that materialize the listed `.gitignore` file in a `tempdir`
and assert the file is or is not present in the walker output.

Group A — Basic literal and wildcard patterns

1. `*.log` ⇒ `error.log` ⇒ ignored.
2. `*.log` ⇒ `error.txt` ⇒ kept.
3. `*.log` ⇒ `nested/dir/error.log` ⇒ ignored.
4. `.env` ⇒ `.env` ⇒ ignored.
5. `.env` ⇒ `.env.docker` ⇒ kept.
6. `.env` ⇒ `nested/dir/.env` ⇒ ignored.
7. `test*.txt` ⇒ `test.txt` ⇒ ignored.
8. `test*.txt` ⇒ `test123.txt` ⇒ ignored.
9. `test*.txt` ⇒ `mytest.txt` ⇒ kept.
10. `test?.txt` ⇒ `test1.txt` ⇒ ignored.
11. `test?.txt` ⇒ `test.txt` ⇒ kept.
12. `test?.txt` ⇒ `test12.txt` ⇒ kept.

Group B — Directory-only patterns

13. `build/` ⇒ `build/output.txt` ⇒ ignored.
14. `build/` ⇒ `src/build.rs` ⇒ kept (file, not directory).
15. `build/` ⇒ `nested/build/out.bin` ⇒ ignored.
16. `node_modules/` ⇒ `node_modules/pkg/index.js` ⇒ ignored.
17. `node_modules/` ⇒ `node_modules` (as a file, no descendants) ⇒
    `ignore` crate treats this as a directory pattern, so the bare file
    is kept; document this if Grex parity is required.

Group C — Root-relative patterns

18. `/storage/app/` ⇒ `storage/app/file.txt` ⇒ ignored.
19. `/storage/app/` ⇒ `app/file.txt` ⇒ kept (the regression in
    `ShouldIgnoreFile_WithRootRelativePattern_OnlyMatchesFromRoot`).
20. `/storage/app/` ⇒ `storage/app/subdir/file.txt` ⇒ ignored.
21. `/storage/app` ⇒ `app/Http/Middleware/Foo.php` ⇒ kept.
22. `/secrets.txt` ⇒ `secrets.txt` ⇒ ignored.
23. `/secrets.txt` ⇒ `sub/secrets.txt` ⇒ kept.

Group D — Double-asterisk patterns

24. `**/test.txt` ⇒ `test.txt` ⇒ ignored.
25. `**/test.txt` ⇒ `subdir/test.txt` ⇒ ignored.
26. `**/test.txt` ⇒ `a/b/c/test.txt` ⇒ ignored.
27. `logs/**` ⇒ `logs/2026/05/server.log` ⇒ ignored.
28. `logs/**` ⇒ `logs` (directory itself, no descendants) ⇒ ignored as
    a directory entry.
29. `a/**/b` ⇒ `a/b` ⇒ ignored.
30. `a/**/b` ⇒ `a/x/y/b` ⇒ ignored.
31. `a/**/b` ⇒ `c/a/b` ⇒ kept.

Group E — Negation

32. `*.log\n!important.log` ⇒ `app.log` ⇒ ignored.
33. `*.log\n!important.log` ⇒ `important.log` ⇒ kept.
34. `test*.txt\n!test_backup.txt\n*.tmp\n*.bak` ⇒ `test.txt` ⇒ ignored.
35. Same body ⇒ `test_backup.txt` ⇒ kept.
36. Same body ⇒ `cache.tmp` ⇒ ignored.

Group F — Character classes

37. `test[12].txt` ⇒ `test1.txt` ⇒ ignored.
38. `test[12].txt` ⇒ `test2.txt` ⇒ ignored.
39. `test[12].txt` ⇒ `test3.txt` ⇒ kept.
40. `test[12].txt` ⇒ `test.txt` ⇒ kept.
41. `[A-Z]*.cs` ⇒ `Main.cs` ⇒ ignored.
42. `[A-Z]*.cs` ⇒ `main.cs` ⇒ kept (case-sensitive).
43. `[!a-z]*.txt` ⇒ `Readme.txt` ⇒ ignored (negated class).
44. `[!a-z]*.txt` ⇒ `notes.txt` ⇒ kept.

Group G — Nested `.gitignore`

45. Root `.gitignore`: `*.txt`. `subdir/.gitignore`: `!subdir.txt`. File
    `subdir/subdir.txt` ⇒ kept.
46. Same setup. File `subdir/other.txt` ⇒ ignored (no negation for it).
47. Same setup. File `root.txt` ⇒ ignored.

Group H — Comments and blanks

48. `# ignore logs\n*.log` ⇒ `app.log` ⇒ ignored.
49. `# ignore logs\n*.log\n\n` ⇒ `app.log` ⇒ ignored (trailing blank).
50. `# only comments` ⇒ `anything.txt` ⇒ kept.
51. Empty `.gitignore` ⇒ `anything.txt` ⇒ kept.

Group I — Escape sequences (deviation from Grex)

52. `\#config` ⇒ `#config` ⇒ ignored in `ignore` crate, kept in Grex.
    Mark as a known-divergence test; both implementations should be
    documented.
53. `\!literal` ⇒ `!literal` ⇒ ignored in `ignore` crate, kept in Grex.
54. `foo\ ` (trailing-space form) ⇒ `foo ` ⇒ ignored in `ignore` crate,
    kept in Grex.
55. `\[abc].txt` ⇒ `[abc].txt` ⇒ ignored in `ignore` crate, kept in Grex.

Group J — Malformed patterns

56. `[unterminated\n*.txt\n!keep.txt` (mixed malformed + valid lines)
    plus a file `app.txt` ⇒ ignored. Tests that malformed lines do not
    poison the rest of the file.
57. Same body, file `keep.txt` ⇒ kept.

Group K — Case sensitivity (deviation from Grex)

58. `*.LOG` ⇒ `app.log` ⇒ kept in `ignore` crate (case-sensitive on
    Linux), ignored in Grex. Mark as a divergence test; pick one and
    pin it.
59. `BUILD/` ⇒ `build/x.txt` ⇒ same divergence.

Group L — Out-of-tree files

60. File path outside the search root: `..` ⇒ should never be reachable
    by the walker; test that no file outside `root` is enumerated at
    all, with or without `.gitignore`.

Group M — Search integration

61. Search root contains both `.gitignore` and a matching file. Running
    a search with `respect_gitignore = true` must not return the
    ignored file in `summary.results`.
62. Same setup with `respect_gitignore = false` ⇒ the file IS returned.
63. Search through a subtree where the parent `.gitignore` ignores
    `*.log` and a deeper `.gitignore` negates a specific filename: the
    negated file appears, the others do not.

That is 63 golden cases organized into 13 feature groups. The minimum
viable parity suite is groups A through G (cases 1-47), totalling 47
cases that exercise every behavior asserted in Grex's own test file.
Groups H-M add escape, malformed, case, and integration coverage that
Grex either tests partially or not at all.

## Suggested Next Steps For Grexa

1. Add a `gitignore_parity` integration-test module under
   `crates/grexa-core/tests/` that materializes the golden cases above
   against a `tempdir`. Lock the case-sensitivity policy by either
   flipping `case_insensitive` on the builder or pinning the test
   expectations to canonical (case-sensitive) git semantics. The
   recommended default is canonical Linux behavior — case-sensitive —
   with an opt-in setting to mirror Grex's Windows-native default for
   imported workspaces.
2. Surface ignore-file parse errors as a new `SkipReason::IgnoreParse`
   variant in `search.rs::ProgressEvent`, so the UI can diagnose
   malformed `.gitignore` files instead of silently dropping the rule.
3. Document the four behavioral divergences (case, escapes,
   case-class negation, parent-directory walking) in
   `docs/linux-decisions.md` so they survive future refactors.
