<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Grexa v0.2.0

**Released:** 2026-05-18

This release closes the GUI bridge wiring gap flagged in the v0.1
post-release audit. Grexa now exposes every Grex-equivalent
capability through the Qt 6 / Kirigami shell; the Rust core has
been at feature parity since v0.1.

## Highlights

- **Folder picker.** Native `QtQuick.Dialogs.FolderDialog` —
  Breeze on KDE, XDG portal under Wayland / Flatpak. Browsed paths
  land in the recent-paths store.
- **Container search.** Target dropdown surfaces detected Docker
  and Podman (rootless + rootful) runtimes with their live
  container lists. Discovery runs off-thread; the GUI never
  blocks on `docker ps` / `podman ps`.
- **Filter drawer.** Every `SearchOptions` filter is now toggleable
  from the Search page: gitignore, hidden, binary, system files,
  symlinks, subfolder recursion, match-files glob, exclude-dirs
  glob.
- **Content / Files toggle.** Per-search segmented control with
  files-mode deduplication in the result model.
- **Replace flow.** Replace button → confirmation dialog (or skip
  when `replace_confirm` is off) → atomic-rename pipeline →
  auto-flip to Files mode. Residual replace journal surfaces on
  startup so a killed run can be reviewed or dismissed.
- **Result row context menu.** Eight actions: Preview, Open in
  editor, Reveal in file manager, Copy path / file name / relative
  path / line content / path:line. Reveal uses
  `org.freedesktop.FileManager1.ShowItems` over D-Bus; copy uses
  `wl-copy` / `xclip` per session type.
- **Search-within-results.** Live substring or regex filter that
  narrows the visible rows without re-running the search.
- **Sortable columns.** Click Path / Line / Match to sort; click
  again to flip direction.
- **In-session tabs.** Mailspring-style pill tab bar above the
  search bar. Ctrl+T opens a new tab; Ctrl+W closes the active
  one. Each tab carries its own path / term / flags / within-filter.
- **Export.** Save the visible result set to CSV, JSON, or
  Markdown from the toolbar Export menu.
- **History page.** Every completed search is listed and can be
  re-opened (populates the form; user still has to hit Search to
  re-run).
- **Profiles page.** Save the current form as a named search
  preset; reopen later from the Profiles nav entry.
- **Editor preset + custom template.** Settings → Editor exposes
  the 9 built-in editor presets plus a custom-command template
  with `{path}` / `{file}` / `{line}` substitution. The custom
  template wins when set, regardless of which preset is selected.
- **Accessibility.** Reduced motion (Settings → Accessibility)
  zeros every transition duration so the app animates instantly.
  High contrast biases separator alphas up.
- **Privacy.** `privacy_redact_paths` wraps the
  `grexa-gui.log` writer with a `$HOME` → `~` redactor so
  diagnostics are safe to share.
- **Single-instance.** Advisory `flock` on
  `$XDG_RUNTIME_DIR/grexa/grexa.lock`; a second invocation logs
  and exits without spawning a duplicate window.
- **Notifications.** Long searches (≥4s, ≥1 match) and every
  replace completion fire a `notify-send` desktop toast.
- **Keyboard shortcuts.** F1 → About, Ctrl+, → Settings, Ctrl+1..4
  page navigation, Ctrl+T / Ctrl+W tab management, Esc → cancel
  running search, Ctrl+Q → quit, Space on a focused result row
  → context preview.

## Verification

- `cargo test --workspace`: **291 passing** across 8 crates.
- `cargo clippy --workspace --release -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `cargo deny check`: clean.
- Offscreen smoke (`QT_QPA_PLATFORM=offscreen`): clean.
- Live KDE Plasma 6 Wayland verification by independent audit
  agent against the running binary.

## Schema migration

`DefaultSettings` gained nine new fields with `#[serde(default)]`
so older `settings.json` files parse cleanly with sensible
defaults:

- `editor_preset: u8` — default 8 (XdgOpen).
- `editor_custom_command: String` — empty by default.
- `replace_confirm: bool` — true (Grex behavior).
- `replace_show_journal_on_startup: bool` — true.
- `privacy_redact_paths: bool` — false.
- `accessibility_reduced_motion: bool` — false.
- `accessibility_high_contrast: bool` — false.

No data migration is required; previous v0.1 settings load
without modification.

## Known limits

- **Per-tab result isolation.** Tabs share a single
  `SearchController` row buffer — switching tabs reloads the form
  but doesn't restore the previous tab's results. True per-tab
  state requires multiple controllers or a tabbed model and is
  tracked for v0.3.
- **Theme palette swap.** Custom themes from the 12-variant
  `ThemePreference` enum (Light, Dark, GentleGecko, BlackKnight,
  Diamond, Dreams, Paranoid, RedVelvet, Subspace, Tiefling,
  Vibes) are honored by `DesignTokens` for surface tints and
  separators, but a full Qt palette swap requires
  `KColorSchemeManager` which isn't bridged through cxx-qt-lib
  yet. v0.3 target.
- **Diagnostic log redaction** only covers `$HOME` →`~`
  substitution today; structured-field redaction is a v0.3
  enhancement.

## Upgrade

Replace the v0.1 binary in place; no config or data conversion
needed. Package recipes for Flatpak, AppImage, Arch, Fedora,
Debian, and openSUSE all install the new icons +
metainfo entry automatically.

Bug reports: <https://github.com/visorcraft/grexa/issues>.
