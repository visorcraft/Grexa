<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

<p align="center">
  <img src="packaging/icons/512x512/apps/io.visorcraft.Grexa.png" alt="Grexa logo" width="250">
</p>

<h1 align="center">Grexa</h1>

<p align="center">
  <strong>Fast, precise file-content search for Linux.</strong>
</p>

<p align="center">
  A Qt 6 / Kirigami desktop app and scriptable Rust CLI for searching,
  filtering, previewing, and safely replacing text across local files,
  documents, and containers.
</p>

## What is Grexa?

Grexa is a Linux-native port of
[Grex](https://github.com/visorcraft/grex), rebuilt as a Rust workspace
with a Qt 6 / Kirigami interface. It is designed for developers and
power users who need fast local search with predictable filters,
grep-style automation, and a polished desktop workflow.

Grexa can:

- Search by literal text or regex, including advanced regex features
  through a fast `regex` / `fancy-regex` cascade.
- Respect `.gitignore`, hidden-file settings, glob filters, size
  filters, binary-file rules, symlinks, and recursive directory
  options.
- Preview matches with file path, line, column, encoding, modified
  time, and sorted result views.
- Replace text safely with atomic file writes and a replace journal.
- Search extracted text from OOXML, ODF, and PDF documents.
- Search inside Docker or Podman containers.
- Run as either the `grexa` desktop app or the `grexa-cli` command.
- Store API keys in the Linux Secret Service when optional AI features
  are configured.

## Setup

### Requirements

- Linux on Wayland or X11. KDE Plasma 6 is the primary desktop target.
- Qt 6.6+ and Kirigami 6 for the GUI.
- Rust 1.95+ only when building from source.
- Optional: `pdftotext` from Poppler for PDF search.
- Optional: Docker or Podman for container search.
- Optional: KWallet or GNOME Keyring for AI-provider keys.

### Install development packages

Use your distro's package manager before building from source. The
development packages also satisfy the GUI runtime requirements on most
systems.

| Distro | Command |
| ------ | ------- |
| Debian / Ubuntu | `sudo apt install rustc cargo qt6-base-dev qt6-declarative-dev qt6-tools-dev qml6-module-org-kde-kirigami clang poppler-utils` |
| Fedora | `sudo dnf install rust cargo qt6-qtbase-devel qt6-qtdeclarative-devel kf6-kirigami-devel clang poppler-utils` |
| Arch / Manjaro | `sudo pacman -S rust qt6-base qt6-declarative kirigami clang poppler` |
| openSUSE | `sudo zypper install rust cargo qt6-base-devel qt6-declarative-devel kirigami6-devel clang poppler-tools` |

The repository uses [`just`](https://just.systems/) for common tasks.
If it is not installed, the equivalent `cargo` commands still work.

```bash
cargo install just
```

## Install

### From a GitHub Release

Download the latest `grexa-<version>-linux-x86_64.tar.gz` from the
repository's GitHub Releases page, then unpack it:

```bash
tar -xzf grexa-<version>-linux-x86_64.tar.gz
cd grexa-<version>-linux-x86_64

./bin/grexa
./bin/grexa-cli --help
```

To install the archive into `/usr/local`:

```bash
sudo install -Dm755 bin/grexa /usr/local/bin/grexa
sudo install -Dm755 bin/grexa-cli /usr/local/bin/grexa-cli
sudo cp -a share/. /usr/local/share/
```

### From source

```bash
git clone https://github.com/visorcraft/grexa.git
cd grexa

just ci
just build-release

target/release/grexa
target/release/grexa-cli --help
```

CLI-only builds do not need Qt:

```bash
cargo build -p grexa-cli --release
target/release/grexa-cli ~/code TODO
```

### Packaging

Packaging recipes live under [`packaging/`](packaging/), including
Flatpak, AppImage, Debian, Fedora, openSUSE, and Arch/CachyOS
metadata. See [docs/build-and-test.md](docs/build-and-test.md) for
packaging commands and release automation details.

## Tweak Grexa

### Common CLI workflows

```bash
# Basic content search
grexa-cli ~/code TODO

# Regex search
grexa-cli ~/code 'fn\s+\w+_test' --regex --case-sensitive

# JSON output for scripts
grexa-cli ~/code TODO --format json | jq '.[] | .full_path'

# Search inside a Podman container
grexa-cli /etc TODO --container web --runtime podman

# Generate shell completions
grexa-cli completions bash > ~/.local/share/bash-completion/completions/grexa-cli
```

### Desktop settings

The GUI settings page auto-saves changes. Grexa stores local app data
under standard XDG locations:

| Data | Default path |
| ---- | ------------ |
| Settings | `~/.config/grexa/settings.json` |
| Recent paths, history, profiles | `~/.local/share/grexa/` |
| Logs and replace journal | `~/.local/state/grexa/` |

Set `GREXA_LOG` to tune logging:

```bash
GREXA_LOG=debug grexa
```

### Optional integrations

- PDF search uses `pdftotext` when available.
- Container search uses Docker or Podman from `PATH`.
- AI-provider keys are stored in the system Secret Service, not in QML
  or plain-text config files.
- Localization currently ships English, German, and Japanese catalogs.

Full usage details are in [docs/usage.md](docs/usage.md). CLI flags,
settings, paths, and keyboard shortcuts are in
[docs/reference.md](docs/reference.md).

## Contribute

Contributions are welcome through the standard fork-and-pull-request
workflow. Start with [CONTRIBUTING.md](CONTRIBUTING.md), which covers
local setup, coding standards, tests, documentation expectations,
localization rules, dependency policy, and pull request requirements.

The short version:

```bash
git clone https://github.com/<you>/grexa.git
cd grexa
git checkout -b fix-or-feature-name

just ci
```

Before opening a pull request, include focused tests for behavior
changes, update relevant docs, and make sure `just ci` passes.

## Documentation

- [docs/features.md](docs/features.md) — feature inventory
- [docs/usage.md](docs/usage.md) — user workflows
- [docs/reference.md](docs/reference.md) — settings and CLI reference
- [docs/build-and-test.md](docs/build-and-test.md) — build, test, and packaging guide
- [docs/architecture.md](docs/architecture.md) — workspace architecture
- [docs/gui-design.md](docs/gui-design.md) — Qt / cxx-qt bridge design
- [docs/translations.md](docs/translations.md) — localization workflow
- [docs/security.md](docs/security.md) — threat model and disclosure policy
- [docs/feature-parity.md](docs/feature-parity.md) — Grex / Grexa parity matrix

## License

Grexa is licensed under GPL-3.0-only, matching the upstream Grex
project. See [LICENSE](LICENSE) for the full text and
[CREDITS.md](CREDITS.md) for third-party attribution.
