# Security and Privacy

Grexa searches user-selected files, can rewrite local text files, can invoke
container and desktop helper programs, and can optionally send prompts or
matched file excerpts to a user-configured AI endpoint. This document states
what data crosses each boundary and which protections are enforced.

## Telemetry

Grexa ships no analytics, update check, crash upload, usage ping, advertising
identifier, or background telemetry.

Logs and replace recovery data are local files under
`$XDG_STATE_HOME/grexa/`. Grexa does not upload them.

## Outbound traffic

Grexa itself makes HTTP requests only for explicitly enabled AI features:

| User action | Request | Data |
| ----------- | ------- | ---- |
| Test endpoint or blank-model discovery | `GET <base>/v1/models` | Optional bearer key; no file content |
| Send chat message | `POST <base>/v1/chat/completions` | Model ID, fixed instructions, current search path/query/modes/filter suggestions, and the single typed message |
| Summarize results | `POST <base>/v1/chat/completions` | The same search context plus a fixed summary instruction, full paths, line numbers, and bounded matching-file excerpts |

Every path requires `ai_search_enabled = true` and a direct user action. AI is
off by default. A saved key does not enable requests.

The current chat UI shows local message history, but each typed message is a
standalone request. Previous visible turns are not sent with later messages.
The current search path, query, text/regex mode, Content/Files mode, and filter
suggestions are included as system context on every chat request.

Summary evidence is capped at 400 visible result rows and then packed within
the configured 2,000 through 40,000 character budget. Surrounding lines can
contain sensitive material that was not itself matched. Review the selected
endpoint and search scope before summarizing.

Transport rules:

- bearer keys are attached only to HTTPS URLs or loopback HTTP
  (`localhost`, `127.0.0.1`, `::1`);
- redirects are disabled, preventing credential replay to a redirect target;
- requests time out after 90 seconds;
- response bodies are capped at 4 MiB;
- response text is parsed as data and never executed.

Docker or Podman may communicate with a local or remote daemon according to
the user's runtime configuration, including `DOCKER_HOST`. Grexa invokes the
runtime CLI; it does not independently open the daemon socket.

Opening project links from About/Credits delegates to the desktop URL handler
only after the user selects the link.

## AI credentials

API keys are stored with `keyring-core` through the Linux Secret Service:

```text
service = com.visorcraft.Grexa.ai
account = <canonical endpoint base URL>
```

Compatible backends include KWallet, GNOME Keyring, and KeePassXC's Secret
Service integration.

Protections:

- one credential per endpoint;
- no plaintext settings fallback;
- no key in QML properties;
- no key in `settings.json`, grexa-db records, logs, exports, container
  mirrors, or replace journal;
- `AiSearchConfig` redacts its key in `Debug`;
- remote plaintext HTTP never receives the bearer header.

If the session bus or Secret Service is unavailable, key operations fail with
`SecretError::Backend`. Grexa does not silently save the value elsewhere.

## Local file reads

Search reads files below the user-selected root. It does not execute searched
content.

Default traversal excludes hidden paths, symlink following, common dependency
directories, supported binary/document formats, and pseudo filesystems. Users
can deliberately enable broader traversal.

Safety ceilings include:

- 512 MiB per file;
- 64 directory levels;
- 2 MiB compared per line;
- 1,000,000 result rows;
- bounded regex work.

See [Reference: resource bounds](reference.md#resource-bounds).

### Document extraction

- OOXML, ODF, and ZIP are parsed in-process with bounded entry count and size.
- RTF receives a best-effort non-executing text reduction.
- PDF is passed to the external `pdftotext` process with a 30-second timeout
  and bounded output.

`pdftotext` is additional attack surface for untrusted PDFs. Keep Poppler
updated and avoid searching hostile documents with privileges beyond those
needed.

## Replace

Replace is intentionally destructive.

Protections:

- GUI confirmation is on by default;
- GUI replace reuses the exact options captured by the completed preview
  search;
- CLI supports `--dry-run`;
- archive/document formats are rejected;
- files above 512 MiB are rejected;
- output is written to a temporary file in the destination directory and
  atomically persisted;
- permissions are restored;
- symbolic-link and race-resistant write checks are performed on Linux;
- file, match, and regex-time limits prevent unbounded rewrites.

There is no content backup and no undo. Atomic replacement prevents a torn
individual file; it does not make a multi-file operation transactional.

Use Git, a filesystem snapshot, or a separate backup when rollback matters.

### Replace journal disclosure

While replacement is active, this local file exists:

```text
$XDG_STATE_HOME/grexa/replace-journal.json
```

It contains:

- start time;
- search term;
- replacement text;
- search root;
- regex flag;
- modified paths;
- failed paths.

It is deleted after clean completion. A crash or hard stop leaves it for
recovery review. The privacy path-redaction setting does not rewrite this
journal. Treat the state directory as sensitive if search/replacement strings
are secrets.

## Container boundary

Container search invokes `docker` or `podman` with argv arrays. User terms and
paths are never interpolated into a shell command.

Grexa:

- resolves a requested container against the live container list before
  forwarding an ID;
- uses command time and output caps;
- kills the whole process group on timeout/overflow;
- does not request privileged mode;
- does not modify the container during exposed search workflows.

Access to a Docker socket can be equivalent to host root. Grexa neither grants
that access nor reduces its power. Use a rootless runtime where practical and
do not add users to privileged runtime groups solely for Grexa.

When in-container `grep` is unavailable or cannot express comparison
semantics, Grexa copies the selected path to:

```text
$XDG_CACHE_HOME/grexa/container-mirrors/
```

Those mirrors contain real container file content. Directory permissions
follow the user's XDG directory and process umask; Grexa does not encrypt them.
Mirrors are pruned after CLI searches and periodically by the GUI, but a crash
can leave them until the next cleanup. Delete the directory manually when
handling sensitive containers.

The internal library has a container replacement API. The shipped GUI and CLI
do not expose it.

## External programs and `PATH`

Grexa resolves helper programs from inherited `PATH`:

- `pdftotext`;
- `docker` / `podman`;
- configured editor and `xdg-open`;
- `gdbus`, `gio`, `notify-send`, `wmctrl`;
- `wl-copy` / `xclip`;
- optional desktop cache refresh tools;
- the currently deferred Baloo adapter.

All are started directly with argv arrays, not through a shell. This prevents
shell metacharacters in a path, search term, container ID, or file content from
becoming commands.

Grexa still trusts the selected executable. Launch it with a trusted `PATH`;
do not place attacker-writable directories before system binary directories.

Custom editor commands are tokenized without a shell and substitute only
`{path}`, `{file}`, and `{line}`. They do not perform command substitution,
environment expansion, pipes, or redirection. A user can choose a wrapper
script when those features are needed.

## Output hardening

- Direct terminal output replaces C0/C1 control characters, except tab, with
  U+FFFD. This prevents matched content from injecting terminal escape
  sequences.
- Piped/redirected output is unchanged so downstream programs receive the
  original data.
- CSV cells beginning with common formula characters receive a leading single
  quote to prevent formula execution in spreadsheet applications.
- JSON is serialized through `serde_json`; AI response content is never
  evaluated.

## Logs and diagnostics

CLI log:

```text
$XDG_STATE_HOME/grexa/grexa.log
```

GUI logs:

```text
$XDG_STATE_HOME/grexa/grexa-gui.<date>.log
```

The CLI defaults to `warn`; the GUI defaults to `info`. At `info` or more
verbose levels, logs can contain full search roots, search terms, file paths,
encoding details, and error messages.

`privacy_redact_paths = true` changes GUI file logs by replacing occurrences
of the current `$HOME` path with `~`. It does not:

- redact search terms or replacement text;
- redact paths outside `$HOME`;
- change stderr;
- change CLI logs;
- change the replace journal;
- change exported results.

Review logs before attaching them to a public issue. Prefer the narrowest
useful `GREXA_LOG` filter.

## Plain-file database

Recent paths, history, and profiles are readable Markdown records under:

```text
$XDG_DATA_HOME/grexa/db/
```

This transparency is deliberate, but it also means any process with access to
the user's data directory can read search roots, terms, and profile settings.
Backups and sync tools copy that data as ordinary files.

Materialized grexa-db views are symlink trees. Opening a database in
**Tools → Database** should be treated like opening any user-controlled
directory. View deletion is restricted to validated view paths by grexa-db.

## Threat summary

| Threat | Mitigation and residual risk |
| ------ | ---------------------------- |
| Malicious matched bytes alter terminal state | TTY control-character replacement; redirected bytes remain raw by design. |
| Spreadsheet executes exported match as formula | Formula-like CSV cells are prefixed with `'`. |
| Regex consumes unbounded CPU | Fast engine where possible; extended backtrack/time caps; pathological work can still consume the allowed budget. |
| Search exhausts memory | File, line, match, result, document, output, and GUI snapshot caps. |
| Replace tears a file | Same-directory temporary file and atomic persist. Multi-file replace is not transactional. |
| Replace changes too much | Preview snapshot, confirmation/dry-run, file cap, journal. No undo. |
| Container term/path becomes a command | Direct argv arrays and live-container ID resolution. |
| Container daemon grants host privilege | User controls socket access; prefer rootless Podman. |
| Malicious AI endpoint redirects bearer key | Redirects disabled; key withheld from remote HTTP. Endpoint still receives explicitly sent prompts/evidence. |
| Keyring unavailable | Hard failure; no plaintext fallback. |
| Sensitive data persists after a crash | Replace journal and container mirrors can remain; review/delete XDG state/cache paths. |
| Trojan helper on `PATH` | Use a trusted desktop/session `PATH`; helpers are not bundled in source builds. |
| Logs disclose project details | Conservative log level, documented path, optional limited home-prefix redaction, manual review before sharing. |

## Dependency hygiene

- `Cargo.lock` is committed.
- the Rust toolchain is pinned;
- release builds consume the committed `Cargo.lock`, and packaging paths use
  `--locked` or an offline vendored source tree where required;
- `cargo-deny` enforces license/advisory policy;
- `cargo-audit` checks RustSec;
- Dependabot checks Cargo and GitHub Actions;
- GitHub Actions are commit-pinned;
- third-party credits and license text are bundled;
- linuxdeploy downloads in release CI are checksum-verified.

Run:

```bash
just preflight
```

## Reporting a vulnerability

Do not file a public issue, discussion, or pull request for a security
vulnerability.

Use GitHub private vulnerability reporting:

1. Open the repository's **Security** tab.
2. Select **Report a vulnerability**.
3. Include impact, reproduction, affected version, Linux distribution/desktop,
   relevant configuration, and a minimal proof of concept.

Reports are acknowledged privately, assessed, and coordinated through the
advisory thread. Reporters receive credit unless they request anonymity.
Please allow reasonable time for a fix before public disclosure.
