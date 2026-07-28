<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Dependency and License Policy

Grexa is GPL-3.0-only. Every Rust dependency, bundled asset, and shipped runtime
component must be redistributable under compatible terms.

## License boundary

All crates in this repository inherit `GPL-3.0-only`.

`grexa-db` is the sole intentional exception. It is a separately maintained,
Apache-2.0 engine consumed through a pinned git tag. The permissive license
allows the database engine to be embedded outside Grexa.

Three rules protect that boundary:

1. `grexa-db` must not take GPL-only dependencies.
2. `grexa-db` must not depend on `grexa-core` or another Grexa GPL crate.
3. Files in the `grexa-db` repository use `Apache-2.0` SPDX headers. Files in
   this repository use `GPL-3.0-only`.

Apache-2.0 code may flow into GPL-3.0-only Grexa. The reverse dependency would
remove the independent embedding option.

## Accepted dependency licenses

`deny.toml` and `about.toml` must contain the same accepted set:

| License |
| ------- |
| `0BSD` |
| `Apache-2.0` |
| `Apache-2.0 WITH LLVM-exception` |
| `BSD-3-Clause` |
| `CC0-1.0` |
| `CDLA-Permissive-2.0` |
| `GPL-3.0-only` |
| `ISC` |
| `LGPL-2.1-or-later` |
| `MIT` |
| `Unicode-3.0` |
| `Unlicense` |
| `Zlib` |

`cargo-deny` rejects licenses outside this list unless a reviewed,
version-specific clarification is added. There are no clarification entries
today.

## Source policy

Cargo sources are restricted to:

- the crates.io index;
- `https://github.com/visorcraft/grexa-db`.

Unknown registries and unknown git sources fail `cargo deny`. Wildcard
dependency versions are denied. Multiple versions warn because some ecosystem
stacks cannot avoid them safely.

## Direct dependency groups

| Purpose | Direct crates |
| ------- | ------------- |
| Search and traversal | `regex`, `fancy-regex`, `ignore`, `globset` |
| Unicode and encodings | `encoding_rs`, `chardetng`, `unicode-normalization`, `icu_casemap`, `icu_locale_core`, `icu_properties` |
| Documents | `zip`, `quick-xml` |
| Storage and serialization | `grexa-db`, `serde`, `serde_json`, `serde_yaml_ng`, `serde_repr`, `tempfile` |
| CLI | `anyhow`, `clap`, `clap_complete`, `clap_mangen`, `ctrlc` |
| Containers | `libc` |
| AI and secrets | `ureq`, `keyring-core`, `zbus-secret-service-keyring-store` |
| Localization | `fluent`, `unic-langid` |
| GUI bridge | `cxx`, `cxx-qt`, `cxx-qt-lib`, `cxx-qt-build`, `qt-build-utils` |
| GUI integration | `notify` |
| Logging and errors | `tracing`, `tracing-subscriber`, `tracing-appender`, `thiserror` |
| Build metadata | `vergen-git2` |
| Tests | `assert_cmd`, `predicates`, `proptest`, `tempfile` |

Exact versions and transitive crates come from `Cargo.lock`, not this summary.
The generated [third-party supplement](credits-third-party.md) lists every
resolved package and license text.

## Runtime components

Some functionality uses system components rather than Rust crates:

| Component | Role | Typical license family |
| --------- | ---- | ---------------------- |
| Qt 6 | GUI runtime | LGPL-3.0/GPL/commercial |
| KDE Frameworks Kirigami | GUI components | LGPL-2.1-or-later |
| Poppler `pdftotext` | PDF extraction | GPL-2.0-or-later |
| Docker/Podman CLI | Container access | Apache-2.0 and project-specific combinations |
| KWallet/GNOME Keyring Secret Service | Credential storage | component-specific free-software licenses |

Verbatim texts used by the in-app license viewer live under `LICENSES/`.
`RUNTIME_COMPONENTS` in
`apps/grexa-gui/src/qobjects/settings.rs` maps each component to its SPDX
identifier and bundled text.

## Enforcement

| Command | Purpose |
| ------- | ------- |
| `just deny` | Check licenses, sources, bans, and advisories through `cargo-deny --all-features`. |
| `just audit` | Check `Cargo.lock` against RustSec through `cargo-audit`. |
| `just credits` | Regenerate `docs/credits-third-party.md` from `Cargo.lock` through `cargo-about`. |
| `just preflight` | Run `just ci`, deny, audit, and credits regeneration. |

CI runs `cargo-deny` in a dedicated job. Dependabot checks Cargo and GitHub
Actions weekly. Release preparation uses the full preflight gate.

`docs/credits-third-party.md` is generated. Never hand-edit it.

## Adding or updating a dependency

1. Confirm the feature belongs in the smallest owning crate.
2. Check the package source, license expression, maintenance state, and known
   advisories.
3. For `grexa-db`, verify the one-way license boundary remains intact.
4. Update `Cargo.toml`; let Cargo refresh `Cargo.lock`.
5. Run `just deny` and `just audit`.
6. Run `just credits`; review the generated attribution delta.
7. Update `CREDITS.md` or `LICENSES/` when a direct/runtime component changes.
8. Run `just ci`.

When adding an accepted license, update `deny.toml`, `about.toml`, this
document, and the compatibility rationale in the same change.

## Reporting a concern

Open a normal issue for attribution or license-policy gaps. Use the private
process in [Security and privacy](SECURITY.md) when the concern also creates a
security vulnerability.
