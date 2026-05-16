# Grexa

> Fast, precise file content search for Linux. Inspired by [Grex](https://github.com/visorcraft/Grex), built ground-up for KDE Plasma.

Grexa is a daily-driver developer utility. It does grep / `rg`-style
searches with a polished Qt 6 / Kirigami interface, atomic-rename
safe replace, OOXML / ODF / PDF document extraction, Docker + Podman
container search, optional AI assistance, and a fully scriptable CLI.

## Status

Alpha — the Rust core (search, replace, container adapter, AI HTTP
client, document extraction, encoding detection, settings, history,
profiles, context preview, sorting, gitignore parity) and CLI ship
working today. The Qt 6 / Kirigami GUI is in active development; see
[PLAN.md](PLAN.md) for the full phase map.

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

The GUI is built with `cargo build --release -p grexa` (placeholder
binary today; the full Kirigami shell lands in Phase 4 of
[PLAN.md](PLAN.md)).

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
- [PLAN.md](PLAN.md) — phase-by-phase implementation map

## Licensing

Grexa is licensed under GPL-3.0-only, matching the upstream Grex
project. See [LICENSE](LICENSE).

Third-party dependencies are required to use a compatible permissive
or copyleft license; the allowlist lives in
[`deny.toml`](deny.toml). Run `just deny` to enforce the policy.

## Contributing

- `just ci` runs format, clippy, and tests. Pre-PR sanity check.
- `just manpage` + `just completions` regenerate the CLI artifacts.
- `python3 scripts/check_locale_sync.py` enforces the Fluent
  translation key parity across locales.
- New strings must land in `crates/grexa-i18n/locales/en/grexa.ftl`
  before any caller can reference them.
- `docs/*.md` is the source of truth for behavior contracts; if you
  change something the audit doc describes, update both the docs and
  the code in the same change.

## Reporting issues

Use the GitHub issue tracker. For security concerns, see
[docs/security.md](docs/security.md).
