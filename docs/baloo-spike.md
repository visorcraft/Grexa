<!-- SPDX-FileCopyrightText: 2026 VisorCraft LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-only -->

# Baloo Candidate-Seeding Decision

Grexa evaluated KDE Baloo as a way to seed candidate files before running its
own search engine.

[Baloo]: https://docs.kde.org/stable_kf6/en/plasma-desktop/kcontrol/baloo/

## Current status

Deferred.

The repository contains:

- `BalooAdapter`;
- `NullBalooAdapter`;
- `BaloosearchCliAdapter`;
- test stub support;
- `SearchOptions::use_file_index`;
- CLI compatibility flags `--use-index` and `--no-index`.

The search pipeline does not call the adapter. The setting and flags therefore
have no effect in Grexa 1.11.1. The GUI does not expose a Baloo toggle.

## Why it remains deferred

1. Baloo commonly excludes source trees and hidden directories from its default
   index. Grexa's main workload often gains no candidate reduction.
2. An index can be stale. Grexa must reopen and recheck every candidate for
   correct content, filters, and offsets.
3. `baloosearch6`, `baloosearch`, and `baloo-search` vary across distributions
   and versions. Their directory and output interfaces are not one stable
   contract.
4. Regex searches and comparison modes that Baloo cannot express still require
   the normal walker.
5. Grexa must keep working on non-KDE desktops and minimal packages without a
   Baloo runtime.

## Existing adapter contract

`crates/grexa-core/src/baloo.rs` defines availability, indexed-path, and
candidate-query operations. The null adapter provides the portable default.
The CLI adapter probes known executable names without making Baloo a build-time
dependency.

Unit tests use the null and stub adapters. Current production search does not
select either the CLI adapter or its candidates.

## Conditions for revisiting

Wire candidate seeding only after all of these are true:

- representative benchmarks show a meaningful win;
- a stable interface can be supported across target distributions;
- candidates can be revalidated without changing search semantics;
- failure, stale data, unsupported filters, regex, and non-indexed roots all
  fall back to the normal walker;
- packaging remains optional and non-KDE systems still work.

Until then, `grexa-core` traversal is the only production search source.
