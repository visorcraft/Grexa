# Accessibility Plan

PLAN.md phase 15 lines 453-454 require an accessibility pass plus
AT-SPI roles for custom result tables, row actions, command buttons,
filter controls, and dialogs. This doc records what Grexa's
core/CLI/AI/containers crates contribute and what the GUI will own.

## Core / CLI layer (already shipping)

- **Structured progress events.** `ProgressEvent::FileScanned`,
  `FileSkipped`, `Match` carry typed data the screen-reader-friendly
  GUI can announce as "Scanning file …" / "10 matches in 5 files so far".
- **Structured tracing logs.** `tracing::info!(matches, files_matched,
  elapsed_ms, "search completed")` lets the GUI mirror state into
  ARIA-live equivalents.
- **CLI text output is line-oriented.** `grexa-cli` prints one match
  per line in `path:line:col:content` form, which is the canonical
  shape screen-readers and terminal accessibility hooks expect.
- **Documented exit codes** (0 / 1 / 2) so terminal users on assistive
  tech don't have to read body text to know the search status.
- **Localized strings.** `crates/grexa-i18n` ships catalogs whose
  keys map 1:1 to UI strings; no English-only labels leak through.

## GUI layer (Phase 4 / 18 deliverables)

The QML shell must:

1. Set `Accessible.role` and `Accessible.name` on every clickable
   control (`Search`, `Stop`, `Replace`, `AI`, `Reset`, `Filter Options`,
   `Profiles`, `History`, `Export`, every column header, every result
   row).
2. Wire `Accessible.description` to the localized tooltip text from
   the Fluent catalog.
3. Set `Accessible.focusOnPress = true` so keyboard navigation lands
   in the right place after each command-strip action.
4. Use Kirigami's `BasicListItem` / `ListItem` controls for results,
   which already announce row content correctly under AT-SPI.
5. Surface live regions for status text: `Accessible.announcement`
   triggered on every `ProgressEvent::FileScanned` batch.
6. Mark non-decorative icons with `Accessible.name`; mark purely
   decorative icons with `Accessible.ignored = true`.
7. Preserve the keyboard shortcuts from Grex: Enter to search, Enter
   to replace from the replacement input, Space for preview, Escape
   to close preview / dialogs, F1 for About, double-click → Enter to
   open result. PLAN.md phase 4 line 254 covers the keyboard surface.

## Settings

- High-contrast theme support: `DefaultSettings.theme_preference`
  already enumerates the eight high-contrast variants Grex ships
  (`BlackKnight` … `Vibes`) under the same integer encoding so
  imports round-trip. The GUI maps them to QQC2 Desktop Style
  palettes.
- Reduced-motion: when `gtk-enable-animations = 0` on GNOME or
  `PlasmaThemeOption Animate = false` on KDE, the GUI must skip
  filter-pane / tab-switch / AI-arrival transitions. Tracked as a
  `cxx-qt` controller hook in Phase 18.

## CI coverage

- `cargo test` runs the property tests + golden tests + status-string
  formatting tests; this catches regressions in the strings layer.
- GUI accessibility is verified under `QT_ACCESSIBILITY=1` with
  Qt's automated tester. CI sets the env var on the offscreen platform
  before running `qmltestrunner`.
- `desktop-file-validate` and `appstreamcli validate` already block
  desktop-entry regressions that would hide Grexa from accessibility
  listings.

## Manual checklist (before each release)

- [ ] Run Orca on KDE Plasma against the live app; verify every
  command-strip button announces correctly.
- [ ] Verify keyboard-only flow: tab between every focusable control,
  trigger every keyboard shortcut without using the pointer.
- [ ] Verify high-contrast theme inversion looks reasonable.
- [ ] Verify fractional scaling (125 / 150 / 200%) doesn't crop
  control labels.
- [ ] Verify the CLI's `--quiet` and exit codes match documented
  behavior.

This doc is intentionally short — the heavy lifting lives in the GUI
phase. The point is to capture today's promises so the GUI authors
have a contract to test against.
