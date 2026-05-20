# Grex Encoding Detection Audit

This document records the behavior of Grex `Services/EncodingDetectionService.cs`
that Grexa must either preserve, replace with a Linux-native equivalent, or
document as intentionally non-applicable.

Source evidence:

- `Services/EncodingDetectionService.cs`
- `Services/IEncodingDetectionService.cs`
- `Services/SearchService.cs` (call sites at lines 467, 2195, 2257)
- `Services/ContextPreviewService.cs`
- `ViewModels/TabViewModel.cs` (parallel BOM-only `DetectFileEncoding` at line
  1760, plus the result write at line 1013 and the `"Unknown"` fallback at line
  1037)
- `Models/FileSearchResult.cs` (the `Encoding` field default is `"Unknown"`)
- `Tests/Services/EncodingDetectionServiceTests.cs`

## Public Contract

`IEncodingDetectionService` exposes three entry points:

- `DetectFileEncoding(string filePath)` reads the whole file with
  `File.ReadAllBytes` and delegates to the byte-based path.
- `DetectEncoding(byte[] bytes)` runs on an in-memory buffer with no file name
  context.
- `DetectEncoding(byte[] bytes, string fileName)` adds a file name hint that
  influences the statistical scoring stage only.

All three return an `EncodingDetectionResult` with:

- `Encoding Encoding` — a `System.Text.Encoding` instance.
- `double Confidence` — clamped to `[0.0, 1.0]` in the constructor.
- `bool HasBom` — only set to `true` by the BOM branch.
- `string DetectionMethod` — a human-readable trace line such as
  `BOM detected: UTF-8 with BOM`, `Statistical analysis: Western European (Windows)`,
  `Heuristic: UTF-16 LE detected by null byte pattern`, `Heuristic: Defaulting to UTF-8`,
  `Statistical analysis failed, using UTF-8 fallback`, `Empty file`, or
  `Error reading file: <message>`.

Empty or null input returns `Encoding.UTF8` with confidence `0.1` and the
`DetectionMethod` set to `Empty file`. A file read exception returns
`Encoding.UTF8` with confidence `0.1` and `DetectionMethod` prefixed with
`Error reading file:`.

## Encodings Attempted

Grex builds a `CommonEncodings` array at static-init time. Five encodings are
unconditional (the four `Encoding.*` properties plus `UTF-32BE` populated
through the names list), and another 30 are added via
`Encoding.GetEncoding(name)` inside a try/catch that silently skips unsupported
labels. The total advertised by `docs/grex-audit-inventory.md` of "30+"
corresponds to the 35 candidate encodings below.

Unconditional (always present even without code-page providers):

1. `UTF-8` (`Encoding.UTF8`)
2. `Unicode` — UTF-16 Little Endian (`Encoding.Unicode`)
3. `BigEndianUnicode` — UTF-16 Big Endian
4. `UTF-32` — UTF-32 Little Endian (`Encoding.UTF32`)

Looked up by name (verbatim strings from `EncodingDetectionService.cs`):

5. `UTF-32BE`
6. `ISO-8859-1` (Latin-1)
7. `ISO-8859-2` (Latin-2)
8. `ISO-8859-3` (Latin-3)
9. `ISO-8859-4` (Latin-4)
10. `ISO-8859-5` (Cyrillic)
11. `ISO-8859-6` (Arabic)
12. `ISO-8859-7` (Greek)
13. `ISO-8859-8` (Hebrew)
14. `ISO-8859-9` (Latin-5, Turkish)
15. `ISO-8859-10` (Latin-6, Nordic)
16. `ISO-8859-11` (Thai)
17. `ISO-8859-13` (Latin-7, Baltic)
18. `ISO-8859-14` (Latin-8, Celtic)
19. `ISO-8859-15` (Latin-9, Western European with Euro)
20. `ISO-8859-16` (Latin-10, South-Eastern European)
21. `Windows-1252` (Western European)
22. `Windows-1250` (Central European)
23. `Windows-1251` (Cyrillic)
24. `Windows-1253` (Greek)
25. `Windows-1254` (Turkish)
26. `Windows-1255` (Hebrew)
27. `Windows-1256` (Arabic)
28. `Windows-1257` (Baltic)
29. `Windows-1258` (Vietnamese)
30. `Shift-JIS`
31. `GB2312`
32. `GBK`
33. `Big5`
34. `EUC-KR`
35. `KOI8-R`
36. `KOI8-U`

That is 4 unconditional + 31 lookup names = 35 distinct encodings, but
`UTF-32BE` is reachable by both the `Encoding.UTF32` BOM branch (LE) and the
lookup branch (BE), so the effective count of distinct schemes is 35. Several
of these only resolve on .NET if
`System.Text.Encoding.CodePages` has been registered through
`Encoding.RegisterProvider(CodePagesEncodingProvider.Instance)`; otherwise the
try/catch drops them. The service does not check whether any encodings
survived.

## Detection Order

The flow inside `DetectEncoding(byte[] bytes, string fileName)` is exactly:

1. **BOM check** (`DetectByBom`). Iterates `BomSignatures` and returns on the
   first match. Match always sets `Confidence = 0.95`, `HasBom = true`, and the
   message `BOM detected: <human label>`.
2. **Statistical analysis** (`DetectByStatisticalAnalysis`). Decodes the bytes
   with each `CommonEncodings` entry, scores it, drops scores below `0.1`, and
   keeps the highest-scoring result.
3. **Heuristic fallback** (`DetectByHeuristics`). Only invoked when the
   statistical winner has `Confidence < 0.3`. The heuristic and statistical
   results are then compared by confidence and the higher of the two is
   returned.
4. **Hard fallback**. If statistical analysis produced no candidates at all
   (every encoding threw or scored below `0.1`), the method returns
   `Encoding.UTF8` with confidence `0.2` and `DetectionMethod = "Statistical analysis failed, using UTF-8 fallback"`.
5. **Heuristic hard fallback**. The heuristic path itself returns
   `Encoding.UTF8` with confidence `0.4` and message
   `Heuristic: Defaulting to UTF-8` if no other branch fires.

BOM table (`BomSignatures`), in iteration order:

- `EF BB BF` -> `UTF-8 with BOM`
- `FF FE` -> `UTF-16 Little Endian with BOM`
- `FE FF` -> `UTF-16 Big Endian with BOM`
- `FF FE 00 00` -> `UTF-32 Little Endian with BOM`
- `00 00 FE FF` -> `UTF-32 Big Endian with BOM`

The BOM table iteration order is dictionary-insertion order, which means the
2-byte UTF-16 LE BOM (`FF FE`) is checked **before** the 4-byte UTF-32 LE BOM
(`FF FE 00 00`). UTF-32 LE files therefore mis-detect as UTF-16 LE today; this
is a latent bug. Grexa already fixes it in
`crates/grexa-core/src/encoding.rs` by checking UTF-32 LE before UTF-16 LE.

## Confidence Scoring

`CalculateEncodingConfidence` is a weighted sum of four factors, then clamped
with `Math.Min(1.0, sum)`:

- `CheckValidCharacterSequences` x 0.4 — ratio of decoded characters that are
  not invalid control characters (CR, LF, TAB are allowed).
- `CheckCharacterFrequency` x 0.3 — fraction of the first 10 000 bytes that
  appear in a frequency table for the encoding's `EncodingName`. The frequency
  table store is bootstrapped with only two entries:
  - `UTF-8`: every byte `0..127` mapped to `0.1`.
  - `Western European (Windows)`: every byte `0..255`, with `0.1` for
    `< 128` and `0.01` otherwise.
  Any other encoding hits the neutral `0.5` default and never benefits from
  the table. The `Dictionary<string, ...>` is keyed on
  `encoding.EncodingName`, which is locale-dependent on .NET, so even
  `Encoding.UTF8` may miss the table when the host locale renames it.
- `CheckFileNameHints` x 0.1 — pattern matches on the lower-cased file name:
  - `shift_jis` or `sjis` boosts Shift-encodings to `0.8`
  - `gb2312`, `gbk`, or `chinese` boosts GB-/Chinese-named encodings to `0.8`
  - `euc-kr` or `korean` boosts EUC-KR to `0.8`
  - `koi8`, `cyrillic`, or `russian` boosts KOI8/Cyrillic encodings to `0.8`
  - otherwise returns the neutral `0.5`
  - empty file name returns `0.5`
- `CheckCommonTextPatterns` x 0.2 — examined on the first 1 000 characters of
  the decoded string. Adds `0.2` for whitespace, `0.3` for any letter, `0.1`
  for any digit, `0.2` for any of the keywords `function`, `class`, `import`,
  `export`, `public`, `private`, `var`, `let`, `const` (case-insensitive),
  and `0.2` if `(/)`, `{/}`, `[/]` counts are each within two of balanced.

### Cutoffs

The hard-coded thresholds are:

- `0.1` — minimum to keep a statistical candidate (anything below is dropped
  before sorting).
- `0.3` — confidence below which the heuristic stage runs. **This is the
  practical "cutoff" the audit asks about.**
- `0.95` — fixed confidence for every BOM hit.
- `0.6` — fixed confidence for UTF-16 LE/BE inferred by null-byte heuristic.
- `0.5` — fixed confidence for Shift-JIS, GB2312, and EUC-KR inferred by
  heuristic byte-pattern scans.
- `0.4` — fixed confidence for the UTF-8 heuristic default.
- `0.2` — confidence stamped on the "no statistical winner" UTF-8 fallback.
- `0.1` — confidence used for empty input and read-error fallbacks.

`EncodingDetectionResult` clamps the constructor argument to `[0.0, 1.0]`, so
out-of-range values do not propagate.

## Read Amount

Grex reads the whole file. `DetectFileEncoding(string filePath)` calls
`File.ReadAllBytes(filePath)` and passes the full buffer into `DetectEncoding`.
There is no streaming, no prefix mode, and no size cap.

Within the byte-based path, two sub-routines do bound their work:

- `CheckCharacterFrequency` samples at most the first 10 000 bytes.
- `CheckCommonTextPatterns` samples at most the first 1 000 decoded
  characters.

Everything else (BOM check, statistical decode of each encoding,
`CheckValidCharacterSequences`, every heuristic byte-pattern scan in
`IsLikelyShiftJIS`, `IsLikelyChinese`, `IsLikelyKorean`) iterates the entire
buffer. For a 100 MB log this means the buffer is held in memory and decoded
roughly 35 times.

`TabViewModel.DetectFileEncoding(string)` (the separate, simpler BOM-only
helper used for the result row label) is the only path that reads a prefix —
it opens a `FileStream`, reads `min(4096, length)` bytes, and inspects only
the first three.

## Failure Behavior

The default after all stages fail is always `Encoding.UTF8`. The result row
column shows the constant string `"Unknown"` (`Models/FileSearchResult.cs`
default and `ViewModels/TabViewModel.cs` line 1037) when the surrounding
file-info collection itself throws; that is distinct from the encoding
service's own UTF-8 fallback, which still surfaces as a UTF-8 row.

The `DetectFileEncoding(string)` entry point never throws. Any
`File.ReadAllBytes` exception is caught and converted to a low-confidence
UTF-8 result.

## Label Format

Two different label producers exist in Grex and they do not agree:

1. `SearchService.cs` line 2257 writes `Encoding = encoding.EncodingName` for
   replace-mode results. `EncodingName` is the long, locale-dependent
   description from `System.Text.Encoding`, for example `Unicode (UTF-8)`,
   `Western European (Windows)`, `Japanese (Shift-JIS)`.
2. `TabViewModel.cs` line 1013 writes the result of its inline
   `DetectFileEncoding(string)` helper, which returns one of the short
   constants `"UTF-8"`, `"UTF-16 LE"`, `"UTF-16 BE"`, `"Binary/Unknown"`, or
   `"Unknown"`.

The BOM message strings (used in `DetectionMethod`, not in the column) are
`UTF-8 with BOM`, `UTF-16 Little Endian with BOM`,
`UTF-16 Big Endian with BOM`, `UTF-32 Little Endian with BOM`,
`UTF-32 Big Endian with BOM`.

Grexa's `DetectedEncoding::label` already standardises on the short forms
`UTF-8`, `UTF-8 BOM`, `UTF-16 LE`, `UTF-16 BE`, `UTF-32 LE`, `UTF-32 BE`.
That matches the user-facing `TabViewModel` labels with a single difference —
Grexa uses `UTF-8 BOM` where Grex uses plain `UTF-8` for BOM-decorated files
in the result table. We adopt the Grexa labels and update parity tests
accordingly.

## Memory And Performance

Per-call cost for a file of size `N` bytes with `E` candidate encodings (`E`
is typically 4-35 depending on registered code pages):

- one full `File.ReadAllBytes` allocates `N` bytes.
- BOM check: `O(1)` to `O(5)` comparisons on the prefix.
- Statistical stage: `E` independent `Encoding.GetString` decodes, each
  allocating a UTF-16 `string` of length proportional to `N` characters.
  `CheckValidCharacterSequences` walks the decoded string once.
  `CheckCharacterFrequency` walks the first `min(N, 10 000)` bytes once.
  `CheckCommonTextPatterns` walks the first 1 000 decoded characters.
- Heuristic stage: full-buffer scans in `IsLikelyShiftJIS`,
  `IsLikelyChinese`, `IsLikelyKorean`. Each is `O(N)`.

For a 100 MB file with all 35 encodings registered this is roughly 3.5 GB of
transient string allocation per detect call, with `O(35 * N)` byte work. The
service is invoked once per file by `SearchService.SearchTextFileAsync`
(line 467), once again by `ReplaceTextFileAsync` (line 2195), and again by
`ContextPreviewService` when the preview dialog opens. Caching is absent.

Grexa must avoid this. The replacement should:

- Detect from a bounded prefix (4 KB matches Grex's `TabViewModel` helper).
- Stream the BOM check separately from any heavier heuristic.
- Hand the result down to the search loop so a single detection per file
  feeds both content search and preview.

## Linux Replacement Plan

### Tier 1: Already covered by Grexa BOM path

`grexa-core/src/encoding.rs::detect_from_bytes` already recognises every BOM
that Grex recognises. The five BOM-based encodings need no further work:

- `UTF-8` (with and without BOM)
- `UTF-16 LE`
- `UTF-16 BE`
- `UTF-32 LE`
- `UTF-32 BE`

Two fixes carried by Grexa over Grex:

- UTF-32 LE BOM is checked before UTF-16 LE BOM (Grex's iteration order is
  buggy).
- UTF-8 BOM produces a distinct label (`UTF-8 BOM`) instead of folding into
  plain UTF-8.

Decoding via `encoding_rs` covers UTF-8 and UTF-16 LE/BE end-to-end. UTF-32 is
currently surfaced as lossy UTF-8 because `encoding_rs` does not decode it;
Grex's own BOM path also relies on
`Encoding.GetEncoding("UTF-32BE")` and produces decoded text only when the
code-page provider is registered, so we are not losing parity for the common
case. Plan: add a manual UTF-32 LE/BE decoder (or shell to `iconv` once Tier 3
is in) when a real-world report demands it.

### Tier 2: Needs `chardetng` heuristic

The Grex statistical and heuristic stages are aimed at single-byte 8-bit
encodings (the ISO-8859 family, Windows-12xx family, KOI8-R/U) and CJK
multibyte families (Shift-JIS, GB2312, GBK, Big5, EUC-KR). These all map
directly to `chardetng::EncodingDetector`, which is the canonical Mozilla
heuristic shipped as a `no_std` Rust crate by Henri Sivonen. `chardetng`
returns a single `encoding_rs::Encoding` and a confidence flag (`is_tld`
input plus an "any non-ASCII" output). The mapping:

- `chardetng` directly identifies these targets that Grex tries to detect:
  windows-1250, windows-1251, windows-1252, windows-1253, windows-1254,
  windows-1255, windows-1256, windows-1257, windows-1258,
  ISO-8859-2, ISO-8859-7, ISO-8859-8, KOI8-U,
  Shift_JIS, EUC-JP, ISO-2022-JP,
  GBK, GB18030, Big5,
  EUC-KR.
- Decoding is done by `encoding_rs` for every one of these.

This is the recommended runtime path for non-BOM input. The plan is:

1. Read up to 1 MB prefix (`chardetng`'s recommended maximum useful sample
   size) into a `Vec<u8>`.
2. Feed it to `EncodingDetector::feed` with `last = true`.
3. Call `guess(None, true)` to get the `&'static Encoding`.
4. Map the guess to a stable Grexa label (`label` table in
   `grexa-core/src/encoding.rs`).

### Tier 3: Needs ICU or iconv

`chardetng` does **not** detect or distinguish these Grex targets:

- ISO-8859-1, ISO-8859-3, ISO-8859-4, ISO-8859-5, ISO-8859-6, ISO-8859-9,
  ISO-8859-10, ISO-8859-11, ISO-8859-13, ISO-8859-14, ISO-8859-15,
  ISO-8859-16
- KOI8-R
- GB2312 (collapses into GBK in `chardetng`)
- UTF-32 LE and UTF-32 BE without a BOM

For these we will:

- Decode known-label files (when the user supplies a label, or a file extension
  hint matches) via the `encoding_rs` set that overlaps with these
  (windows-1252 covers most ISO-8859-1 byte patterns; `encoding_rs` also
  ships `iso-8859-2..16`, `koi8-r`, `koi8-u`).
- Use `icu_normalizer`/`icu_casemap` only for normalization concerns, not
  detection.
- Hold a Tier-3 escape hatch: optional `iconv` (`encoding_rs_io` plus a
  manual fallback through `libc::iconv` via the `iconv` crate) for the long
  tail. We keep this gated behind a feature flag because Grex's own coverage
  of these encodings is best-effort and depends on
  `CodePagesEncodingProvider` being registered.

Practical recommendation: ship Tier 1 + Tier 2 by default. Treat Tier 3 as a
"explicit label" path, not an auto-detect path. Document the gap in
`docs/linux-decisions.md`.

### Crate Summary

- Tier 1 (BOM): `encoding_rs` (already a dependency).
- Tier 2 (heuristic): `chardetng` for guessing + `encoding_rs` for decoding.
- Tier 3 (ICU/iconv): `encoding_rs` for the broader label set plus an
  `iconv`-backed fallback for the residual long tail.

## Test Port List

The Grex tests below describe the contract Grexa must preserve. Each one maps
to a Rust test the replacement should expose; for non-BOM tests the Rust port
should target the `chardetng` path.

From `Tests/Services/EncodingDetectionServiceTests.cs`:

1. `DetectEncoding_WithUTF8BOM_ReturnsUTF8WithHighConfidence` — BOM prefix
   `EF BB BF` plus ASCII payload returns UTF-8, `HasBom = true`, confidence
   above 0.9, `DetectionMethod` contains `"BOM"`. Rust port: covered by
   `encoding::tests::detect_utf8_bom` and `read_utf8_with_bom_strips_marker`;
   add an explicit confidence assertion once Grexa carries a confidence field.
2. `DetectEncoding_WithUTF16LEBOM_ReturnsUTF16LEWithHighConfidence` — BOM
   prefix `FF FE` returns UTF-16 LE, `HasBom = true`, confidence above 0.9.
   Rust port: extends `detect_utf16_le` / `read_utf16_le_decodes_correctly`.
3. `DetectEncoding_WithUTF8WithoutBOM_ReturnsUTF8WithReasonableConfidence` —
   plain ASCII bytes return UTF-8, `HasBom = false`, confidence above 0.
   Rust port: `detect_plain_utf8`. Add: `chardetng` confirms UTF-8 for ASCII.
4. `DetectEncoding_WithEmptyBytes_ReturnsUTF8WithLowConfidence` — empty
   buffer returns UTF-8, confidence below 0.2, `DetectionMethod` contains
   `"Empty"`. Rust port: new test, expect Grexa to short-circuit and report
   UTF-8 with no decoder error.
5. `DetectEncoding_WithNullBytes_ReturnsUTF8WithLowConfidence` — null buffer
   returns UTF-8 low-confidence. Rust port: Rust signatures take `&[u8]`, so
   the equivalent is `detect_from_bytes(&[])`.
6. `DetectFileEncoding_WithValidFile_ReturnsEncoding` — round-trip via a
   temp file. Rust port: extend `read_utf8_with_bom_strips_marker` to also
   call `detect_from_path`.
7. `DetectFileEncoding_WithNonExistentFile_ReturnsUTF8Fallback` — Grex
   swallows IO errors and reports UTF-8 with `DetectionMethod`
   `Error reading file: ...`. **Behavior change for Grexa**: `detect_from_path`
   currently returns the `io::Error`; we should keep that for library callers
   and add a `detect_from_path_or_default` that mirrors Grex for UI callers.
   Port the test against that wrapper.
8. `DetectEncoding_WithFileNameHint_CanUseHintForDetection` — accepts a file
   name hint without throwing. **Not ported**: the hint stage is `0.1` of the
   weight and is largely cosmetic in Grex; Grexa relies on `chardetng` instead.
9. `DetectEncoding_WithASCIIOnlyText_ReturnsUTF8` — pure ASCII returns
   UTF-8. Rust port: equivalent to `detect_plain_utf8`.
10. `DetectEncoding_WithTextContainingNullBytes_MayDetectUTF16` — alternating
    null bytes from `Encoding.Unicode.GetBytes("Hello")` should produce a
    sensible result. Rust port: feed the same bytes into the BOM-less
    `chardetng` path and assert it returns UTF-16 LE.
11. `EncodingDetectionResult_Constructor_ClampsConfidenceBetween0And1` — only
    meaningful if Grexa adds a confidence type. Track as a TODO.
12. `DetectEncoding_WithWindows1252Text_CanDetectEncoding` — accent-bearing
    Windows-1252 bytes should return a non-null encoding. Rust port: assert
    `chardetng` returns `windows-1252` (or one of its aliases).

Additional tests Grexa should add that Grex does not exercise:

- UTF-32 LE BOM is detected ahead of UTF-16 LE BOM (Grex regression).
- UTF-8 BOM produces the label `UTF-8 BOM` distinct from plain `UTF-8`.
- A KOI8-R sample is decoded when the user supplies the label explicitly,
  even though `chardetng` does not auto-detect it.
- A 50 MB synthetic file is detected from the first 1 MB prefix and never
  reads more than the cap.

## Non-Applicable Behavior

- The Grex `TabViewModel.DetectFileEncoding` short-circuit for `\\wsl$` paths
  is a Windows-only concern. Grexa native paths use the standard detection
  path.
- The fallback when `EncodingDetectionService` cannot register
  `System.Text.Encoding.CodePages` is moot on Linux; Grexa starts from
  `encoding_rs`'s full label set.
- `Encoding.EncodingName`'s locale-dependent display strings (`Unicode (UTF-8)`,
  `Western European (Windows)`) are intentionally dropped in favour of the
  short Grexa labels.
