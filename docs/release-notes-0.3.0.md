<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Grexa v0.3.0

**Released:** 2026-05-18

Polish + responsiveness pass on top of the v0.2 GUI-parity release.
Every item here is a direct response to real-use feedback against
the v0.2 build — no fictional roadmap entries.

## Highlights

### True per-tab result isolation

Switching tabs now preserves each tab's full result buffer, not
just its form fields. Each QML tab carries a stable monotonic
`tabId`; the `SearchController` keeps a Rust-side
`HashMap<i32, TabSnapshot>` so the row buffer, counters, status
text, and within-filter state all survive a tab round-trip.

The view projection (`visible`) is rebuilt on restore, so flipping
result-mode or within-filter while a different tab is active
doesn't leave stale indices behind. Closing a tab drops its
snapshot.

### Responsive action toolbar

The Search-page action toolbar (Save profile…, Export…, Replace…,
Stop, Clear, AI assist) used to clip off the right edge when the
window narrowed below ~1100px. It's now a `QtQuick.Flow` — buttons
wrap to additional rows instead of disappearing. The path picker
gained `Layout.minimumWidth: 140` so the SearchBar's primary
Search button stays reachable at narrow widths.

Verified at 800px (3-row toolbar) and 2560px (single-row toolbar)
under KDE Plasma Wayland.

### Pink-gecko taskbar icon

The Wayland compositor was resolving Grexa's window to the
generic "X" placeholder because the running binary didn't set its
`app_id` and no desktop file was installed.

- `QGuiApplication::set_desktop_file_name("io.visorcraft.Grexa")`
  is called before any window is shown, tying every Grexa window
  to the canonical app id.
- `ensure_user_desktop_integration()` at startup writes the
  embedded `.desktop` + every hicolor PNG (16..512px) +
  scalable SVG into `$XDG_DATA_HOME/applications` and
  `$XDG_DATA_HOME/icons/hicolor`. Idempotent — only writes when
  missing, so a packaged install on `/usr/share` still wins.
  After a fresh install we ping `kbuildsycoca6`,
  `update-desktop-database`, and `gtk-update-icon-cache` so the
  icon appears without a session restart.

### Settings auto-save

Every checkbox, dropdown, spinbox, and text field on the Settings
page now commits to disk the moment its value changes. The Apply
button is gone — it was a footgun ("did I save?"). A small
"Saved" pill fades in for ~1.4s after each commit as visual
confirmation. The Reload button stays — it's still useful when
`settings.json` was edited externally.

TextFields commit on `onEditingFinished` (focus loss / Enter)
rather than every keystroke, so `settings.json` doesn't thrash
while the user is mid-edit. The qproperty still updates per
keystroke for any consumer that reads it live.

### UX cleanup — toggle semantics + idempotent re-clicks

A handful of small affordances surprised users in v0.2; v0.3
fixes them:

- **Filters button** is now `checkable: true` and the drawer's
  `onClosed` / `onOpened` callbacks sync `checked` back. A second
  click closes the drawer; Esc / click-outside un-presses the
  button.
- **Esc shortcut** is now gated on `enabled: busy` — it only
  cancels a running search. With no search in flight, Esc falls
  through to Qt's default popup/drawer close handling.
- **Export… menu** toggles on second click (was a silent no-op
  because `Menu.popup()` does nothing when already visible).
- **Tab click on the active tab** is now a no-op (previously
  cancelled an in-flight search as a hidden side effect of
  `persistActiveTab()`).
- **Sidebar nav click on the current page** is now a no-op
  (previously tore down + rebuilt the page, losing typed form
  state and scroll position).
- **FlagChip** (`.* ` regex + `Aa` case-sensitive on the
  SearchBar) no longer breaks its parent's declarative binding
  by imperatively writing its own `checked` state.

## Verification

- `cargo test --workspace`: **291 passing** across 8 crates.
- `cargo clippy --workspace --release -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- Offscreen smoke (`QT_QPA_PLATFORM=offscreen`): clean.
- Live KDE Plasma 6 Wayland verification at 800px and full
  width.

## Schema migration

No new `DefaultSettings` fields. Older `settings.json` files load
unchanged. v0.2 → v0.3 needs no data migration.

## Known limits

- **Theme palette swap.** Custom theme variants still bias
  `DesignTokens` for surface tints and separators but a full Qt
  palette swap requires `KColorSchemeManager` which isn't
  bridged through cxx-qt-lib yet. v0.4 target.
- **Diagnostic redaction** still only covers `$HOME` →`~`
  substitution. Structured-field redaction (paths in tracing
  span fields) is a v0.4 enhancement.

## Upgrade

Drop-in replacement for v0.2.x. No config or data conversion.
Package recipes for Flatpak, AppImage, Arch, Fedora, Debian, and
openSUSE all install the new icons + metainfo entry automatically.

Bug reports: <https://github.com/visorcraft/grexa/issues>.
