# Grexa vX.Y.Z

Released: YYYY-MM-DD

## Highlights

- One-sentence headlines, three at most. The audience is busy users
  who scan release notes for what changed.

## New

- User-visible features added in this release. One bullet per feature,
  with a link to the PLAN.md phase or feature doc.

## Changed

- Behavior that existed before but works differently now. Migration
  notes belong here.

## Fixed

- Bugs closed. Reference the issue number or the failing test that
  proves the regression won't return.

## Deprecated

- Anything that still works but is going away. Include the version
  where removal is planned.

## Removed

- Things deleted in this release. Be honest — users grep release
  notes for removed features when they go missing.

## Security

- Advisories, CVE references, or audit findings remediated.

## Performance

- Benchmarks that meaningfully improved or regressed. Be precise:
  workload, hardware, before/after timings.

## Developer notes

- API surface changes that affect downstream embedders. Cargo
  feature flags added or removed.

## Verification

Each row in this section must reference a test or a manual check.
Examples:

- `cargo test --workspace` → 252 tests pass on x86_64 Arch + Fedora
  41.
- `python3 scripts/check_locale_sync.py` → all locales in sync.
- Container smoke test against rootless Podman 5.x — manual run on
  the maintainer's box, no daemon required for CI.
- AppImage smoke: `scripts/post_package_smoke.sh
  target/appimage/Grexa-X.Y.Z-x86_64.AppImage`.
- Flatpak smoke: `flatpak run io.visorcraft.Grexa ~/code TODO`.

## Known issues

- One per bullet, with a workaround if known, and a link to the
  tracking issue.

## Upgrade notes

- Anything users must do manually. Examples: re-import a Grex
  settings.json, regenerate shell completions, rotate stored API
  keys (and how).

## Credits

- Contributors who landed PRs in this release. Translators credited
  per-locale.
- Upstream projects whose recent changes Grexa benefited from
  (encoding_rs, ignore, fluent, …) — optional, do this when a
  specific upstream feature is the reason a Grexa feature became
  possible.

---

Template version: 2026-05-16. Update when the release-notes shape
changes.
