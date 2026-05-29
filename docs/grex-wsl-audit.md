# Grex WSL Service Audit

This document records Grex `Services/WindowsSubsystemLinuxService.cs` behavior
and why it is intentionally non-applicable to Grexa's Linux-native runtime.

Source evidence:

- `Services/WindowsSubsystemLinuxService.cs`
- `Tests/Services/WindowsSubsystemLinuxServiceTests.cs`
- `Services/SearchService.cs`
- `docs/grex-search-service-audit.md`

## Public Service Shape

Grex exposes `WindowsSubsystemLinuxService` as a static helper with:

- `GetDefaultDistributionName()`
- `GetWslDistributions()`
- `TryConvertToNativeWslPath(string mountedPath)`
- `IsLikelyMountedWslPath(string path)`

It also defines a `WslDistribution` model with:

- `Name`
- `Version`
- `Guid`
- `BasePath`
- `State`

The service is not a search implementation by itself. It discovers WSL
distributions and converts some Windows paths before other Grex search paths
delegate work to `wsl.exe`.

Grexa replacement:

- Do not port the service.
- Do not add a WSL distribution model to runtime search.
- Treat Linux paths as native paths handled by normal Grexa search.
- Keep only migration/import handling for old Grex settings or history that
  contain Windows WSL paths.

## Registry Discovery

Grex reads the per-user registry key:

```text
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Lxss
```

`GetDefaultDistributionName()`:

1. Opens the `Lxss` key.
2. Reads `DefaultDistribution`.
3. Opens the subkey named by that GUID.
4. Reads `DistributionName`.
5. Returns null if any value is missing, blank, or inaccessible.

`GetWslDistributions()`:

1. Opens the same `Lxss` key.
2. Enumerates all subkeys.
3. Builds `WslDistribution` records from `DistributionName`, `Version`,
   `BasePath`, and `State`.
4. Returns an empty list on missing registry data or any registry access
   exception.

Grexa replacement:

- There is no Windows registry.
- There is no default WSL distribution to discover.
- Linux home directories, bind mounts, external drives, SMB/NFS mounts, SSHFS
  mounts, and KIO FUSE mounts should be discovered through normal Linux path
  selection and filesystem APIs.

## Mounted WSL Drive Detection

`IsLikelyMountedWslPath` is intentionally narrow. It returns true only for a
drive-letter path whose remainder starts with `\home\` or `/home/`, ignoring
case.

Examples covered by tests:

- `P:\home\user` -> true
- `p:\HOME\user` -> true
- `P:/home/user` -> true
- `C:\Users\user` -> false
- `\\wsl$\Ubuntu\home\user` -> false
- blank path -> false

This is a Windows convenience for users who mounted a WSL filesystem to a
drive letter. It is not a general path classifier.

Grexa replacement:

- Do not classify `/home/...` as WSL.
- Do not treat `/mnt/...` as a WSL marker.
- Do not treat drive-letter strings as local search roots.
- During import, detect likely mounted WSL drive-letter paths and mark them as
  needing user review, because Grexa cannot know which Linux mount or home path
  they were meant to represent.

## Mounted Drive Conversion

`TryConvertToNativeWslPath` accepts a Windows drive-letter path such as:

```text
P:\home\user\project
```

If the input is empty or not a drive-letter path, it returns the original
string. Otherwise it:

1. Removes the drive prefix.
2. Normalizes separators to backslashes.
3. Enumerates WSL distributions from the registry.
4. Tests both native UNC forms for each distribution:

```text
\\wsl.localhost\<distribution>\<relative-path>
\\wsl$\<distribution>\<relative-path>
```

5. Returns the first UNC path that exists as a file or directory.
6. Returns the original path when no matching distribution/path exists.

The conversion is best-effort and non-fatal. It depends on Windows registry
data and Windows UNC access to WSL filesystems.

Grexa replacement:

- Do not perform runtime conversion.
- Do not access `\\wsl$` or `\\wsl.localhost` paths.
- Do not shell out to `wsl.exe`.
- Imported UNC WSL paths should be translated only by explicit migration rules
  or presented to the user as unresolved Windows paths.

## Relation To Grex Search

Grex `SearchService` has its own WSL branch that handles:

- `\\wsl$\...`
- `\\wsl.localhost\...`
- `/mnt/...`
- `\mnt\...`
- most absolute Unix-style paths

That branch converts paths and delegates actual search/replace work to commands
inside a WSL distribution. `WindowsSubsystemLinuxService` supports the Windows
side of this path handling but does not define Grexa behavior.

Grexa replacement:

- Native Linux search is the primary local search path.
- `/home`, `/mnt`, `/media`, `/run/media`, and other mounted directories are
  regular Linux paths.
- Filesystem availability, permissions, hidden-file handling, symlinks,
  gitignore behavior, search text semantics, and replace semantics belong in
  `grexa-core`, not a WSL adapter.

## Import And Migration Requirements

Grexa may import old Grex data from settings, profiles, history, recent paths,
or exported configuration. If that data contains WSL-related paths, Grexa
should preserve enough information for user recovery without pretending the
path is searchable.

WSL-related Windows inputs:

- `\\wsl$\<distribution>\...`
- `\\wsl.localhost\<distribution>\...`
- drive-letter paths likely matching `X:\home\...` or `X:/home/...`
- Grex paths that were automatically routed to WSL, including `/mnt/...`
  entries saved from the Windows app

Recommended import behavior:

- Store the original string.
- Mark it as unresolved or requiring review.
- If a deterministic Linux equivalent exists only by simple slash conversion,
  offer it as a suggestion rather than silently replacing it.
- Do not run WSL detection during normal search.
- Do not recreate Windows-specific distribution state.

## Tests To Replace

Grex WSL service tests verify Windows classifier and pass-through behavior:

- likely mounted WSL drive-letter detection
- non-drive paths returned unchanged by conversion

Grexa should not port these as runtime service tests. Instead, add migration
tests when Grex-to-Grexa import is implemented:

- WSL UNC paths are recognized as unresolved imported paths.
- likely mounted WSL drive-letter paths are flagged for review.
- native Linux `/home/...` and `/mnt/...` paths remain normal searchable paths.
- Windows UNC and drive-letter paths are not accepted as native Linux roots.

## Decision

WSL support is a compatibility layer for a Windows application searching Linux
files through Windows. Grexa runs on Linux, so this layer would be a source of
wrong path classification and unnecessary Windows coupling.

The Grexa implementation should remove WSL concepts from runtime search and
limit WSL awareness to import/migration diagnostics for users coming from Grex.
