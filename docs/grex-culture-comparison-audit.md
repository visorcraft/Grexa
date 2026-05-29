# Grex Culture-Aware String Comparison Audit

This document records the precise behavior of Grex's culture-aware,
Unicode-normalization-aware, and diacritic-aware string comparison so that
Grexa can either reproduce it on Linux or document an intentional deviation.
The audit is the source of truth for the Rust port of
`Services/SearchService.cs::ContainsStringWithCultureAwareComparison` and the
related normalization plumbing.

Source evidence:

- `Services/SearchService.cs` (lines 77-94: dispatch surface; 152-162: regex
  compilation; 491, 621, 733, 802: per-line comparison entry points;
  1556-1641: `ContainsStringWithCultureAwareComparison`, `RemoveDiacritics`,
  `GetStringComparison`)
- `Models/StringComparisonMode.cs` (three-value enum)
- `Models/UnicodeNormalizationMode.cs` (five-value enum, mapping extension)
- `Tests/Services/SearchServiceTests.cs` (case-sensitive and case-insensitive
  Ordinal fixtures around lines 120-198; all other invocations pass
  `StringComparisonMode.Ordinal, UnicodeNormalizationMode.None,
  diacriticSensitive: true, culture: null`)
- `IntegrationTests/SearchWorkflowTests.cs` (no culture/normalization
  fixtures; uses defaults)
- `Tests/Grex.Cli.Tests/CliSearchRunnerTests.cs` (mocks; verifies surface
  but not behavior)
- `crates/grexa-core/src/search.rs::normalize_for_text_search` (current Rust
  approximation)

> Note on the Grex docs claim: the user-facing documentation states that
> "regex search ignores culture, normalization, and diacritic flags." This
> audit verifies that claim against the source and finds one important
> deviation: .NET's `Regex` *does* honor culture for case-insensitive
> matching unless `RegexOptions.CultureInvariant` is set — and Grex does
> not set it. See section 4.

## 1. Comparison Mode Matrix

`Models/StringComparisonMode` is a plain three-value enum (no flags):

| Mode             | Enum value | .NET `StringComparison` (case-sensitive) | .NET `StringComparison` (case-insensitive) |
| ---------------- | ---------- | ---------------------------------------- | ------------------------------------------ |
| Ordinal          | 0          | `Ordinal`                                | `OrdinalIgnoreCase`                        |
| CurrentCulture   | 1          | `CurrentCulture`                         | `CurrentCultureIgnoreCase`                 |
| InvariantCulture | 2          | `InvariantCulture`                       | `InvariantCultureIgnoreCase`               |

The mapping lives in `GetStringComparison(mode, caseSensitive, culture)` at
`SearchService.cs:1626`. The `culture` argument is *not* consulted by
`GetStringComparison` itself — it is only consulted by the explicit
`CompareInfo.IndexOf` fast path in `ContainsStringWithCultureAwareComparison`
at lines 1581-1595 and only when `mode == CurrentCulture`. The implications:

- `Ordinal` is the codepoint-wise comparison. Case-insensitive ordinal
  comparison does only ASCII case folding for `[A-Z] <-> [a-z]`; all other
  codepoints are compared bit-for-bit. This is identical to what
  `s.eq_ignore_ascii_case` produces in Rust when restricted to ASCII, but
  .NET extends this with simple casefold for the BMP via Unicode tables —
  see "Ordinal edge cases" below.
- `InvariantCulture` uses the culture-invariant `CompareInfo` that .NET
  ships with. The casefold and collation tables come from .NET's bundled
  ICU (since .NET 5) or NLS (legacy Windows-only). Behaviour differs from
  Ordinal for ligature equivalence (`æ` vs `ae` under
  `IgnoreNonSpace`-equivalent options is *not* enabled here, but the
  casefold tables differ on Turkish dotless-I, full casefold of `ß` to
  `SS`, etc).
- `CurrentCulture` follows the thread's `CultureInfo.CurrentCulture` *unless*
  the caller has supplied a `culture` argument and the mode is exactly
  `CurrentCulture`. In that branch only, the code constructs
  `CultureInfo.GetCultureInfo(culture)` and goes through
  `comparisonCulture.CompareInfo.IndexOf(text, searchTerm, compareOptions)`.
  This is the **only** path in Grex where the user-supplied culture override
  takes effect.

### Ordinal edge cases (case-insensitive)

`OrdinalIgnoreCase` in .NET uses `ToUpperInvariant`-equivalent simple
casefold per character. Consequences relevant to test fixtures:

- ASCII letters: bit-flip of bit 5 — `I <-> i`, `A <-> a`.
- Unicode: lowercase Cyrillic, Greek, Latin Extended folds via
  `ToUpperInvariant`. So `É` -> `É` (matches `é` ignore-case).
- Turkish capital dotless I (`İ`, U+0130) does *not* fold to `i` (U+0069)
  under ordinal — `İ` only folds to `i + U+0307` if NFD is applied first.
  Under `OrdinalIgnoreCase` they do not match.
- Sharp-s `ß` (U+00DF) does *not* fold to `SS` under ordinal — it only
  folds to itself.

### Culture-sensitive surprises

`CurrentCultureIgnoreCase` under `tr-TR`:

- `"I".Contains("i", CurrentCultureIgnoreCase)` returns **false** because
  Turkish maps dotless I (`I` U+0049) to dotless i (`ı` U+0131), and
  dotted I (`İ` U+0130) to dotted i (`i` U+0069).
- This is the canonical Turkish I problem and the reason Grex exposes a
  culture override at all.

`CurrentCultureIgnoreCase` under `de-DE`:

- `"strasse".Contains("straße", CurrentCultureIgnoreCase)` returns **true**
  via the eszett equivalence rule baked into the German collation.
- Under `InvariantCulture` the same comparison returns **false** because
  invariant collation in modern .NET (ICU-backed) does *not* apply that
  language-specific rule.

`CurrentCultureIgnoreCase` under `el-GR`:

- Greek final sigma (`ς` U+03C2) and medial sigma (`σ` U+03C3) are
  collation-equivalent under Greek culture; `"ΟΔΟΣ".Contains("οδος",
  CurrentCultureIgnoreCase)` is **true** under `el-GR` and also under
  `InvariantCulture` (Unicode case-folding has Σ -> σ as the simple fold
  but final sigma normalises). Under `Ordinal` it is **false**.

## 2. Unicode Normalization Modes

`Models/UnicodeNormalizationMode` covers the four standard Unicode forms
plus an explicit `None`:

| Mode   | Enum value | .NET `NormalizationForm` | Description                                  |
| ------ | ---------- | ------------------------ | -------------------------------------------- |
| None   | 0          | (not normalized)         | Strings compared as stored on disk           |
| FormC  | 1          | `FormC`                  | NFC — canonical composition                  |
| FormD  | 2          | `FormD`                  | NFD — canonical decomposition                |
| FormKC | 3          | `FormKC`                 | NFKC — compatibility composition             |
| FormKD | 4          | `FormKD`                 | NFKD — compatibility decomposition           |

The mapping is in `UnicodeNormalizationExtensions.ToNormalizationForm`. The
default for legacy callers is `FormC` (note the `_ => FormC` fallback in
the extension method), but the default for the search dispatcher itself is
`None` (`SearchService.cs:78`).

Application order inside `ContainsStringWithCultureAwareComparison`
(`SearchService.cs:1566-1601`):

1. If `mode != None`: apply `text.Normalize(form)` and
   `searchTerm.Normalize(form)`.
2. If `!diacriticSensitive`: apply `RemoveDiacritics` to both strings.
   `RemoveDiacritics` itself does an internal `Normalize(FormD)`, then
   drops `NonSpacingMark` codepoints, then re-`Normalize(FormC)`. This
   means a user choice of `FormD` or `FormKD` is *overwritten* back to FormC
   when diacritic-insensitive search is requested.
3. If `mode == CurrentCulture` and `culture` is supplied: use
   `CultureInfo.GetCultureInfo(culture).CompareInfo.IndexOf` with
   `IgnoreCase` or `None`.
4. Otherwise: `text.Contains(searchTerm, GetStringComparison(...))`.

Practical implications:

- NFKC and NFKD perform *compatibility* mappings: ligature `ﬁ` (U+FB01)
  becomes `fi`, fullwidth Latin `Ａ` becomes `A`, superscript digits like
  `²` become `2`, Roman numerals `Ⅳ` become `IV`. NFC and NFD do not do
  any of these.
- A user enabling `FormKC` + `Ordinal` + case-sensitive should match
  `ﬁle` when searching `file`. This is the only path that yields that
  result.
- `FormD` of `é` (U+00E9) is `e` + U+0301. Searching with `FormD + Ordinal
  + case-sensitive` for `é` against text containing pre-composed `é`
  matches because both sides are decomposed.

## 3. Diacritic Sensitivity

Controlled by the `diacriticSensitive: bool` parameter (default true at the
dispatcher; the UI checkbox toggles it). When false, both the haystack
line and the needle are passed through `RemoveDiacritics`:

```
text.Normalize(FormD)
    .Where(c => CharUnicodeInfo.GetUnicodeCategory(c) != NonSpacingMark)
    .Normalize(FormC)
```

This strips Unicode category `Mn` (NonSpacingMark) only. It does **not**
strip `Mc` (SpacingCombiningMark) or `Me` (EnclosingMark). It does **not**
strip diacritics that are encoded as pre-composed codepoints with no
combining mark form (e.g., Polish `ł` U+0142 has no NFD decomposition
into `l + combining`, so it stays). It does **not** alter ligatures
(`æ`, `œ`, `ß`).

### Concrete examples

| Input text | Needle | diacriticSensitive | Match? | Why                                                            |
| ---------- | ------ | ------------------ | ------ | -------------------------------------------------------------- |
| `café`     | `cafe` | true               | no     | `é` (U+00E9) is one codepoint, never equals `e` (U+0065)       |
| `café`     | `cafe` | false              | yes    | `é` -> `e + U+0301` -> drop U+0301 -> `e`                      |
| `straße`   | `strasse` | false           | no     | `ß` has no combining-mark decomposition; stays `ß`             |
| `straße`   | `strasse` | true, `CurrentCulture=de-DE`, case-insensitive | yes | German collation folds `ß <-> ss` |
| `straße`   | `strasse` | true, `InvariantCulture`, case-insensitive | no | Invariant collation does not fold `ß` |
| `Ωδός`     | `οδός` | false              | yes (capital sigma is fold-equivalent and accents stripped)    |
| `Türkçe`   | `turkce` | false             | yes    | `ü -> u`, `ç -> c` after NFD strip                             |
| `İstanbul` | `istanbul` | true, `Ordinal`, case-insensitive | no  | Ordinal does not fold U+0130 to U+0069                        |
| `İstanbul` | `istanbul` | true, `CurrentCulture=tr-TR`, case-insensitive | yes | Turkish casefold maps İ -> i             |
| `İstanbul` | `istanbul` | true, `CurrentCulture=en-US`, case-insensitive | yes (en-US folds İ -> i̇ ≈ i, and CompareInfo treats them equal in ignore-case) |
| `Iraq`     | `iraq` | true, `CurrentCulture=tr-TR`, case-insensitive | no  | Turkish casefold maps `I` -> `ı`, not `i`                       |
| `ﬁle`      | `file` | true, `Ordinal`, FormKC | yes | NFKC decomposes ligature                                        |
| `ﬁle`      | `file` | true, `Ordinal`, FormC  | no  | NFC does not decompose compatibility ligatures                  |

## 4. Regex vs. Culture/Normalization/Diacritic

**Claim under audit:** Grex docs say regex mode ignores the culture,
normalization, and diacritic flags. **Verdict:** partially true.

Source evidence (`SearchService.cs:152-162`):

```
Regex? compiledRegex = null;
if (isRegex)
{
    var regexOptions = RegexOptions.Compiled;
    if (!searchCaseSensitive)
    {
        regexOptions |= RegexOptions.IgnoreCase;
    }
    compiledRegex = new Regex(searchTerm, regexOptions);
}
```

And the regex consumer at `SearchService.cs:477-487`:

```
if (isRegex && compiledRegex != null)
{
    matches = compiledRegex.IsMatch(line);  // no normalization, no diacritic strip
}
else
{
    matches = ContainsStringWithCultureAwareComparison(...);
}
```

Findings:

1. **Normalization is genuinely ignored.** Regex never invokes
   `Normalize(form)` on either side. `é` literally matches only `é`
   pre-composed; it will not match `e` + U+0301. A user enabling FormD
   will not see decomposition applied to the line text.
2. **Diacritic flag is genuinely ignored.** No `RemoveDiacritics` call is
   in the regex path. A user toggling diacritic-insensitive will not see
   the regex strip combining marks from input lines.
3. **Culture is partially honored.** Because Grex does *not* set
   `RegexOptions.CultureInvariant`, the regex engine's
   `IgnoreCase` flag uses .NET's `CultureInfo.CurrentCulture` at the time
   `Regex.Match` runs. On a Turkish-localized Windows machine, the regex
   pattern `[Ii]` with `IgnoreCase` will match `İ` and `ı` according to
   Turkish casefold rules. This contradicts the user-facing doc claim.

**Parity recommendation for Grexa:** the Rust `regex` crate has no
locale awareness at all — its `case_insensitive` flag uses Unicode simple
casefold derived from the Unicode standard, equivalent to .NET's
`InvariantCulture`-ish behavior. We should:

- Document Grexa regex as locale-invariant (the *intended* Grex
  behavior).
- Treat any Grex test that relied on Turkish regex casefolding as a
  known parity gap and skip it with a comment.

## 5. Fixture Matrix for Rust Reproduction

The Grex test suite is sparse on culture fixtures — only the two ordinal
case-sensitivity tests in `SearchServiceTests.cs` exercise this code path
end-to-end. Everything else mocks the comparator. To get meaningful
parity coverage, Grexa must seed its own fixture set derived from the
documented semantics above. Each row is `(comparison mode,
normalization, diacritic-sensitive, case-sensitive, haystack, needle,
expect_match)`.

### 5.1 Pure Ordinal Fixtures (10)

| # | Mode    | Norm | Diac | Case | Haystack       | Needle    | Match |
| - | ------- | ---- | ---- | ---- | -------------- | --------- | ----- |
| 1 | Ordinal | None | yes  | yes  | `Banana`       | `Banana`  | yes   |
| 2 | Ordinal | None | yes  | yes  | `Banana`       | `banana`  | no    |
| 3 | Ordinal | None | yes  | no   | `Banana`       | `banana`  | yes   |
| 4 | Ordinal | None | yes  | yes  | `café`         | `cafe`    | no    |
| 5 | Ordinal | None | no   | yes  | `café`         | `cafe`    | yes   |
| 6 | Ordinal | None | no   | yes  | `Café`         | `cafe`    | no (case differs) |
| 7 | Ordinal | None | no   | no   | `Café`         | `cafe`    | yes   |
| 8 | Ordinal | None | yes  | no   | `İstanbul`     | `istanbul`| no (ordinal does not fold U+0130) |
| 9 | Ordinal | None | yes  | yes  | `straße`       | `straße`  | yes   |
| 10| Ordinal | None | yes  | no   | `straße`       | `STRASSE` | no    |

### 5.2 Invariant Culture Fixtures (6)

| #  | Mode             | Norm | Diac | Case | Haystack | Needle   | Match |
| -- | ---------------- | ---- | ---- | ---- | -------- | -------- | ----- |
| 11 | InvariantCulture | None | yes  | yes  | `straße` | `STRASSE`| no    |
| 12 | InvariantCulture | None | yes  | no   | `straße` | `STRASSE`| no (invariant does not fold eszett) |
| 13 | InvariantCulture | None | yes  | no   | `İstanbul`| `istanbul`| yes (invariant folds İ via simple casefold) |
| 14 | InvariantCulture | None | no   | no   | `Café`   | `CAFE`   | yes   |
| 15 | InvariantCulture | None | yes  | no   | `Ωδος`   | `ωδος`   | yes   |
| 16 | InvariantCulture | None | yes  | no   | `Iraq`   | `IRAQ`   | yes   |

### 5.3 CurrentCulture With Culture Override (12)

| #  | Mode           | Culture | Norm | Diac | Case | Haystack    | Needle     | Match |
| -- | -------------- | ------- | ---- | ---- | ---- | ----------- | ---------- | ----- |
| 17 | CurrentCulture | tr-TR   | None | yes  | no   | `İstanbul`  | `istanbul` | yes   |
| 18 | CurrentCulture | tr-TR   | None | yes  | no   | `Iraq`      | `iraq`     | no (Turkish folds I to ı) |
| 19 | CurrentCulture | tr-TR   | None | yes  | no   | `Iraq`      | `ıraq`     | yes   |
| 20 | CurrentCulture | en-US   | None | yes  | no   | `İstanbul`  | `istanbul` | yes   |
| 21 | CurrentCulture | en-US   | None | yes  | no   | `Iraq`      | `iraq`     | yes   |
| 22 | CurrentCulture | de-DE   | None | yes  | no   | `straße`    | `STRASSE`  | yes (German eszett rule) |
| 23 | CurrentCulture | de-DE   | None | yes  | yes  | `straße`    | `strasse`  | no (case-sensitive disables fold) |
| 24 | CurrentCulture | de-DE   | None | yes  | no   | `Großstadt` | `GROSSSTADT`| yes |
| 25 | CurrentCulture | el-GR   | None | yes  | no   | `ΟΔΟΣ`      | `οδος`     | yes (final-sigma fold) |
| 26 | CurrentCulture | el-GR   | None | yes  | no   | `λόγος`     | `ΛΟΓΟΣ`    | yes   |
| 27 | CurrentCulture | (bogus)`xx-XX` | None | yes | no | `Banana`   | `banana`   | yes (falls back to OrdinalIgnoreCase per catch block) |
| 28 | CurrentCulture | (null)  | None | yes  | no   | `Banana`    | `banana`   | yes (falls back to thread's current culture, which the host process pins to en-US for tests) |

### 5.4 Normalization Form Fixtures (10)

`café-d` denotes `cafe` + combining acute. `café-c` denotes pre-composed.

| #  | Mode    | Norm   | Diac | Case | Haystack         | Needle           | Match |
| -- | ------- | ------ | ---- | ---- | ---------------- | ---------------- | ----- |
| 29 | Ordinal | None   | yes  | yes  | `café-d`         | `café-c`         | no    |
| 30 | Ordinal | FormC  | yes  | yes  | `café-d`         | `café-c`         | yes   |
| 31 | Ordinal | FormD  | yes  | yes  | `café-c`         | `café-d`         | yes   |
| 32 | Ordinal | FormC  | yes  | yes  | `café-d`         | `café-d`         | yes   |
| 33 | Ordinal | None   | no   | yes  | `café-d`         | `cafe`           | yes (FormD+strip in RemoveDiacritics covers this) |
| 34 | Ordinal | FormKC | yes  | yes  | `ﬁle`            | `file`           | yes   |
| 35 | Ordinal | FormC  | yes  | yes  | `ﬁle`            | `file`           | no    |
| 36 | Ordinal | FormKC | yes  | yes  | `Ａ１２３`        | `A123`           | yes   |
| 37 | Ordinal | FormKD | no   | yes  | `Café-c`         | `cafe`           | no (case differs) |
| 38 | Ordinal | FormKD | no   | no   | `Café-c`         | `cafe`           | yes   |

### 5.5 Regex-vs-Culture Fixtures (5)

These document the *current* Grex regex behavior, which deviates from
the docs claim.

| #  | Regex pattern | IgnoreCase | Thread culture | Haystack    | Match (Grex actual) |
| -- | ------------- | ---------- | -------------- | ----------- | ------------------- |
| 39 | `[Ii]`        | yes        | tr-TR          | `İstanbul`  | yes (Turkish fold)  |
| 40 | `i`           | yes        | tr-TR          | `I`         | no                  |
| 41 | `i`           | yes        | en-US          | `I`         | yes                 |
| 42 | `café`        | no         | en-US          | `café`      | yes                 |
| 43 | `café`        | no         | en-US          | `cafe`      | no (no NFD applied) |

**Grexa intent:** rows 39 and 40 will produce different behavior in the
Rust port (Unicode-invariant casefold). Document this in
`grex-audit-inventory.md` as an intentional parity gap, not a bug.

### Fixture totals

- Section 5.1: 10
- Section 5.2: 6
- Section 5.3: 12
- Section 5.4: 10
- Section 5.5: 5
- **Total: 43 fixtures**

## 6. ICU Strategy Recommendation

The Rust standard library has no notion of culture-aware comparison.
`str::contains` is byte-wise; `unicode-normalization` covers NFC/NFD/
NFKC/NFKD; the `caseless` crate provides Unicode simple and full
casefold; `regex::RegexBuilder::case_insensitive(true)` uses Unicode
simple casefold (locale-invariant).

To match the Grex matrix above, the candidate stacks are:

### Option A: ICU4X (pure-Rust, bundled data)

- Covers: full collation, locale-aware casefolding (`icu_casemap` /
  `icu_collator`), normalization, and segmenter.
- Reproduces: all of section 5.1, 5.2, 5.3 including Turkish, German,
  Greek cases. Section 5.4 normalization is straight `icu_normalizer`.
  Section 5.5 cannot be reproduced because we're using Rust's `regex`
  crate, but that is the desired behavior (locale-invariant regex per
  the Grex docs claim).
- Caveats: data size is significant (~4-7 MiB embedded when full CLDR
  data is included). Build complexity is real — ICU4X requires
  `databake` or `provider` setup to bundle CLDR tables. API churn until
  ICU4X 2.0 stabilises.
- Cost: medium-high integration effort; zero runtime dependency on a
  system library.

### Option B: System ICU4C via `rust_icu_*`

- Covers: same as A; matches .NET's bundled ICU more closely because
  .NET on modern platforms also uses ICU4C.
- Reproduces: all of section 5.1-5.4 with very high fidelity. Likely the
  closest behavioral match to Windows/.NET 8 Grex on a per-test-row
  basis.
- Caveats: requires `libicu-dev` (Debian/Ubuntu), `icu` (Arch), or
  `icu4c` (Fedora) at build and runtime. Version skew between distros
  causes subtle differences (e.g., Ubuntu 22.04 ships ICU 70, Arch
  rolling ships ICU 76; some collation rules change between versions).
  Static linking is possible but adds another ~10 MiB.
- Cost: low integration effort; high deployment friction (extra
  runtime dependency, distro-specific packaging).

### Option C: `caseless` + `unicode-normalization` only (status quo path)

- Covers: NFC/NFD/NFKC/NFKD, Unicode-default simple+full casefold,
  combining-mark stripping (already done in `normalize_for_text_search`).
- Reproduces: all of section 5.1 (Ordinal). Section 5.2 (Invariant)
  approximately matches because Unicode simple casefold is essentially
  what InvariantCultureIgnoreCase does — but with edge cases on Turkish
  dotless I and German eszett that differ. Section 5.4 matches
  perfectly. Section 5.3 (Turkish, German, Greek culture-specific
  rules) cannot be reproduced at all.
- Caveats: forces us to ship Grexa without the Grex "selected culture"
  override feature, or to ship it as a no-op with a UI hint.
- Cost: lowest — already implemented for the diacritic and
  normalization paths in `normalize_for_text_search`.

### Recommendation

**Option A (ICU4X) for the v1 Linux port, gated behind a feature flag
`grexa-culture-icu` that is on by default**, with Option C remaining as
the fallback compile path for embedded/minimal builds. Rationale:

- ICU4X is the only path that lets us reproduce Turkish-I, German-eszett,
  Greek-sigma fixtures (sections 5.3) without taking a system-library
  dependency.
- ICU4C (Option B) ties Grexa to whichever ICU version the host distro
  ships, producing user-visible inconsistencies between Arch and Debian
  users for the same fixture.
- Option C is preserved as the fallback so a build with `--no-default-
  features` still compiles and passes section 5.1, 5.2 (approximately),
  and 5.4 fixtures.

Concrete crate set for Option A:

- `icu_normalizer` — NFC/NFD/NFKC/NFKD plus combining-mark utilities.
- `icu_collator` — culture-aware `IndexOf`-style substring search via
  `Collator::compare_utf8` and a custom needle scanner (ICU4X does not
  ship a one-shot `index_of`; we will need a small Boyer-Moore-style
  loop on top of collation keys, or a linear scan for the v1 cut).
- `icu_casemap` — locale-aware uppercase/lowercase/titlecase/fold for
  the case-folded fast path when collation is overkill.
- `icu_locid` — parsing `tr-TR`, `de-DE`, `el-GR` user input.
- `icu_provider_blob` or compiled data via `icu_provider_baked` for
  shipping CLDR tables. Plan for ~4 MiB binary increase.

## 7. Performance Notes

Per-line cost (text mode) decomposes as:

| Path                            | Cost (asymptotic, per line) | Notes                                            |
| ------------------------------- | --------------------------- | ------------------------------------------------ |
| Ordinal, None, diac-sens        | O(n)                        | Byte/codepoint compare; fastest.                 |
| Ordinal, FormC, diac-sens       | O(n) + NFC alloc            | NFC is a single allocation per line + needle.    |
| Ordinal, None, diac-insens      | O(n) + NFD alloc + filter   | RemoveDiacritics allocates twice.                |
| Ordinal, FormKD, diac-insens    | O(n) + NFKD + NFD + filter  | Worst-case Ordinal path; double-normalises.      |
| InvariantCulture, *, *          | O(n) via CompareInfo        | ICU-backed; ~5-10x slower than Ordinal.          |
| CurrentCulture + culture string | O(n) via CompareInfo        | Same cost class as Invariant, plus the cost of constructing/looking up the `CultureInfo` (cached after first call). |
| Regex                           | O(n) automaton step         | Independent of mode flags; bounded by pattern.   |

User-facing implications for Grexa status UI:

- **Fast path** (default): Ordinal, None, diacritic-sensitive. No banner.
- **Medium path**: any normalization form, or diacritic-insensitive. Show
  "Normalizing Unicode..." in the status bar during long scans so users
  understand the slowdown.
- **Slow path**: any culture override (`CurrentCulture` + non-null
  culture string), or any path going through ICU collation. Show
  "Culture-aware comparison may be slower" hint next to the culture
  picker, and stream progress aggressively (smaller `FileScanned` batch
  size) so the UI never feels frozen.
- **Cancellation**: the existing `cancel.is_cancelled()` check every 64
  lines in `search_file` is sufficient for Ordinal and FormC paths but
  may be too coarse for the ICU collation path on long lines. Plan to
  add a per-match cancellation check inside the ICU substring loop.

Memory:

- NFD/NFKD allocates a new `String`. For a 4-KiB line with average
  combining-mark density, the decomposed form is ~1.3x. Worst case is
  Hangul or pre-composed Vietnamese at ~3x. Cap line length before
  normalising — the existing `MATCH_PREVIEW_MAX_CHARS = 400` only
  governs the preview, not the comparison; we should add an internal
  `MAX_COMPARE_LINE_BYTES` (suggest 1 MiB) and skip normalisation for
  lines beyond it, falling back to Ordinal with a logged warning.

## 8. Open Questions And Parity Gaps

1. .NET on Linux uses ICU; .NET on Windows historically used NLS. The
   Grex Windows binary is therefore *currently* running against the NLS
   tables, which have small but real differences from ICU's CLDR tables.
   Grexa-on-Linux running against ICU4X CLDR will match the Linux build
   of Grex (if any) but not the Windows build. **Decision:** document as
   a known acceptable drift; do not try to ship NLS tables.
2. .NET regex with `IgnoreCase` and no `CultureInvariant` flag — the
   actual Grex behavior — cannot be reproduced in the Rust `regex` crate
   because `regex` has no locale parameter. **Decision:** do not try;
   document as a deliberate change because the Grex behavior contradicts
   Grex's own user-facing documentation.
3. The bogus-culture fallback at `SearchService.cs:1591` catches
   `CultureNotFoundException` and proceeds with `GetStringComparison`,
   which then uses `StringComparison.CurrentCulture` — i.e., the thread
   culture, not Ordinal. Fixture #27 above captures this. The Rust port
   should match: if the user supplies `xx-XX`, fall back to the host
   locale via `sys-locale`, not to Ordinal.
4. `RemoveDiacritics` strips only `NonSpacingMark` (Mn). .NET's
   implementation matches what we want, and the current Rust
   `normalize_for_text_search` uses `is_combining_mark` which returns
   true for Mn + Mc + Me. **Action item:** narrow the Rust filter to
   `Mn`-only using `unicode_general_category::get_general_category`
   for exact parity. This will affect text containing devanagari or
   thai vowel signs (Mc), where the current Rust implementation strips
   them but Grex does not.
5. Grex's culture override only applies when `mode == CurrentCulture`.
   Even if the user picks `InvariantCulture` and supplies `tr-TR`, the
   override is ignored. The Rust port should preserve this surprising
   behavior or fix it; recommend matching Grex exactly for v1 and
   filing an issue to revisit in v1.1.

## 9. Implementation Checklist

For the Grexa cut over this audit drives:

- [ ] Replace `normalize_for_text_search` with a `Comparator` trait that
  takes `StringComparisonMode`, `UnicodeNormalizationMode`, and an
  optional `Locale` from `icu_locid`.
- [ ] Implement `OrdinalComparator` (byte and ASCII-case-fold paths) —
  covers fixtures 1-10, 29-38.
- [ ] Implement `InvariantComparator` via `icu_collator::Collator` with
  the `und` (undetermined) locale — covers 11-16.
- [ ] Implement `LocaleComparator(locale)` via
  `icu_collator::Collator::try_new(locale)` — covers 17-26.
- [ ] Implement the bogus-culture fallback (try the locale, on error
  fall back to host locale, not Ordinal) — covers 27-28.
- [ ] Wire the diacritic-insensitive transform via `icu_normalizer::
  ComposingNormalizer::new_nfd()` + `Mn`-only filter — match fixture
  33's expectation.
- [ ] Add `grexa-culture-icu` feature flag wrapping all ICU usage; the
  default `cfg(feature = "grexa-culture-icu")` path enables sections
  5.1-5.4. Without the feature, sections 5.3 fall back to invariant
  (Unicode default) and a warning is surfaced in the UI status.
- [ ] Wire `MAX_COMPARE_LINE_BYTES` guard before normalization.
- [ ] Stream progress events more frequently on the ICU collation path.
- [ ] Update `grex-audit-inventory.md` to mark this audit complete and
  cross-reference into the search-service audit.
