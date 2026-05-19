# Grex DockerSearchService Audit

This document records the behavior of Grex `Services/DockerSearchService.cs`
that Grexa must preserve, replace with Docker/Podman equivalents, or explicitly
drop.

Source evidence:

- `Services/DockerSearchService.cs`
- `Models/DockerContainerInfo.cs`
- `Models/DockerContainerOption.cs`
- `Models/DockerMirrorInfo.cs`
- `Tests/Services/DockerSearchServiceTests.cs`

## Public Service Shape

Grex exposes a singleton-style `DockerSearchService.Instance`, but the class is
also constructible with an injectable `IDockerProcessRunner` and custom mirror
root for tests.

Core public behavior:

- Check Docker CLI availability.
- Discover running containers.
- Check and cache whether `grep` exists inside a container.
- Search directly in a container through Docker exec and grep.
- Mirror a container path to a local directory as fallback.
- Clean up one mirror or prune expired mirrors.

Grexa replacement:

- Split runtime-neutral contracts into `grexa-containers`.
- Support Docker and Podman through runtime adapters.
- Preserve a direct in-container search path and a mirror/archive fallback.
- Avoid a global singleton in core logic; GUI controllers can own runtime state.

## Container Models

`DockerContainerInfo` fields:

- `Id`
- `Name`
- `Image`
- `Status`
- `State`

Derived fields:

- `ShortId`: empty for blank id, otherwise first 12 characters or full shorter id
- `DisplayName`: `Name (ShortId)` when name exists, otherwise `ShortId`
- `ToString()`: display name

`DockerContainerOption` wraps an optional container:

- `Label`
- `Container`
- `IsLocal`: true when `Container == null`

`DockerMirrorInfo` fields:

- `ContainerId`
- `ContainerName`
- `ContainerPath`
- `LocalMirrorPath`
- `LocalSearchPath`
- `CreatedUtc`

Grexa should add a runtime kind (`docker`, `podman`) to container and mirror
records so paths and actions can report their origin.

## Docker Availability

`IsDockerAvailableAsync` runs:

```text
docker version --format "{{.Server.Version}}"
```

It returns true only for exit code 0. Any exception returns false.

`GetContainersAsync` runs:

```text
docker ps --format "{{json .}}"
```

It parses JSON lines. Malformed lines are skipped. Non-zero exit, empty output,
or exceptions return an empty list. Parsed fields are `ID`, `Names`, `Image`,
`State`, and `Status`.

Grexa replacement:

- Docker adapter should keep this behavior where using the Docker CLI.
- Podman adapter should support compatible `podman ps --format "{{json .}}"`.
- Native socket/API implementations can differ internally but should keep the
  same non-fatal discovery behavior.

## Grep Availability

Before direct search, Grex checks `which grep` inside the container through
Docker.DotNet exec APIs.

Behavior:

- Result is cached per container id in `_grepAvailabilityCache`.
- Exit code 0 plus non-empty stdout means available.
- Docker client unavailable, exec failure, or missing grep means false.
- Cache can be cleared for one container or all containers.

Grexa replacement:

- Cache grep availability per runtime plus container id.
- Missing grep should set the search result to "fallback required", not a fatal
  user-visible error.
- Minimal or distroless containers should go directly to mirror/archive fallback.

## Direct Container Search

`SearchInContainerAsync` requires a non-null container and returns an empty
successful result for empty or whitespace search terms.

If Docker API client creation fails, or grep is not available, it returns:

- `Success = false`
- `GrepNotAvailable = true`
- an explanatory `ErrorMessage`

When grep is available:

1. Normalize the container path.
2. Build a `sh -c` command that uses `find`, `xargs`, and `grep`.
3. Execute it through Docker.DotNet container exec.
4. Treat exit code 0 as matches, 1 as no matches, and greater than 1 as error.
5. Treat "not found" or "command not found" stderr as grep unavailable.
6. Parse stdout into `SearchResult` records.
7. Apply match-file filtering as post-processing.
8. If requested, read `.gitignore` patterns from the container and post-filter
   results.

Direct search result object:

- `Success`
- `Results`
- `ErrorMessage`
- `GrepNotAvailable`

Grexa replacement:

- Keep direct exec search read-only.
- Preserve exit code semantics: 0 match, 1 no match, 2+ error.
- Keep the fallback signal distinct from hard errors.
- For Podman, account for rootless socket/CLI behavior and exec output
  differences.

## Grep Command

`BuildGrepCommand` returns:

```text
sh -c "find '<path>' <find filters> -print0 2>/dev/null | xargs -0 -P 4 -r grep <flags> -- '<term>' 2>/dev/null || true"
```

Grep flags:

- `-Hn`: file and line number
- `-E` for regex
- `-F` for fixed string
- `-i` unless case-sensitive
- `-s` to suppress errors
- `-I` to skip binary files at grep level

Find behavior:

- `-maxdepth 1` when subfolders are disabled.
- `-L` before other options when symbolic links are included.
- `-type f`.
- Hidden files excluded with `! -name '.*'`; hidden directories excluded with
  `! -path '*/.*'` when recursive.
- System path exclusions are only added when recursive and system files are not
  included.
- Binary extension exclusions are added when binary files are not included.
- Excluded dirs become `! -path '*/dir/*'` filters when recursive.
- File name include filters are intentionally not pushed into grep/find; they
  are post-processed because grep `--include` was considered slow in containers.
- `xargs -0 -P 4 -r` provides null-safe and parallel grep execution.

Grexa replacement:

- Preserve null-delimited traversal for paths with spaces and special
  characters.
- Detect whether `xargs -r` and `grep -I` are available in BusyBox/minimal
  environments; fall back if not.
- Keep file name filtering post-processing unless a faster portable approach is
  proven.

## Search Filters

Direct search filter behavior:

- Hidden filtering is based on dotfile/dot-directory names.
- System path filtering excludes `/sys/`, `/proc/`, `/dev/`, `/.git/`,
  `/vendor/`, `/node_modules/`, `/storage/framework/`, `/bin/`, and `/obj/`.
- Binary extension list includes common Windows/Linux binaries and libraries:
  `.exe`, `.dll`, `.bin`, `.zip`, `.tar`, `.gz`, `.7z`, `.rar`, image/audio/video
  formats, Office/PDF formats, `.pdb`, `.cache`, `.lock`, `.pack`, `.idx`,
  `.so`, `.dylib`, `.a`.
- Match-file patterns use `|` separators, `-` exclusions, and case-insensitive
  `*`/`?` wildcards.
- Exclude-dir patterns are regex when they start with `^` or contain `(`, `[`,
  or `$`; otherwise they are comma-separated directory names.
- Direct `.gitignore` support reads only `<searchPath>/.gitignore`, skips empty
  and comment lines, ignores negation patterns, and implements a simplified
  wildcard matcher.

Important mismatch:

- The main `SearchService` has richer `.gitignore` behavior through
  `GitIgnoreService`. Docker direct search has simplified `.gitignore`
  filtering. Grexa should not copy the simplified behavior as the final target;
  instead, collect patterns robustly or use a helper strategy and test it.

## Grep Output Parsing

Expected stdout format:

```text
filename:line_number:line_content
```

Parsing uses regex:

```text
^(.+?):(\d+):(.*)$
```

Behavior:

- Null, empty, or whitespace output returns an empty list.
- Lines starting with `Binary file` are skipped.
- Content containing colons is preserved.
- Relative path is calculated by stripping the normalized container root prefix.
- Regex mode compiles a .NET regex to calculate columns and match counts; invalid
  regex falls back to literal matching.
- Text matching uses ordinal or ordinal-ignore-case for column/count/preview.
- Line display is sanitized and truncated to 500 characters plus `...`.
- Preview segments match Grex `SearchService` behavior with a 400 character
  centered snippet and sanitized output.

Grexa parity requirements:

- Preserve container paths in `FullPath`.
- Preserve relative path calculation from the searched container root.
- Keep robust parsing when line content contains colons.
- Consider whether filenames containing colons need stronger parsing than Grex.

## Mirror Fallback

`MirrorPathAsync` requires a non-null container and non-empty container path.
The mirror root defaults to:

```text
%LocalAppData%\Grex\docker-mirrors
```

It creates:

```text
<mirrorRoot>/<container.ShortId>_<guid>
```

Container paths are normalized before use.

When symbolic links are included:

- Grex runs `docker cp "<containerId>:<normalizedPath>" "<mirrorDirectory>"`.
- This preserves symlinks and may fail on Windows without privileges.

When symbolic links are excluded:

- Grex creates a tar archive inside the container using `tar -cf --dereference`.
- It copies that tar out with `docker cp`.
- It extracts locally with `tar.exe`.
- It deletes the local tar and attempts to remove the container tar, even on
  failure.
- `--dereference` copies symlink targets rather than symlink objects.

Failure behavior:

- Mirror directory creation failures are fatal.
- Docker copy/tar failures clean up the mirror directory and rethrow.
- Symlink privilege errors become `DockerSymlinkException` with the original
  Docker error.

`ResolveLocalSearchRoot` returns the extracted last path segment when it exists
under the mirror directory; otherwise it returns the mirror directory itself.

Grexa replacement:

- Use `$XDG_CACHE_HOME/grexa/container-mirrors` for mirror/cache data.
- Preserve path display as container paths even when searching a local mirror.
- Add runtime kind to mirror records.
- For Podman, support rootless UID/GID mappings and archive/cp differences.
- Keep mirror fallback read-only for 1.0.

## Mirror Cleanup

Cleanup behavior:

- Null mirror info or blank/nonexistent local path is a no-op.
- Existing mirror paths are deleted recursively.
- Cleanup exceptions are swallowed.
- `PruneExpiredMirrorsAsync` removes directories older than the retention period.
- Default retention is 6 hours.
- Missing mirror root returns 0 removed directories.
- Directory enumeration failures return 0.
- Cancellation is checked while pruning directories.

Grexa parity requirements:

- Prune expired mirrors after search and on startup.
- Keep cleanup best-effort and non-fatal.
- Log cleanup failures for diagnostics without interrupting search.

## Container Path Normalization

`NormalizeContainerPath`:

- blank input becomes `/`
- trims whitespace
- strips surrounding single or double quotes
- changes backslashes to `/`
- collapses repeated `//`
- prepends `/` if missing

Tests cover:

- `var/www/html` -> `/var/www/html`
- `/var/www//html` -> `/var/www/html`
- `C:\data\logs` -> `/C:/data/logs`

Grexa should preserve this lenient normalization for pasted paths.

## Current Grexa Status Against This Audit

As of the current implementation:

- `grexa-containers` implements Docker/Podman runtime detection, CLI-backed
  container listing, direct in-container grep, mirror/archive fallback, path
  rewriting back to container paths, mirror pruning, and context preview.
- The GUI target selector lists detected Docker/Podman containers when
  container search is enabled and dispatches searches through
  `search_container`.
- Unit tests cover runtime detection, CLI parsing, grep output parsing, direct
  grep invocation, mirror fallback setup, and path rewriting. Live-daemon tests
  are gated behind the `container-live` feature.

Remaining gaps:

- add grep availability caching if repeated probes are a measured problem
- add container `.gitignore` parity if Grexa decides to mirror Grex behavior
  inside containers
- broaden live tests for mirror fallback and cleanup/prune behavior
- add GUI automation for the container target selector and error states
