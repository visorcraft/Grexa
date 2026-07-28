<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Linux Decisions

Grexa is a Linux/Qt port of the Windows/WinUI application
[Grex](https://github.com/visorcraft/grex). This document records deliberate
platform changes. Search semantics stay aligned unless a decision below says
otherwise.

The `grex-*-audit.md` files preserve upstream evidence. This page describes the
current Linux product.

## Platform and build

| Grex assumption | Grexa decision |
| --------------- | -------------- |
| Windows 11, WinUI 3, and Windows App SDK | Qt 6, Qt Quick Controls, and Kirigami on Linux. |
| .NET runtime | Stable Rust 2024 edition. |
| C# to XAML binding | cxx-qt QObjects and a compiled QML module. |
| MSIX | GitHub artifacts for AppImage, Flatpak, Arch, Debian, Fedora, and tar; an openSUSE source recipe is also maintained. |
| Visual Studio/MSBuild | Cargo through `just`; no project CMake build. |
| Windows-only APIs | freedesktop, Qt, or explicit command-line helpers. |

Qt is needed only for the GUI crate. `grexa-core`, `grexa-cli`,
`grexa-containers`, `grexa-ai`, and `grexa-i18n` remain Qt-free.

## Paths and filesystems

| Grex behavior | Grexa decision |
| ------------- | -------------- |
| `%LocalAppData%\Grex` | XDG configuration, data, cache, state, and runtime locations. |
| Drive-letter paths | No automatic mapping. Select the path as mounted on Linux. |
| UNC and WSL paths | No string translation. Use a native Linux or mounted local path. |
| `smb://`, `sftp://`, and other abstract URLs | Not search roots. Browse or mount them through the desktop, then select the local GVFS/KIO-FUSE/mount path. |
| NTFS-style case-insensitive path identity | Exact, case-sensitive recent-path dedupe. Profile names retain their documented name matching behavior. |
| Aggressive path canonicalization | Avoided where it would erase meaningful symlink, bind-mount, GVFS, or KIO-FUSE behavior. |
| Windows system-directory exclusions | Linux exclusions add `/proc`, `/sys`, `/dev`, `/run`, dependency/build directories, and other unsafe/noisy defaults. `--include-system` explicitly overrides them. |
| Windows Recycle Bin | `gio trash` when available. |

Grexa does not mock filesystem semantics. Tests use temporary real directories,
permissions, symlinks, and gitignore files.

## Configuration and persisted data

Settings are atomic JSON at:

```text
$XDG_CONFIG_HOME/grexa/settings.json
```

Recent paths, search history, and search profiles use the Apache-2.0
`grexa-db` engine:

```text
$XDG_DATA_HOME/grexa/db/recent_paths/
$XDG_DATA_HOME/grexa/db/search_history/
$XDG_DATA_HOME/grexa/db/search_profiles/
```

Records are Markdown with YAML frontmatter. Older Grexa JSON list stores are
imported only when the destination collection is empty, then renamed to
`.bak`. This is a Grexa storage migration, not a Windows Grex backup importer.

The current release has no public Grex backup converter. PascalCase Grex
settings, drive letters, UNC paths, and WSL paths must be translated manually.
See [Migration from Grex](migration-from-grex.md).

AI keys never enter JSON or grexa-db. They live in the Linux Secret Service
under service `com.visorcraft.Grexa.ai`.

## Search index

Grex can use Windows Search to seed candidate files. Linux has no equivalent
enabled in Grexa.

The code retains `use_file_index`, `--use-index`, `--no-index`, and a Baloo
adapter contract for compatibility and evaluation. Production search does not
call Baloo. All searches use Grexa's own walker or the container paths
described below.

Any future candidate provider must be optional, must recheck every candidate,
and must fall back to the normal walker for stale indexes, errors, regex,
unsupported comparisons, and non-indexed roots. See
[Baloo candidate-seeding decision](baloo-spike.md).

## Containers

| Grex behavior | Grexa decision |
| ------------- | -------------- |
| Docker Desktop and Windows named pipe | Linux Docker CLI and configured daemon/socket. |
| Docker-only target | Docker plus rootless and rootful Podman. |
| One search path | Direct in-container grep when capable, otherwise a bounded local archive mirror searched by `grexa-core`. |
| Container writeback | Not exposed. Public container workflows are search-only. |

Runtime subprocesses use argv arrays, a 30-second timeout, bounded output, and
live-container ID resolution. Grexa never constructs a shell command from the
term, path, or container name.

## Desktop services

| Windows integration | Linux replacement |
| ------------------- | ----------------- |
| Explorer reveal | `org.freedesktop.FileManager1.ShowItems` through `gdbus`, then `xdg-open`. |
| Registry editor discovery | Explicit editor presets or a custom shell-free argv template. |
| Toast notification | `notify-send` and the freedesktop notification service. |
| Clipboard APIs | Qt clipboard with `wl-copy` or `xclip` helper paths. |
| Single-instance activation | `flock` in the XDG runtime/cache directory, D-Bus activation through `gdbus`, then `wmctrl` fallback. |
| Windows shell launch | Direct process argv, never a shell. |

External helpers are optional. Their absence disables the related integration
or selects a documented fallback; it does not change core local search.

## Window and appearance behavior

- The window manager owns window placement.
- Grexa persists width and height, not X/Y coordinates.
- Kirigami and Qt provide Linux desktop integration.
- Grexa includes system, light, dark, OLED black, and compatible named palette
  choices.
- Mica, acrylic, WinUI resource dictionaries, and custom Windows chrome are
  not ported.
- Breeze icons and Qt SVG plugins are bundled by the AppImage path because
  Kirigami symbolic icons need them at runtime.

## Localization

Grex resource catalogs become embedded Fluent catalogs. English is the source
of truth and German/Japanese ship with exact key parity. QML requests strings
through controller `i18n(...)` and `i18nPlural(...)` invokables rather than
`qsTr()`.

Culture-aware searching is separate from UI translation. ICU4X implements
casing and locale handling; Unicode normalization preserves source offsets
through a grapheme mapping.

## AI and networking

Grexa has no telemetry, update check, or crash uploader.

The only direct HTTP feature is user-enabled OpenAI-compatible AI support.
Credentials are sent only to HTTPS endpoints or loopback HTTP, redirects are
disabled, and each request is bounded. Search evidence leaves the machine only
after explicit AI enablement and a user action.

Container runtimes and desktop mounts may perform their own network activity
outside Grexa's HTTP client.

## Replace safety

Grexa keeps local replacement synchronous, encoding-aware, permission-aware,
and atomic. It adds explicit resource caps and a state journal for interrupted
operations. It does not create backup copies and has no undo stack.

Searchable archives, Office/OpenDocument files, PDFs, and containers are not
rewritten by the public GUI or CLI.

## Licensing boundary

The Grexa workspace is GPL-3.0-only. The separately maintained `grexa-db`
engine is Apache-2.0 so it can be embedded independently. Dependency direction
is one way: GPL Grexa crates may use `grexa-db`; `grexa-db` must not depend on
Grexa GPL crates.

## Out of scope

- Windows, macOS, mobile, and web targets
- WSL/UNC path translation
- Windows Search and Windows shell integrations
- custom window decoration
- telemetry
- a public Grex backup importer
- public container or archive replacement

Update this document whenever a platform divergence changes.
