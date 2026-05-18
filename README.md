# Grexa

> Fast, precise file content search for Linux. A Linux/Qt port of
> [Grex](https://github.com/visorcraft/grex), built ground-up for
> KDE Plasma.

Grexa is a daily-driver developer utility. It does grep / `rg`-style
searches with a polished Qt 6 / Kirigami interface, atomic-rename
safe replace, OOXML / ODF / PDF document extraction, Docker + Podman
container search, optional AI assistance, and a fully scriptable CLI.

## Status

**v0.3.0** — Polish + responsiveness pass on top of the v0.2 GUI
parity release. Per-tab result-row isolation now preserves the
full row buffer across tab switches; the action toolbar wraps to
additional rows on narrow windows instead of clipping; the pink
gecko renders correctly in the Wayland taskbar via Qt's
`setDesktopFileName` + auto-installed hicolor theme; Settings
auto-save on change (no more Apply button); Filters/Esc/Export-menu
toggles behave as users expect; re-clicking the active tab or
sidebar nav item is now a no-op instead of a hidden side effect.

v0.2 added the original GUI parity surface: folder picker,
filter drawer, target selector for Docker/Podman, replace flow with
residual-journal recovery, sortable result columns,
search-within-results, in-session tabs, Profiles page, History page,
export to CSV/JSON/Markdown, result row context menu, single-instance
lockfile, KNotification, keyboard shortcuts, Editor/Replace/
Accessibility/Privacy settings.

The Rust core, CLI, container adapter, AI client, document extraction,
encoding detection, settings, history, profiles, context preview,
sorting, gitignore parity, and Fluent localization (en / de / ja) all
ship working. The Qt 6 / Kirigami GUI binary boots via
[cxx-qt 0.8](https://github.com/KDAB/cxx-qt).

Release notes: [docs/release-notes-0.3.0.md](docs/release-notes-0.3.0.md).

## Quick start

### Requirements

- Linux (Wayland or X11; KDE Plasma 6 recommended)
- Rust **1.95+** (stable)
- Qt 6.6+ + Kirigami 6 (for the GUI; not required for the CLI)
- Optional: `pdftotext` (Poppler) for PDF search;
  `docker` or `podman` for container search; KWallet or
  GNOME Keyring for AI API-key storage.

### CLI

```bash
# install from source
cargo install --path crates/grexa-cli

# basic search
grexa-cli ~/code TODO

# regex with case sensitivity
grexa-cli ~/code 'fn\s+\w+_test' --regex --case-sensitive

# inside a Podman container
grexa-cli /etc TODO --container web --runtime podman

# JSON output for piping
grexa-cli ~/code TODO --format json | jq '.[] | .full_path'

# shell completions
grexa-cli completions bash > ~/.local/share/bash-completion/completions/grexa-cli
```

### GUI

```bash
cargo build --release -p grexa
target/release/grexa
```

The GUI is a Rust + Qt 6 / Kirigami binary built with
[cxx-qt 0.8](https://github.com/KDAB/cxx-qt) — pure Cargo, no CMake.
QML files under `apps/grexa-gui/qml/` are bundled into the binary at
build time via Qt's resource system and registered under the
`com.visorcraft.Grexa 1.0` QML module. The contract-shaped page
placeholders ship today; live wiring against the controllers
continues in Phases 4–18 of [PLAN.md](PLAN.md).

## Architecture

Grexa is a Cargo workspace:

| Crate              | Responsibility |
| ------------------ | -------------- |
| `grexa-core`       | Search, replace, encoding, gitignore, glob filters, context preview, sorting, settings, history, profiles, document extraction, desktop integration helpers. |
| `grexa-containers` | Docker + Podman detection, container listing, direct `exec` grep, archive mirror fallback. |
| `grexa-ai`         | OpenAI-compatible HTTP client, model discovery, secret-service-backed API key storage. |
| `grexa-cli`        | Headless `grexa-cli` binary with all search/replace/container flags + shell completions + man page generator. |
| `grexa-i18n`       | Fluent-backed localization (en / de / ja today; plural-aware). |
| `grexa` (apps/)    | Qt 6 / Kirigami GUI shell. |

See [docs/architecture.md](docs/architecture.md) for the full breakdown.

## Documentation

- [docs/features.md](docs/features.md) — what Grexa does, end to end
- [docs/usage.md](docs/usage.md) — workflows for KDE, Docker, Podman,
  AI, replace, CLI
- [docs/architecture.md](docs/architecture.md) — module map + data
  paths
- [docs/build-and-test.md](docs/build-and-test.md) — distro
  prerequisites and dev workflow
- [docs/reference.md](docs/reference.md) — settings schema, CLI
  reference, paths, keyboard shortcuts, encoding support
- [docs/translations.md](docs/translations.md) — localization pipeline
  for translators
- [docs/security.md](docs/security.md) — threat model, telemetry
  policy, secret storage
- [docs/linux-decisions.md](docs/linux-decisions.md) — what was
  intentionally removed or replaced from Grex
- [docs/migration-from-grex.md](docs/migration-from-grex.md) —
  bringing Grex settings / history / profiles into Grexa
- [docs/gui-design.md](docs/gui-design.md) — cxx-qt bridge + QML
  module map
- [docs/release-notes-0.3.0.md](docs/release-notes-0.3.0.md) —
  v0.3.0 changes (polish + responsiveness)
- [docs/release-notes-0.2.0.md](docs/release-notes-0.2.0.md) —
  v0.2.0 changes (Phase 20 GUI parity)
- [docs/release-notes-0.1.0.md](docs/release-notes-0.1.0.md) —
  v0.1.0 changes
- [PLAN.md](PLAN.md) — phase-by-phase implementation map (Phase 20
  closed; v0.3 release-ready)
- [AGENTS.md](AGENTS.md) — guidelines for AI assistants working on
  this repo. AI tooling (Claude Code, Cursor, etc.) reads this first.
- [CREDITS.md](CREDITS.md) — third-party attribution

## Licensing

Grexa is licensed under GPL-3.0-only, matching the upstream Grex
project. See [LICENSE](LICENSE) for the full text.

Third-party Rust crates and runtime components are credited in
[CREDITS.md](CREDITS.md). Every dependency must use a
GPL-3.0-compatible license; the allowlist lives in
[`deny.toml`](deny.toml). Run `just deny` to enforce the policy.

## Contributing

- `just ci` runs format, clippy, and tests. Pre-PR sanity check.
- `just manpage` + `just completions` regenerate the CLI artifacts.
- `python3 scripts/check_locale_sync.py` enforces the Fluent
  translation key parity across locales.
- New strings must land in `crates/grexa-i18n/locales/en/grexa.ftl`
  before any caller can reference them.
- `docs/grex-*-audit.md` pins upstream behavior; if you change
  something an audit doc describes, update the audit and the code
  in the same change. Intentional divergences belong in
  `docs/linux-decisions.md`.
- Every new source file gets a two-line SPDX REUSE header
  (`SPDX-FileCopyrightText: 2026 VisorCraft LLC` +
  `SPDX-License-Identifier: GPL-3.0-only`). See
  [AGENTS.md](AGENTS.md) for the conventions in full.

## Reporting issues

Use the GitHub issue tracker. For security concerns, see
[docs/security.md](docs/security.md).
