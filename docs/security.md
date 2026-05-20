# Security and Privacy

Grexa is a developer tool that reads (and optionally writes) arbitrary
local files plus optionally sends a small context payload to an AI
endpoint. This doc records the threat model, the data Grexa touches,
and the policies it enforces.

## Telemetry policy

**Grexa ships zero telemetry.** No analytics, no error reports, no
ping-home behavior. Opt-in diagnostics surface as local log files
under `$XDG_STATE_HOME/grexa/`, never as outbound traffic.

A future "diagnostics" feature, if added, must:

- be off by default,
- redact every path / search term before submission,
- surface a one-time consent dialog,
- target a documented, versioned endpoint.

## Outbound traffic

Grexa makes network requests in exactly two situations:

1. **AI Search Chat** — only when `ai_search_enabled` is true AND
   the user clicks the AI button AND they've supplied an endpoint +
   key. The HTTP body is described in
   [grex-ai-search-service-audit.md](grex-ai-search-service-audit.md).
   The set of allowed endpoints is OpenAI-compatible servers; see
   [ai-provider-scope.md](ai-provider-scope.md).
2. **Endpoint test** (`Settings → AI Search → Test endpoint`) —
   a single `GET /v1/models` against the user-supplied endpoint.

Both call paths are gated by `DefaultSettings.ai_search_enabled`.

No other Grexa subsystem opens a socket.

## Local file access

- **Search** reads files the user explicitly points at. Default
  exclusions include `.git`, `vendor`, `node_modules`, `storage/framework`,
  `bin`, `obj`, `sys`, `proc`, `dev`. Override with `--include-system`.
- **Replace** writes via atomic temp-file-then-rename on the same
  filesystem. Permissions are preserved. A journal in
  `$XDG_STATE_HOME/grexa/replace-journal.json` records each file the
  replace pipeline modifies; the journal is deleted on clean exit.
- **Container search** reads container filesystems via `docker exec` /
  `podman exec` (no privileged operations). When `grep` is missing the
  archive mirror writes to
  `$XDG_CACHE_HOME/grexa/container-mirrors/...`; the mirror is
  user-readable only by default and is pruned by
  `prune_mirrors(max_age_secs)`.

## Replace risks

- **No undo.** Replace writes are committed atomically. The journal
  records *which* files were modified but not their previous content.
  Users who need rollback should snapshot the tree first (git stash, btrfs
  snapshot, `cp -r`).
- **Archived documents are never modified.** OOXML / ODF / ZIP / PDF /
  RTF files are extracted read-only for search and skipped by the
  replace pipeline.
- **Containers are read-only.** The container adapter has no replace
  entry point; the search engine refuses to write to a container target.

## Container runtime sockets

Mounting a container socket grants substantial privileges. Grexa never
needs root to use these sockets — it relies on the user's existing
membership in `docker` / `podman` groups (or the rootless Podman
session). Grexa never installs helpers, never writes to a container
during a search, and never elevates privileges on its own.

The Settings UI must surface "this is privileged access" explicitly
when the user enables `enable_container_search`.

## API key handling

API keys for the AI endpoint are stored in the system keyring via the
[`keyring`](https://crates.io/crates/keyring) crate, which on Linux
talks to `org.freedesktop.secrets` (KWallet / GNOME Keyring /
KeePassXC).

- Service id: `io.visorcraft.Grexa.ai`
- Account: canonical endpoint base URL
- Multiple endpoints can each have their own key without overwriting.

**No plaintext fallback.** If the keyring is unavailable (no D-Bus
session, no secret-service daemon), `store_api_key` returns
`SecretError::Backend(_)`. The UI surfaces this verbatim. Users who
want a one-shot key without storing it can paste it into the
endpoint test field; the value lives only in memory.

API keys are excluded from:

- `settings.json` exports
- `tracing` log output (the AI client never logs the value)
- screenshots / diagnostics
- container-search archive mirrors
- the replace journal

## Path redaction in diagnostics

`grexa-cli` logs to `$XDG_STATE_HOME/grexa/grexa.log`. The default
fields are search path, query, regex flag, case sensitivity, gitignore
flag — none of which would be considered a secret by themselves.

If a user enables a privacy mode (a future GUI toggle), the logger
will redact:

- Path prefixes outside the user's home directory
- The search term
- The replacement string
- The detected encoding label for content under `/etc/`, `/var/`,
  `/proc/`, `/sys/`, `/dev/`

This list is in the GUI's Phase 4 deliverables.

## Threat model summary

| Threat | Mitigation |
| ------ | ---------- |
| User searches a tarball-of-malware on their disk | Search reads bytes, never executes. Document extractor pipes through `pdftotext` (separate process), `quick-xml` (no entity expansion), and lossy decoders. The extractor itself never spawns shells or interprets contents. |
| User replaces something they didn't intend | Confirmation dialog before replace (GUI); journal records the change set; atomic rename means files are either fully replaced or untouched. |
| AI endpoint is malicious (returns crafted JSON) | The client parses with `serde_json` (no eval), surfaces errors through the typed `AiSearchResponse` enum, and never executes content. |
| Container search exec-injection | Every argv to `docker exec` / `podman exec` is built as a Rust array, not concatenated; tests pin this. |
| Stale mirror leaks information | `prune_mirrors(max_age_secs)` runs on startup and after each search; mirrors live under the user's cache dir (0700). |
| Keyring not available, user tries to enable AI | `store_api_key` returns `SecretError::Backend(_)`. UI must refuse to fall back to plaintext. |
| Logs accidentally capture an API key | The `AiSearchClient` never logs the key; the keyring layer doesn't log either. |
| Search root is `/` | `/proc`, `/sys`, `/dev`, `/run`, and the system-dir auto-exclusions kick in; tests in `crates/grexa-core/tests/root_safety.rs` pin this. |

## Dependency hygiene

- `cargo-deny` enforces the license allowlist in `deny.toml`.
- `cargo-audit` checks the RustSec advisory database.
- `dependabot.yml` opens weekly PRs.
- `just deny` and `just audit` are pre-PR checks.

## Reporting a vulnerability

Email the security contact in the repository's metadata. We aim to
acknowledge within 72 hours and to publish a patch + advisory within
two weeks of confirming a fix.

Coordinated disclosure: please give us a 30-day window before public
disclosure unless the vulnerability is already actively exploited.
