# Dependency License Review

Grexa is licensed under GPL-3.0-only. Every direct dependency must
ship with a license that is compatible with GPL-3.0-only redistribution.
This doc records the policy, the enforcement mechanisms, and the
review notes for each non-permissive dep.

## Policy

The allowlist lives in [`deny.toml`](../deny.toml). At the time of
writing, permitted licenses are:

- MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception
- BSD-2-Clause, BSD-3-Clause, 0BSD
- ISC, MPL-2.0
- Unicode-DFS-2016, Unicode-3.0
- Zlib, BSL-1.0, CC0-1.0
- GPL-3.0, GPL-3.0-only, GPL-3.0-or-later
- LGPL-2.1-or-later, LGPL-3.0-only, LGPL-3.0-or-later

Anything outside this set is rejected by `cargo-deny check`. Adding a
new license requires updating both `deny.toml` and this doc.

## Enforcement

- `just deny` runs `cargo-deny check`. CI runs the same command
  ([`.github/workflows/ci.yml`](../.github/workflows/ci.yml) job
  `deny`).
- `just audit` runs `cargo-audit` against the RustSec database for
  known vulnerabilities.
- `dependabot.yml` opens PRs for both Rust and GitHub Actions
  dependency updates on a weekly schedule.

## Direct dependencies

| Crate | License | Notes |
| ----- | ------- | ----- |
| `anyhow` | MIT / Apache-2.0 | Used in CLI for error glue. |
| `clap`, `clap_complete`, `clap_mangen` | MIT / Apache-2.0 | CLI parsing. |
| `chardetng` | Apache-2.0 / MIT | Mozilla-origin encoding heuristic. |
| `ctrlc` | MIT | CLI signal handler. |
| `encoding_rs` | Apache-2.0 / MIT | Mozilla encoding decoder. |
| `fancy-regex` | MIT | Extended regex engine. |
| `fluent`, `fluent-bundle` | Apache-2.0 / MIT | Localization runtime. |
| `globset`, `ignore` | Unlicense / MIT | gitignore + glob handling. |
| `keyring` | Apache-2.0 / MIT | Secret-Service wrapper. |
| `quick-xml` | MIT | XML parser for OOXML / ODF. |
| `regex` | Apache-2.0 / MIT | Fast regex engine. |
| `serde`, `serde_json`, `serde_repr` | MIT / Apache-2.0 | Serialization. |
| `tempfile` | Apache-2.0 / MIT | Atomic-rename writes. |
| `thiserror` | Apache-2.0 / MIT | Error derive macro. |
| `tracing`, `tracing-subscriber`, `tracing-appender` | MIT | Logging. |
| `unic-langid` | Apache-2.0 / MIT | BCP-47 parsing. |
| `unicode-normalization` | Apache-2.0 / MIT | Unicode form conversion. |
| `ureq` | Apache-2.0 / MIT | Sync HTTP client. |
| `zip` | MIT | OOXML / ODF / ZIP unpacking. |

Test/dev dependencies (`assert_cmd`, `predicates`, `proptest`,
`tempfile`) are also MIT/Apache-2.0/CC0.

## Transitive dependencies

`cargo-deny check` examines the full transitive tree against the same
allowlist. As of v0.1.0-alpha the tree is clean with one explicit
clarification:

- **`ring`** ships with a custom OpenSSL-derived license. The
  `deny.toml` `[[licenses.clarify]]` entry pins the version Grexa
  actually consumes (via `ureq`'s rustls feature) and asserts the
  combined `MIT AND ISC AND OpenSSL` license. The OpenSSL portion is
  the ChaCha20-Poly1305 implementation, which is compatible with
  GPL-3.0-only redistribution under SFLC's interpretation; ring's
  authors have publicly assented to this use. Re-evaluate when ring
  changes upstream.

### Runtime / system components

Beyond the Rust crate tree, Grexa relies on system runtimes that are
provided by downstream packagers rather than vendored into the source
distribution:

| Component | License |
| --------- | ------- |
| Qt 6 (Core, Qml, Gui, Quick) | LGPL-3.0 |
| KDE Frameworks 6 — Kirigami | LGPL-2.1+ |
| Poppler (`pdftotext`) | GPL-2.0+ |
| Docker / Podman CLI | Apache-2.0 |
| Secret Service backends (KWallet / GNOME Keyring) | various |

The full license texts for these components are bundled under the
top-level [`LICENSES/`](../LICENSES/) directory and surfaced in-app.
The `about.toml` accepted-license set and the `deny.toml` allow list
are kept in sync, so both the crate tree and these system components
clear the same compatibility bar.

## When adding a new dependency

1. Check the crate's license on crates.io.
2. If it's not on the allowlist, decide:
   - Is the functionality essential? Can we vendor a smaller MIT
     equivalent?
   - If essential, add the license to `deny.toml` *and* this doc
     with the justification.
3. Run `just deny` locally before pushing.
4. Update `Cargo.lock`; CI will re-verify.

## Audit cadence

- **Weekly**: dependabot PRs (automated review).
- **Per release**: human pass over `cargo tree --depth 2`; flag any
  new transitive deps that don't show up in this table.
- **Per major version bump of `ureq` / `keyring`**: re-check the
  rustls / secret-service backends, since both pull TLS / D-Bus crates
  whose licenses occasionally shift.

## Security advisories

`cargo-audit` consults <https://github.com/RustSec/advisory-db>. CI
warns on yanked dependencies but does not fail; failures only fire on
unaddressed advisories.

History of fixed advisories:

- (none recorded for v0.1.0-alpha)

## Reporting concerns

If a license review turns up a real conflict (e.g. a new transitive
dep with an incompatible license), open an issue tagged
`license-review`. The PR that lands a remediation should update
`deny.toml` and this doc in the same commit.
