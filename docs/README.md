<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Grexa Documentation

This index routes users, packagers, contributors, and maintainers to the
authoritative document for each topic.

## Users

| Topic | Document |
| ----- | -------- |
| Product overview, install choices, and quick start | [Repository README](../README.md) |
| Desktop and CLI walkthroughs | [Using Grexa](usage.md) |
| Complete CLI, settings, paths, output, shortcuts, and limits | [Reference](reference.md) |
| Current capability inventory | [Features](features.md) |
| Grex-to-Grexa feature status | [Feature parity](feature-parity.md) |
| Moving concepts and data from Grex | [Migration from Grex](migration-from-grex.md) |
| Accessibility behavior and verification | [Accessibility](accessibility.md) |
| AI endpoint compatibility | [AI provider scope](ai-provider-scope.md) |
| Privacy, local data, subprocesses, and vulnerability reporting | [Security and privacy](SECURITY.md) |

## Builders and packagers

| Topic | Document |
| ----- | -------- |
| Prerequisites, builds, tests, generated artifacts, and packages | [Building and testing](build-and-test.md) |
| Fedora / RHEL RPM details | [Fedora packaging](../packaging/fedora/README.md) |
| Translation catalogs and locale additions | [Translations](translations.md) |
| Dependency license policy | [Dependency license review](dependency-license-review.md) |
| Runtime and crate attribution | [Credits](../CREDITS.md) |
| Generated full dependency license supplement | [Third-party licenses](credits-third-party.md) |

Packaging recipes also live directly under [`packaging/`](../packaging/):

- `appimage/` for AppImage
- `arch/` for Arch and CachyOS
- `debian/` for Debian-family packages
- `fedora/` for Fedora and RHEL-family packages
- `flatpak/` for Flatpak
- `opensuse/` for openSUSE

## Contributors and maintainers

| Topic | Document |
| ----- | -------- |
| Contribution workflow and coding rules | [Contributing](../CONTRIBUTING.md) |
| Workspace boundaries and runtime data flow | [Architecture](architecture.md) |
| Qt/QML, cxx-qt, controllers, and build pipeline | [GUI design](gui-design.md) |
| Linux-specific product decisions | [Linux decisions](linux-decisions.md) |
| Result memory and back-pressure limits | [Memory budgets](memory-budgets.md) |
| Baloo evaluation and current deferred status | [Baloo spike](baloo-spike.md) |

## Upstream behavior audits

Files named `grex-*-audit.md`, plus `grex-models-map.md` and
`grex-strings-migration-matrix.md`, preserve evidence from the upstream Windows
application. They are implementation contracts, not the fastest user
introduction.

Start with [the audit inventory](grex-audit-inventory.md), then open the audit
for the subsystem being changed. If behavior pinned by an audit changes, update
the audit and [Linux decisions](linux-decisions.md) in the same pull request.

## Which document is authoritative?

- Runtime CLI syntax: `grexa-cli --help`, generated from
  `crates/grexa-cli/src/main.rs`.
- Settings defaults: `DefaultSettings::default()` in
  `crates/grexa-core/src/storage.rs`.
- Safety caps: constants in `grexa-core`, `grexa-containers`, and the GUI
  search controller.
- Build commands: [`Justfile`](../Justfile).
- Release artifacts: [release workflow](../.github/workflows/release.yml).
- User behavior: [Features](features.md), [Using Grexa](usage.md), and
  [Reference](reference.md).
- Upstream parity: [Feature parity](feature-parity.md) and the audit set.

When these disagree, code and generated CLI help describe current behavior.
Treat the mismatch as a documentation bug and fix the relevant public document.
