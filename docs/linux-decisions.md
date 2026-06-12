# Linux Decisions

Grexa is a Linux-only application. This document records the Windows-specific
behaviors and abstractions Grex carries that are deliberately removed,
replaced, or marked non-applicable in Grexa.

It is a synthesis of the per-file audits in `docs/grex-*-audit.md`. Each row
points to the audit that originally captured the underlying behavior.

## Platform Scope

| Grex assumption                                          | Grexa decision |
| -------------------------------------------------------- | -------------- |
| Windows 11 / Windows App SDK / WinUI 3 host              | Removed. Grexa is Qt 6 + QML + Kirigami on Linux. |
| `Environment.SpecialFolder.LocalApplicationData`         | Replaced. Configuration uses `$XDG_CONFIG_HOME/grexa`, user data uses `$XDG_DATA_HOME/grexa`, caches use `$XDG_CACHE_HOME/grexa`, logs use `$XDG_STATE_HOME/grexa`. (`grex-storage-services-audit.md`) |
| `Package.appxmanifest` / MSIX packaging                  | Removed. Grexa ships as Flatpak (primary), AppImage (secondary), and per-distro packages. |
| `.NET 9` / `System.Text.Json` runtime                    | Removed. Rust 2024 edition with `serde`/`serde_json` and `cxx-qt`. |
| Multiple cultures via `CultureInfo.CurrentCulture.Name`  | Kept conceptually. Replaced with ICU strategy decided in Phase 2 (line 200). Default culture seeds from the system locale at first run. |

## File Systems And Paths

| Grex behavior                                                  | Grexa decision |
| -------------------------------------------------------------- | -------------- |
| Windows drive letters (`C:\...`)                                | Not applicable. Imported entries that begin with a drive letter are translated to `$HOME` (when the user accepts a one-time mapping) or kept as opaque strings with a "not available on Linux" marker. |
| UNC paths (`\\server\share\path`)                               | Not applicable. Grexa searches paths after they are mounted by GIO/GVFS, KIO FUSE, `cifs-utils`, or `autofs`. Abstract `smb://` / `fish://` / `mtp://` URLs are detected and the user is told to mount or browse the real path. (`grex-storage-services-audit.md`) |
| WSL paths (`\\wsl$\<distro>\...`)                               | Removed. Grexa never accesses a WSL filesystem from Linux. Imported WSL paths are marked unavailable. |
| Long-path / namespaced paths (`\\?\C:\...`)                     | Not applicable. |
| Case-insensitive filesystem semantics (NTFS default)            | Linux defaults to case-sensitive. Grexa uses case-sensitive equality for recent-path dedupe; profile name comparison is ASCII case-insensitive to match Grex `OrdinalIgnoreCase`. (`grex-storage-services-audit.md`) |
| System path auto-exclusions (`.git`, `vendor`, `node_modules`, `storage/framework`, `bin`, `obj`, `sys`, `proc`, `dev`) | Kept, with Linux pseudo-filesystem guards added for root searches (`/proc`, `/sys`, `/dev`, `/run`). (`grex-search-service-audit.md`) |
| Path canonicalization                                           | Avoided where it would break KIO-FUSE, GVFS, bind mounts, or symlinks the user expects to follow. |

## Windows Search Index

Grex `Services/WindowsSearchIntegration.cs` integrates with the Windows Search
index for accelerated candidate seeding. (`grex-windows-search-integration-audit.md`)

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| `Microsoft.Search` OLE DB provider                              | Removed. |
| `UseWindowsSearchIndex` setting                                 | Renamed to `use_file_index`. On Linux this becomes optional Baloo candidate seeding. Imports translate the old field name. (`grex-storage-services-audit.md`) |
| Always re-verify candidates with Grex's own engine              | Kept. Baloo is a candidate source, never the source of truth. The custom walker re-validates every candidate. (Phase 13 spike.) |
| Regex searches disabled for index seeding                       | Kept. Baloo cannot prefilter for regex; Grexa always falls back to the walker. |

## Containers

Grex `Services/DockerSearchService.cs` integrates with Docker Desktop's Windows
named pipes. (`grex-docker-search-service-audit.md`)

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| Docker Desktop named pipe (`\\.\pipe\docker_engine`)            | Removed. Linux uses `/var/run/docker.sock` and `$DOCKER_HOST`. |
| Docker-only target                                              | Replaced. Grexa supports Docker, rootless Podman, and rootful Podman. Runtime is selected automatically or via `--runtime`. |
| `EnableDockerSearch` setting                                    | Renamed to `enable_container_search`. Import translates the old field name. (`grex-storage-services-audit.md`) |
| `DockerSearchEnabledChanged` static event                       | Removed. The settings service is value-in/value-out; the GUI controller diffs and emits its own change notification. |

## Notifications, Toasts, And Activation

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| Windows toast notifications via `Microsoft.Toolkit.Uwp.Notifications` | Removed. Grexa uses KNotifications on KDE, with a freedesktop notification fallback on other desktops. |
| Toast diagnostic panel in Settings                              | Replaced. A notification diagnostic surface is added only if Linux notification failures need user-facing diagnostics. |
| Single-instance activation via Windows App SDK                  | Replaced. Optional DBus single-instance activation, only if it fits the rest of the UX. |
| Foreground-window restore on toast click                        | Not implemented for 1.0. |

## Editor And File Manager Integration

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| Windows shell verbs (`shell:Properties`, `explorer.exe /select,`) | Replaced. Grexa uses `org.freedesktop.FileManager1.ShowItems` with `xdg-open` as a fallback. |
| Editor templates inferred from Windows registry                 | Replaced. Grexa ships explicit presets for Kate/KWrite, VS Code/VSCodium, JetBrains IDEs, Sublime Text, GNOME Text Editor, Neovim terminal wrapper, and the `xdg-open` default. |
| `Path.GetFullPath`-style normalization for editor launch        | Replaced with a Linux-aware launcher that does not aggressively canonicalize KIO-FUSE / GVFS paths. |

## Themes, Theming, And Visual

Grex `ThemePreference` is an integer enum with the values 0..11. The Linux
counterpart is documented in `grex-storage-services-audit.md` and
`grex-settings-view-audit.md`.

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| Custom WinUI ResourceDictionary themes                          | Removed. Grexa uses QQC2 Desktop Style with an optional Grexa visual theme layer. |
| `ThemePreference.System`                                        | Mapped to "Follow KDE color scheme". |
| `ThemePreference.Light` / `.Dark`                               | Mapped to Grexa Light / Grexa Dark. |
| High-contrast / accent themes (`GentleGecko` … `Vibes`)         | Theme identifiers preserved on disk for round-trip; user-visible labels can be rebranded per Linux design guidance. |
| Grexa-only OLED Black theme                                      | Added as `ThemePreference.OledBlack = 12`; uses true black app surfaces while preserving Grex-compatible `0..11` values. |
| Fluent acrylic / Mica backdrop effects                          | Removed. |
| Per-window position persistence                                 | Removed from the schema. Linux leaves placement to the window manager. Imports drop `WindowX`/`WindowY` and accept width/height only when ≥ 400px. |

## Settings, History, Profiles, Recent Paths

See `grex-storage-services-audit.md` for the full mapping. Highlights:

- Settings JSON moves from `%LocalAppData%\Grex\settings.json` to
  `$XDG_CONFIG_HOME/grexa/settings.json`. Recent-paths/history/profiles move
  to `$XDG_DATA_HOME/grexa/`.
- `AiSearchApiKey` is removed from on-disk settings; the importer routes the
  key to KWallet / Secret Service and never persists plaintext. (Phase 8)
- Window position is dropped; window width/height clamp on import.
- Recent-paths file rename: Grex `search_path_history.json` →
  Grexa `recent_paths.json`. Importer reads either filename.
- `UseWindowsSearchIndex` → `use_file_index`; `EnableDockerSearch` →
  `enable_container_search`.

## CLI

| Grex behavior                                                   | Grexa decision |
| --------------------------------------------------------------- | -------------- |
| Windows path quoting / Powershell escaping in examples          | Replaced with POSIX shell examples. |
| `\\` separators in `--match-files` / `--exclude-dirs`           | Treated as opaque strings. Path-component matching uses `/`. |
| `Console.OutputEncoding = UTF8`                                  | Not needed; standard Linux UTF-8 locale. |
| Exit code semantics (`0` matches, `1` no matches, `2` error)    | Kept. (`grex-search-tab-content-codebehind-audit.md`, PLAN phase 12.) |

## Imports From Grex Backups

The Grex-to-Grexa importer (PLAN phase 10) accepts a `%LocalAppData%\Grex`
directory or zip and translates each file using the rules listed above:

- `settings.json` → schema merge per the storage audit, with Windows-only
  fields stripped and `AiSearchApiKey` routed through the secret store.
- `search_path_history.json` → path translator (drive letter / UNC / WSL).
- `search_history.json` → path translator + dedupe key carried over with
  `True`/`False` casing preserved so identity matches across Grex and Grexa.
- `search_profiles.json` → path translator + ordering preserved + name
  comparison case-insensitive.

## Cancellation And Streaming

This is not a Windows-vs-Linux decision per se, but it is recorded here
because the Rust core's cancellation/streaming model is what makes Phase 4
GUI work plausible without WinUI's Dispatcher pump:

- Every search runs against a `CancelToken` and reports progress through a
  `ProgressEvent` stream. `SearchSummary.cancelled` records whether the
  result is partial. Partial results are kept; memory is not released until
  the summary is dropped. (`crates/grexa-core/src/search.rs`)
- The GUI controller is expected to marshal QObject mutations onto the GUI
  thread; the Rust core itself is thread-pool friendly and does not touch
  Qt types.

## Out Of Scope For 1.0

The following Grex behaviors are intentionally non-applicable for 1.0 and
are not slated for the Linux replacement table above:

- Custom window chrome / non-standard window decoration.
- macOS, Windows, or web target.
- WebView2 / WPF / WinForms compatibility shims.
- ARM Windows.
- Telemetry (Grex has none; Grexa explicitly opts-out unless the user
  enables diagnostics).

## Post-1.0

- Writable container replace: the library implementation (`replace_container`)
  is in place; GUI/CLI integration and a security review of the copy-out/
  replace/copy-back model remain before it is user-facing.
