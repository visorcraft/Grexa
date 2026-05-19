<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Grexa v1.0.0

**Released:** 2026-05-19

The 1.0 line. Grexa is feature-complete against the Grex parity
matrix on Linux and the on-disk surface is now stable. Everything
that shipped in v0.3 carries forward unchanged; this release is a
deliberate promotion of that body of work to a 1.0 commitment,
with no new behavioral changes beyond a small About-page label.

## Why 1.0 now

The v0.2 release closed the original GUI parity gap. The v0.3
release closed every audit-flagged polish item against the v0.2
build and added the missing Wayland taskbar integration and
Settings auto-save. v0.3 has now been running as the daily driver
through the release window with no schema migrations, no
regression escapes, and no outstanding parity items in
[docs/feature-parity.md](feature-parity.md).

That earns the 1.0.

## What's stable from this point

- **On-disk schemas.** `settings.json`, `recent_paths.json`,
  `search_history.json`, `profiles.json`, and
  `replace-journal.json` are the long-term contract. Future
  additive fields will default safely on load; removals or
  rekeyings will ship with a migration step recorded in
  [docs/reference.md](reference.md).
- **CLI flags.** Every `grexa-cli` flag named here will continue to
  parse with the same semantics. Removals require a deprecation
  cycle.
- **Localization keys.** Existing Fluent keys in
  `crates/grexa-i18n/locales/en/grexa.ftl` will not be renamed in
  the 1.x line. New keys land additively.
- **cxx-qt bridge surface.** QObject names, property names, and
  signal names exposed to QML are the public contract for plugin
  and theme work. See [docs/gui-design.md](gui-design.md).

## What changed since v0.3

Nothing behavioral. One UI label:

- **About page.** The third card at the bottom of the About page
  is now titled "Licenses & Credits" (was "Third-party credits").
  Reflects that the linked viewer is the canonical source for the
  GPL-3.0-only license text in addition to the dependency
  attribution list.

The full v0.3 changelog still applies: see
[docs/release-notes-0.3.0.md](release-notes-0.3.0.md) for the
per-tab isolation work, the responsive Search toolbar, the
Wayland taskbar icon, Settings auto-save, locale-aware
pluralization through Fluent, the History/Profiles filter rows,
and the rest.

## Verification

- `cargo test --workspace`: **300 passing** across 8 crates,
  same suite as v0.3.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `python3 scripts/check_locale_sync.py`: en / de / ja in sync.
- KDE Plasma 6 Wayland smoke on the same setup that signed off
  v0.3: clean. No new warnings vs v0.3.

## Upgrade

Drop-in replacement for v0.3.x. No config or data conversion.
Older `settings.json` from v0.2 still loads unchanged.

Packaging recipes for Flatpak, AppImage, Arch / CachyOS, Fedora,
Debian, and openSUSE all bump their `Version:` / `pkgver` /
`changelog` entries to `1.0.0` and install the new metainfo
release block automatically.

## Known limits

Carried forward from v0.3 — both targeted for the 1.x line:

- **Theme palette swap.** Custom theme variants still bias
  `DesignTokens` for surface tints and separators; a full Qt
  palette swap requires `KColorSchemeManager` and is not yet
  bridged through cxx-qt-lib.
- **Diagnostic redaction.** `$HOME` → `~` substitution is in
  place; structured-field redaction for paths inside tracing
  span fields is still a future task.

New in this release:

- **Flatpak `SearchPage` warning.** Under the KDE Platform 6.10
  flatpak runtime (Qt 6.10), launching the GUI emits one
  cosmetic warning to stderr:
  `qrc:/qt/qml/com/visorcraft/Grexa/qml/Main.qml:507:35: QML
  SearchPage: Created graphical object was not placed in the
  graphics scene.` This is a spurious diagnostic emitted when
  Qt 6.10's QML compiler resolves a `Component { SearchPage {} }`
  template declaration — the inner instance is correctly created
  on demand when the user navigates to the page. The host Qt
  doesn't print it. Filed upstream; the deb / rpm / Arch /
  AppImage builds run on the host Qt and are not affected.

Bug reports: <https://github.com/visorcraft/grexa/issues>.
