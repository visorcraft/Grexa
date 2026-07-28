<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

<p align="center">
  <img src="packaging/icons/512x512/apps/com.visorcraft.Grexa.png" alt="Grexa logo" width="220">
</p>

<h1 align="center">Grexa</h1>

<p align="center">
  <strong>Fast, precise file-content search for Linux.</strong>
</p>

<p align="center">
  A Qt 6 / Kirigami desktop workbench and scriptable Rust CLI for searching,
  filtering, previewing, and safely replacing text in local files, documents,
  and containers.
</p>

Grexa is the Linux-native port of
[Grex](https://github.com/visorcraft/grex). It combines a polished desktop
workflow with a headless CLI, both backed by the same synchronous Rust search
engine.

## Why Grexa?

- **Precise search**: literal or regex matching, whole-word mode,
  case/diacritic controls, Unicode normalization, culture-aware comparison,
  file globs, directory exclusions, size filters, `.gitignore`, hidden files,
  and symlink controls.
- **Useful results**: content and file views, line and column numbers,
  match highlighting, context preview, sorting, result-within-result filtering,
  tabs, export, editor launch, and file-manager reveal.
- **Safe replace**: preview and confirmation in the GUI, `--dry-run` in the
  CLI, encoding-aware atomic writes, permission preservation, regex capture
  expansion, and an interruption journal.
- **Document search**: DOCX, XLSX, PPTX, ODT, ODS, ODP, ZIP, RTF, and PDF
  text extraction.
- **Container search**: Docker and Podman support with in-container `grep` and
  a local mirror fallback for minimal images.
- **Optional AI help**: an explicitly enabled OpenAI-compatible chat panel,
  including bounded summaries of visible matches. Credentials stay in the
  Linux Secret Service.
- **Plain-file app data**: recent paths, history, and profiles are Markdown
  records managed by [grexa-db](https://github.com/visorcraft/grexa-db), not an
  opaque application database.
- **Linux-native integration**: Qt 6, Kirigami, Wayland/X11, XDG paths,
  desktop notifications, `org.freedesktop.FileManager1`, and common Linux
  editors.

## Install

[GitHub Releases](https://github.com/visorcraft/grexa/releases) publish Linux
x86_64 artifacts. Choose the native package for your distribution when
available:

| Artifact | Best for | Install or run |
| -------- | -------- | -------------- |
| AppImage | Portable GUI | `chmod +x Grexa-*.AppImage && ./Grexa-*.AppImage` |
| `.pkg.tar.zst` | Arch / CachyOS | `sudo pacman -U grexa-*.pkg.tar.zst` |
| `.deb` | Debian-family systems with Kirigami 6 | `sudo apt install ./grexa_*.deb ./grexa-cli_*.deb` |
| `.rpm` | The Fedora release matching the artifact's Qt ABI | `sudo dnf install ./grexa-*.rpm` |
| `.flatpak` | Sandboxed desktop install | `flatpak install --user ./grexa.flatpak` |
| `.tar.gz` | Unpackaged tree on a compatible Qt/Kirigami system | unpack and run `bin/grexa` or `bin/grexa-cli` |

The AppImage and tarball are not package-manager installations. For the
tarball, the host must provide compatible Qt/Kirigami libraries. Release
tarballs are built on Arch and are not a universal Linux binary:

```bash
tar -xzf grexa-<version>-linux-x86_64.tar.gz
cd grexa-<version>-linux-x86_64
./bin/grexa
./bin/grexa-cli --help
```

To install that tarball under `/usr/local`:

```bash
sudo install -Dm755 bin/grexa /usr/local/bin/grexa
sudo install -Dm755 bin/grexa-cli /usr/local/bin/grexa-cli
sudo cp -a share/. /usr/local/share/
```

See [Building and testing](docs/build-and-test.md) for distro prerequisites,
source builds, Flatpak/AppImage/package commands, and GUI troubleshooting.

## Build from source

The repository pins Rust 1.96. The GUI also needs Qt 6, Kirigami 6, and a C++
toolchain. The CLI does not need Qt.

```bash
git clone https://github.com/visorcraft/grexa.git
cd grexa

just ci
just build-release

target/release/grexa
target/release/grexa-cli --help
```

Without [`just`](https://just.systems/), use the equivalent Cargo commands:

```bash
cargo test --workspace
cargo build --workspace --release
```

CLI-only:

```bash
cargo build -p grexa-cli --release
target/release/grexa-cli ~/code TODO
```

## Desktop quick start

1. Launch `grexa`.
2. Enter a directory and search term. Toggle Regex or Case Sensitive if needed.
3. Open the filter drawer for file globs, excluded directories,
   `.gitignore`, hidden files, documents, system paths, recursion, and
   symlinks. The CLI exposes the additional size and Unicode controls.
4. Select **Content** for one row per matching line or **Files** for one row per
   file.
5. Press **Search**. Use the result filter to narrow the already loaded rows.
6. Select a result and press Space for context or Enter to open it in the
   configured editor.

Useful desktop surfaces:

- Search tabs retain independent forms and result snapshots during the session.
- History records completed searches. Opening an entry restores the form but
  does not run it automatically.
- Profiles save named search configurations.
- Regex Builder tests patterns against sample text before using them.
- Tools → Database browses any grexa-db directory, including Grexa's own data
  at `$XDG_DATA_HOME/grexa/db`.
- Settings auto-save appearance, search defaults, integrations, editor,
  replace, accessibility, privacy, and diagnostics options.

Full walkthrough: [Using Grexa](docs/usage.md).

## CLI quick start

Search syntax:

```text
grexa-cli [OPTIONS] <PATH> <TERM>
grexa-cli replace [OPTIONS] <PATH> <TERM> <REPLACEMENT>
```

Examples:

```bash
# Literal search. Default output: path:line:column:content
grexa-cli ~/code TODO

# Regex, whole words, Rust and Markdown only
grexa-cli ~/code 'TODO|FIXME' --regex --whole-word \
  --match-files '*.rs|*.md'

# Respect ignore files and include hidden paths
grexa-cli ~/code secret --gitignore --include-hidden

# Script with JSON
grexa-cli ~/code TODO --format json | jq -r '.[].full_path'

# Preview a replacement, then apply it
grexa-cli replace ~/code 'old_(\w+)' 'new_$1' --regex --dry-run
grexa-cli replace ~/code 'old_(\w+)' 'new_$1' --regex

# Search a running Podman container
grexa-cli /etc/nginx TODO --container web --runtime podman
```

Exit codes are grep-like: `0` means matches or modifications, `1` means none,
and `2` means an error. Run `grexa-cli --help` or read the
[complete reference](docs/reference.md).

## Defaults and important limits

- Searches recurse by default.
- Matching is case-insensitive, ordinal, and diacritic-sensitive by default.
- Hidden paths, system/dependency directories, binary/documents, symlink
  traversal, and `.gitignore` handling are off until enabled.
- Without `--max-results`, one search returns at most 1,000,000 matching rows.
- A single file larger than 512 MiB is not read into memory.
- Replace is for local text files. Container and archive/document replacement
  are not exposed in the GUI or CLI.
- The `--use-index` and `--no-index` compatibility flags are accepted, but
  Baloo candidate seeding is not wired into the search path yet.
- AI is off by default. Grexa makes no AI request until the user enables it and
  initiates a chat or endpoint test.

See [Reference: resource bounds](docs/reference.md#resource-bounds) for every
enforced cap and timeout.

## Data, privacy, and portability

Grexa follows the XDG Base Directory specification:

| Data | Default path |
| ---- | ------------ |
| Settings | `~/.config/grexa/settings.json` |
| Recent paths | `~/.local/share/grexa/db/recent_paths/` |
| Search history | `~/.local/share/grexa/db/search_history/` |
| Search profiles | `~/.local/share/grexa/db/search_profiles/` |
| CLI log | `~/.local/state/grexa/grexa.log` |
| GUI logs | `~/.local/state/grexa/grexa-gui.*.log` |
| Replace journal | `~/.local/state/grexa/replace-journal.json` |
| Container mirrors | `~/.cache/grexa/container-mirrors/` |

Recent paths, history, and profiles are plain Markdown records with YAML
frontmatter. You can inspect, back up, diff, or version them with ordinary
filesystem tools. Legacy Grexa JSON stores are migrated to these collections
on first use and renamed with a `.bak` suffix.

AI API keys are not stored in these files. They live in the system Secret
Service under `com.visorcraft.Grexa.ai`, keyed by endpoint. Grexa ships no
telemetry. Read [Security and privacy](docs/SECURITY.md) for the exact outbound
traffic, subprocess, logging, and replacement threat model.

The Flatpak can access the home directory, `/run/media`, and paths granted
through the desktop portal. It intentionally has no Docker or Podman socket
access; use a native package, AppImage, or compatible tar build for container
search.

## Documentation

Start at the [documentation index](docs/README.md).

| Need | Document |
| ---- | -------- |
| Learn the desktop and CLI workflows | [Using Grexa](docs/usage.md) |
| Find flags, settings, paths, formats, shortcuts, and limits | [Reference](docs/reference.md) |
| See everything Grexa supports | [Features](docs/features.md) |
| Install, build, test, package, or troubleshoot | [Building and testing](docs/build-and-test.md) |
| Understand crate boundaries and runtime flows | [Architecture](docs/architecture.md) |
| Work on Qt/QML and cxx-qt | [GUI design](docs/gui-design.md) |
| Translate the app | [Translations](docs/translations.md) |
| Migrate concepts and data from Grex | [Migration from Grex](docs/migration-from-grex.md) |
| Review privacy or report a vulnerability | [Security and privacy](docs/SECURITY.md) |
| Contribute | [CONTRIBUTING.md](CONTRIBUTING.md) |

## Contributing and support

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), keep changes
focused, update tests and docs together, and run:

```bash
just ci
```

- Bugs and feature requests:
  [GitHub Issues](https://github.com/visorcraft/grexa/issues)
- Security reports: use
  [GitHub private vulnerability reporting](docs/SECURITY.md#reporting-a-vulnerability)
- Third-party attribution: [CREDITS.md](CREDITS.md) and the generated
  [third-party license supplement](docs/credits-third-party.md)

## License

Grexa is licensed under
[GPL-3.0-only](https://spdx.org/licenses/GPL-3.0-only.html). See
[LICENSE](LICENSE).

The separately maintained grexa-db engine is Apache-2.0 and is consumed as a
pinned dependency. Its permissive license is intentional; Grexa's application
and other workspace crates remain GPL-3.0-only.
