# Grex Scripts Audit

This document records the helper scripts that ship with Grex and the decision
for each one as Grexa diverges into a Linux-native Rust workspace. Scripts that
encode product-level workflows (localization, version stamping) must be ported
to Grexa's tooling, while scripts that exist only to drive `.resw`/UWP-specific
formats can be retired.

Source evidence:

- `Scripts/add_localization_entry.py`
- `Scripts/remove_localization_entry.py`
- `Scripts/test_add_localization_entry.py`
- `Scripts/test_remove_localization_entry.py`
- `Scripts/generate_translation_status.py`
- `Scripts/translate_remaining_entries.py`
- `Scripts/update_version.py`

There are no top-level `*.sh`, `*.ps1`, `*.bat`, or non-`Scripts/` Python
helpers in `/work/repos/visorcraft/grex/`. The repository delegates build
orchestration entirely to MSBuild/Visual Studio, so no shell or PowerShell
glue exists. Decision rows that mention PowerShell are therefore by absence,
not by replacement.

For each entry below: `port` means the workflow must exist in Grexa under a
different implementation; `replace-with-just-target` means the workflow becomes
a `Justfile` recipe (possibly delegating to a small script); `drop` means the
workflow is Grex-specific (UWP `.resw`, `Package.appxmanifest`,
`AboutView.xaml.cs`) and does not survive the port; `replace-with-shell` would
mean a one-shot bash/fish rewrite (no Python entries reach that classification
here because every survivor needs richer logic).

## Inventory

| Path | Language | Lines | Decision |
| --- | --- | ---: | --- |
| `Scripts/add_localization_entry.py` | Python 3 | 168 | port |
| `Scripts/remove_localization_entry.py` | Python 3 | 85 | port |
| `Scripts/test_add_localization_entry.py` | Python 3 (unittest) | 216 | port |
| `Scripts/test_remove_localization_entry.py` | Python 3 (unittest) | 216 | port |
| `Scripts/generate_translation_status.py` | Python 3 | 192 | port |
| `Scripts/translate_remaining_entries.py` | Python 3 (`googletrans`) | 705 | port (rewrite) |
| `Scripts/update_version.py` | Python 3 | 216 | drop (replace-with-just-target) |

Total: 7 scripts, 1798 lines.

## `Scripts/add_localization_entry.py`

- Path: `/work/repos/visorcraft/grex/Scripts/add_localization_entry.py`
- Language: Python 3 (stdlib only, `xml.etree.ElementTree`)
- Lines: 168

What it does: For every `Strings/<lang>/Resources.resw` under the project root,
parses the XML, refuses to add a duplicate `name`, and appends a new `<data>`
element with attributes `name="<key>"` and `xml:space="preserve"`. The new
entry gets a `<value>` child with the supplied text and a `<comment>` child
whose text is `status:complete` for `en-US` and `status:incomplete` for every
other locale. After write-out it reparses and reindents the file with
two-space indentation by walking the tree.

Windows dependence: None. Pure stdlib, no calls to `pri`, `priconfig`,
`MSBuild`, or PowerShell. The `.resw` schema itself is a UWP / WinUI convention
but the script does not require any Microsoft tooling to manipulate it.

Decision: `port`. Grexa still needs a way to add a translatable string in one
shot across every locale shipping with the app. The output format will not be
`.resw` though, so this is a rewrite rather than a copy.

Linux equivalent:

- New script: `scripts/i18n/add-entry.py` (Python 3 stdlib, no third-party
  deps) under `/work/repos/visorcraft/grexa/scripts/i18n/`.
- The script operates on whatever locale storage Grexa chooses (`gettext`
  `.po`/`.pot`, `fluent` `.ftl`, or a flat JSON/YAML keyed by locale). The
  current `linux-decisions.md` does not pin this; the audit treats the choice
  as pending, but the entry-point CLI signature stays `add-entry <key>
  <value>`.
- Wired into the workspace via `just i18n-add KEY VALUE` in
  `/work/repos/visorcraft/grexa/Justfile`.

## `Scripts/remove_localization_entry.py`

- Path: `/work/repos/visorcraft/grex/Scripts/remove_localization_entry.py`
- Language: Python 3 (stdlib only, `xml.etree.ElementTree`, `glob`)
- Lines: 85

What it does: Globs `Strings/*/Resources.resw`, parses each, runs the XPath
`.//data[@name="<entry>"]` to find matching `<data>` nodes, and removes them.
If any matches were found, the tree is written back with `tree.write(..., 
encoding='utf-8', xml_declaration=True)`. Prints a per-file removal log and a
final count. Refuses to run with an empty key.

Windows dependence: None. The script is content-agnostic to the locale list
and does not invoke any Microsoft toolchain.

Decision: `port`. Symmetric with `add_localization_entry.py`; both go together
or not at all.

Linux equivalent:

- New script: `scripts/i18n/remove-entry.py` under
  `/work/repos/visorcraft/grexa/scripts/i18n/`. Whatever format the
  add-counterpart writes, this script must inversely strip the same key from
  every locale file.
- `just i18n-remove KEY` recipe in `/work/repos/visorcraft/grexa/Justfile`.

## `Scripts/test_add_localization_entry.py`

- Path: `/work/repos/visorcraft/grex/Scripts/test_add_localization_entry.py`
- Language: Python 3 (`unittest`)
- Lines: 216

What it does: Standard `unittest.TestCase` suite. The unit tests build a
temporary `<test_dir>/en-US/` and `<test_dir>/fr-FR/` with a hand-rolled
`Resources.resw` containing a single `ExistingKey` entry, then call
`add_entry_to_resw` directly. Coverage:

- `en-US` writes get `status:complete`, others get `status:incomplete`.
- Duplicate keys return `(False, "already exists")` and skip the file.
- `get_strings_directory()` resolves to a path whose `.name` is `Strings`.
- Round-trips XML special characters (`<`, `&`, `"`).
- Adding two new keys in succession preserves both plus the seed key.

It also has integration tests that read the real `Strings/en-US/Resources.resw`
to confirm `AppName`, `SearchTab`, `SettingsTab`, and the previously-added
`StopButton` key are present. Those tests assume Grex's exact key inventory.

Windows dependence: None at the Python level. The integration tests bind to
specific `.resw` keys that exist only in Grex.

Decision: `port`. The unit half is the contract documentation for whatever
`add-entry` Grexa ships; the integration half is rewritten against Grexa's own
canonical key list.

Linux equivalent:

- `scripts/i18n/tests/test_add_entry.py` (pytest or unittest, stdlib only).
- Hooked in CI via `just i18n-test`, which invokes
  `python3 -m unittest discover scripts/i18n/tests` (or `pytest` if Grexa
  picks pytest as the Python harness).
- The Grex-specific integration cases (`StopButton`, etc.) are dropped; new
  cases assert against keys Grexa actually ships in its catalog.

## `Scripts/test_remove_localization_entry.py`

- Path: `/work/repos/visorcraft/grex/Scripts/test_remove_localization_entry.py`
- Language: Python 3 (`unittest`)
- Lines: 216

What it does: Mirror of `test_add_localization_entry.py`. Builds a temp tree
with two locales seeded by an inline `Resources.resw` containing three keys.
Verifies:

- Removing an existing key clears it from every locale file and reports the
  removal count.
- Removing a missing key reports 0 and leaves the file untouched.
- Removing one key does not disturb the others (count decreases by exactly 1).
- Multi-locale removal: both `en-US` and `fr-FR` lose the entry.
- `get_strings_directory()` ends in `Strings`.
- Dotted key names (`MyButton.Content`) survive the XPath round-trip.

Integration tests confirm `AppName`, `SearchTab`, `SettingsTab` are still
present after running the removal logic in a known-good state.

Windows dependence: None.

Decision: `port`. Same justification as the add-test counterpart.

Linux equivalent:

- `scripts/i18n/tests/test_remove_entry.py` under the same harness as the add
  tests.

## `Scripts/generate_translation_status.py`

- Path: `/work/repos/visorcraft/grex/Scripts/generate_translation_status.py`
- Language: Python 3 (stdlib only, `xml.etree.ElementTree`)
- Lines: 192

What it does: Walks `Strings/`, treats `en-US` as the reference, then for each
other locale counts `<data>` entries by the text of their `<comment>` element.
The recognized statuses are `status:complete`, `status:error*`, and
`status:incomplete` (default when no comment exists). A whole locale is
classified `complete` (every entry complete), `error` (any error entries), or
`incomplete` otherwise. Output is plain text: a summary line of counts plus
two ranked tables (top 10 locales with fewest entries remaining; top 10 with
the most errors). Nothing is written to disk.

Windows dependence: None.

Decision: `port`. Grexa keeps the "how complete is each locale" dashboard
because the catalog ships ~100 locales and CI needs to surface drift.

Linux equivalent:

- `scripts/i18n/status.py` under `/work/repos/visorcraft/grexa/scripts/i18n/`.
- Output stays plain text by default; add a `--json` flag so a future GitHub
  Actions job (or a `cargo xtask`) can ingest the summary.
- `just i18n-status` recipe in the `Justfile`.
- If Grexa adopts `gettext` instead of an `.resw`-shaped XML, the per-entry
  status check switches from "`<comment>` element text" to "is the `msgstr`
  empty or marked `fuzzy`" — the report format stays the same.

## `Scripts/translate_remaining_entries.py`

- Path: `/work/repos/visorcraft/grex/Scripts/translate_remaining_entries.py`
- Language: Python 3 with `googletrans==4.0.0rc1`
- Lines: 705

What it does: Largest script in the tree. Walks every non-`en-US` locale, and
for each entry that still matches the English value exactly (and is not a
hard-coded technical key such as `AppName`, `KBComboBoxItem.Content`,
`MBComboBoxItem.Content`, `GBComboBoxItem.Content`, `URLPresetButton.Content`)
asks `googletrans` for a translation. A 78-row `LANG_CODE_MAP` translates
Grex's BCP-47-ish folder names (`fr-FR`, `pt-BR`, `zh-CN`, `jv-Latn-ID`, ...)
into the short codes `googletrans` expects. Failed locales (e.g. Fijian
`fj-FJ`) are mapped to `en` so they get short-circuited and their entries are
marked `status:error:permanent`.

Beyond translating, the script normalizes the `.resw` files: it ensures every
entry has a `<comment>` (defaulting to `status:incomplete` for non-English),
detects entries already translated (value diverges from English) and stamps
them `status:complete`, batches writes every 5 translations, backs off on
rate limits (`AttributeError` from `googletrans`, HTTP 429), and reformats
output so `<comment>` ends on its own line (a regex post-pass:
`</comment>\s*</data>` -> `</comment>\n  </data>`).

Windows dependence: None at the Python layer. The script does not call PowerShell,
`pri`, or `priconfig`; it manipulates XML directly. It does require the
`googletrans` package, which is an unofficial scrape of Google Translate and
is widely known to break with API changes.

Decision: `port`, but as a rewrite. Three reasons to rewrite rather than copy:

1. `googletrans==4.0.0rc1` is unmaintained and the script literally hard-codes
   workarounds for its `AttributeError` failure mode. Grexa should not inherit
   that dependency.
2. The 78-row locale map is `.resw`-folder shaped; Grexa's locale identifiers
   will be whatever the chosen i18n format prescribes (typically POSIX-style
   `fr_FR` for `gettext`, or `fr-FR` for ICU).
3. The technical-key allow-list (`AppName`, `KB/MB/GB`, `Tab`) is Grex's
   product vocabulary and not Grexa's.

Linux equivalent:

- `scripts/i18n/translate.py` under `/work/repos/visorcraft/grexa/scripts/i18n/`.
- Replace `googletrans` with one of:
  - `argos-translate` (offline, open-source, packaged on Linux).
  - The DeepL or LibreTranslate HTTP API behind a `GREXA_TRANSLATE_API_KEY`
    env var (matches Grexa's existing pattern of optional remote services).
  - A "no-op" mode that emits a TODO list and stops, intended for human
    translators.
- Read the translation backend from an environment variable so CI can run the
  status pass without ever performing network calls.
- `just i18n-translate` recipe in the `Justfile`, plus a `--dry-run` flag that
  only emits the list of entries needing attention.
- Keep the safety invariant: the script must never overwrite a translation
  whose value already differs from `en-US`. That invariant is the one piece
  of logic worth carrying over verbatim.

## `Scripts/update_version.py`

- Path: `/work/repos/visorcraft/grex/Scripts/update_version.py`
- Language: Python 3 (stdlib only, `re`)
- Lines: 216

What it does: Accepts a `X.Y` version string and edits four hard-coded files
via regex-on-bytes:

- `Controls/AboutView.xaml.cs` — replaces the literal
  `VersionTextBlock.Text = "Version X.X";`
- `Package.appxmanifest` — replaces `Version="X.X.0.0"` on the
  `<Identity>` element
- `Properties/AssemblyInfo.cs` — replaces `AssemblyVersion`,
  `AssemblyFileVersion`, and `AssemblyInformationalVersion` attributes
- `app.manifest` — replaces `<assemblyIdentity version="X.X.0.0"`

After a successful pass it prints the suggested follow-up `git add`,
`git commit`, `git tag`, and `git push --tags` commands but does not execute
them.

Windows dependence: Conceptually total. Every file the script touches is a
Microsoft packaging or WinUI artifact:

- `Package.appxmanifest` — UWP/MSIX manifest, only meaningful to MSBuild and
  the Windows packaging tooling.
- `Controls/AboutView.xaml.cs` — WinUI code-behind that does not survive the
  Avalonia / GTK port.
- `Properties/AssemblyInfo.cs` — .NET Framework / .NET assembly metadata.
- `app.manifest` — Win32 side-by-side assembly manifest, irrelevant on Linux.

Decision: `drop` (replace-with-just-target). The workflow ("bump the version
in one place and let the build see it everywhere") is retained, but the
implementation is unrecognizable.

Linux equivalent:

- A `just version <X.Y.Z>` recipe in `/work/repos/visorcraft/grexa/Justfile`
  that:
  1. Edits `[workspace.package].version` in
     `/work/repos/visorcraft/grexa/Cargo.toml` via `cargo set-version`
     (`cargo install cargo-edit`) — Cargo's workspace inheritance then
     propagates to every member crate without a regex sweep.
  2. Stamps `/work/repos/visorcraft/grexa/packaging/io.visorcraft.Grexa.metainfo.xml`
     with a new `<release version="X.Y.Z" date="YYYY-MM-DD"/>` row. This is
     the AppStream metadata Flatpak and AppImage consume.
  3. Optionally regenerates the manpage (`just manpage`) and shell
     completions (`just completions`) so the version embedded in `--version`
     output matches.
- No equivalent of `Package.appxmanifest`, `AboutView.xaml.cs`, or
  `app.manifest` exists, so the four-file fan-out collapses to two files.

## Follow-up Scripts Grexa Should Gain

Grex relied on Visual Studio for everything that was not localization-related.
Linux-native tooling has to fill the gap. The follow-ups below are not in scope
for the script audit per se, but they are the natural home for work that Grex
left implicit:

- `scripts/extract-locale.sh` (or a `just i18n-extract` recipe). Walks the
  Rust crates and the Avalonia/Slint/egui XAML-equivalent, runs `xgettext` (or
  the equivalent for the chosen i18n stack), and refreshes the master locale
  template. Grex never had this because Visual Studio's resw editor took the
  role.
- `scripts/fixtures/generate-search-corpus.sh`. Produces the synthetic
  directory trees the `SearchService` integration suite uses. Today
  `/work/repos/visorcraft/grex/IntegrationTests/SearchWorkflowTests.cs`
  manufactures these inline via `Directory.CreateDirectory(...)`. A standalone
  generator keeps the fixtures reproducible outside the test runner and lets
  the Rust port reuse the same content.
- `scripts/smoke/cli-smoke.sh`. Drives `cargo run -p grexa-cli` against the
  generated corpus, captures stdout, and diffs against a golden file. Grex
  had no CLI smoke test; the new `Grex.Cli` project has one only inside
  `Tests/Grex.Cli.Tests` and depends on .NET.
- `scripts/smoke/gui-smoke.sh`. Launches the Grexa GUI under
  `WAYLAND_DISPLAY` / a virtual X server, screenshots the main window, and
  fails CI on a hash mismatch. The equivalent on Grex was `UITests/` (WinAppDriver).
- `scripts/packaging/build-flatpak.sh` and
  `scripts/packaging/build-appimage.sh`. Thin wrappers over
  `flatpak-builder` and `appimagetool` that reference
  `/work/repos/visorcraft/grexa/packaging/flatpak/` and
  `/work/repos/visorcraft/grexa/packaging/appimage/`. They replace the
  `msix`/`appxmanifest` packaging that `Package.appxmanifest` implied.
- `scripts/release/cut-release.sh`. Runs `just version`, regenerates
  manpages, regenerates completions, runs `just ci`, builds Flatpak +
  AppImage, and tags the commit. Subsumes the trailing instructions from
  `update_version.py` (`git add`, `git tag v<x.y>`, `git push --tags`).

Suggested layout under `/work/repos/visorcraft/grexa/scripts/`:

```
scripts/
  i18n/
    add-entry.py
    remove-entry.py
    status.py
    translate.py
    tests/
      test_add_entry.py
      test_remove_entry.py
  fixtures/
    generate-search-corpus.sh
  smoke/
    cli-smoke.sh
    gui-smoke.sh
  packaging/
    build-flatpak.sh
    build-appimage.sh
  release/
    cut-release.sh
```

All recipes (`i18n-add`, `i18n-remove`, `i18n-status`, `i18n-translate`,
`i18n-test`, `version`, `fixtures`, `smoke`, `release`) live in
`/work/repos/visorcraft/grexa/Justfile` so contributors discover them via
`just --list`. Python scripts target Python 3.11 stdlib-only by default;
optional dependencies (a translator backend) are isolated behind an env-var
toggle so the default checkout never needs `pip install` to lint or test.
