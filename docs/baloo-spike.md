# Baloo Candidate-Seeding Spike

Phase 13 evaluated whether Grexa should integrate with KDE's [Baloo] file
indexer as a candidate-seeding accelerator. This doc records the spike
outcome and the keep/defer/drop call.

[Baloo]: https://docs.kde.org/stable_kf6/en/plasma-desktop/kcontrol/baloo/

## Recommendation: **defer, keep the trait surface**

Ship the [`BalooAdapter`](../crates/grexa-core/src/baloo.rs) trait and a
no-op default in v1.0; do **not** enable Baloo seeding by default and do
**not** add a Settings toggle until a future iteration measures real
acceleration on representative source-code trees.

## Why defer

1. **Baloo's index excludes source code by default.** The default
   include list is `$HOME` minus `.cache`, `.local/share`, hidden dotdirs,
   `*.git`, and a long binary-extension list. Code repositories with
   `.git` siblings are silently skipped. Source-code search — Grexa's
   primary use case — gets no benefit from Baloo until the user
   manually adds their `~/code/...` paths to the indexer's include list.

2. **Indexer freshness is loose.** Baloo updates on `inotify` events,
   but bulk operations (`git checkout`, `rg`, `find -delete`) routinely
   leave the index stale for tens of seconds. Grexa's walker delivers
   correctness; Baloo would have to be re-verified anyway, so its
   acceleration is bounded by "candidates per second".

3. **CLI surface is unstable.** Plasma 6's binary is `baloosearch6`;
   earlier installs ship `baloosearch`; Debian backports ship a
   `baloo-search` symlink. The CLI flags differ across versions
   (`-d <dir>` vs `--directory <dir>`), and structured output isn't
   stable. We'd need either:
   - A D-Bus client against `org.kde.baloo.searchIndex` — adds a
     `zbus`-shaped runtime dependency on the Rust side. Worth doing
     only after the spike proves the headline acceleration is real.
   - A C wrapper around `KFileMetaData` — pulls in KDE Frameworks at
     build time, which is fine for distros but heavy for AppImage /
     CI.

4. **Disable-for-regex is a hard rule.** PLAN.md phase 13 line 422
   forbids Baloo prefiltering for regex searches. The vast majority of
   Grex's existing searches are text but the GUI surface (regex toggle,
   regex builder) means the indexer must be invisible to a substantial
   fraction of real searches. The infrastructure cost ends up paying
   off only on the text-mode fraction.

5. **Distros that ship Grexa without KDE Frameworks shouldn't crash.**
   The trait + `NullBalooAdapter` keeps the build green on Fedora's
   GNOME spin, Arch's i3 / Sway profiles, and Ubuntu MATE without
   conditional compilation.

## What landed in this spike

- `crates/grexa-core/src/baloo.rs`:
  - `BalooAdapter` trait with `is_available`, `is_path_indexed`,
    `candidates_for`.
  - `NullBalooAdapter` — always reports unavailable.
  - `BaloosearchCliAdapter` — probes `baloosearch6` / `baloosearch` on
    `$PATH` and shells out for candidates. Used only when the GUI
    explicitly opts in.
  - `StubBalooAdapter` — canned candidate lists for tests.
- `--use-index` / `--no-index` CLI flags (Phase 12) flow into
  `SearchOptions::use_file_index`. Today the search engine ignores the
  field; a future change wires the engine through `BalooAdapter` when
  the flag is set *and* the adapter reports `is_path_indexed(root) ==
  true`.

## Acceptance test for future "keep" decision

Re-evaluate this spike when:

1. We have telemetry — even opt-in — from real Grexa users showing the
   median search exceeds 1 second on a tree Baloo could conceivably
   accelerate (`$HOME` documents, not source repos).
2. Plasma 7 lands with a documented D-Bus contract for the search
   service.
3. A pure-Rust binding (`kde-baloo-rs`?) exists with a permissive license
   compatible with GPL-3.0-only.

Until then the trait stays; the runtime ignores it; the documentation
above is the source of truth for "why isn't Baloo used?"

## Mocked test plan

- `null_adapter_reports_unavailable` — sanity check the no-op path.
- `stub_adapter_returns_canned_candidates` — pins the trait contract.
- `baloosearch_cli_adapter_reports_unavailable_when_binary_missing` —
  pins behavior on systems without KDE so CI doesn't depend on a live
  indexer.

These tests run in CI without any KDE packages installed.
