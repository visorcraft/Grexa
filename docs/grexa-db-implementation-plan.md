# grexa-db Implementation Plan

Status: **Phase 0–3 complete. Future items deferred.**

## Related documents

- **[`docs/grexa-db-design.md`](grexa-db-design.md)** (17 KB) — the full
  design spec: storage layout, schema format, field types, API, view
  materialization semantics, concurrency model, peer-review resolutions,
  all decisions.
- **[`crates/grexa-db/README.md`](../crates/grexa-db/README.md)** (2 KB) —
  quick-start examples for library users.

## Phase 0 — Engine ✅

| Module | Tests | What |
|--------|-------|------|
| `frontmatter` | 17 | YAML extraction (BOM/CRLF/edge cases) |
| `schema` | 20 | Type system (string/int/float/bool/date/array/enum/ref) |
| `record` | 5 | Frontmatter + body + field access |
| `collection` | 10 | Lazy record iteration, noise filtering, path-traversal guard |
| `db` | 11 | Root discovery, multi-collection, per-root Mutex |
| `query` | 17 | 9 filter operators, streaming filters, buffering `order_by` |
| `view` | 14 | Symlink-swap materialization, GC, group encoding |
| **Total** | **94** | 4 claude review passes, all critical/high fixed |

Commits: `9c9daaf` → `22c0bcf`

## Phase 1 — Validation + CLI ✅

| Deliverable | Tests | What |
|-------------|-------|------|
| `validation.rs` | 13 | Type checking, range enforcement, required fields, null=missing |
| `validate_all` | — | On Collection and Db |
| `grexa-db-cli` | — | `collections`, `records`, `query --filter`, `validate`, `materialize` |

Commits: `28d11ca` → `ce7536f`

## Phase 2 — GUI Integration ✅

### Initial integration

| Deliverable | Commit | What |
|-------------|--------|------|
| `DbController` qobject | `c969fd1` | cxx-qt bridge; openDb, collectionNames, recordPaths, validate, materializeView |
| Threading | `e406a5c` | record_paths / validate / query / materialize async via `qt_thread().queue()` |
| `DatabasePage.qml` | `c969fd1` | Collection list, record list, validate, materialize |
| Navigation | `c969fd1` | Wired into Main.qml sidebar |
| Review fixes | `df03c7e` | `ItemDelegate` (KF6), `tracing::warn`, row cap |

### Completed Phase 2 features (`f849b5c`)

| Deliverable | Invokable(s) added | What |
|-------------|--------------------|------|
| **Schema browser page** | `schema_json` | Returns a JSON array of `{name, type, required}` from `Collection::schema()`; QML renders a field table. |
| **Structured-filter sidebar** | `query_records` | Takes a JSON array of `[field, op, value]`, builds a typed `Query` with chained `.filter()` calls, returns matching paths (async, capped at 500). |
| **Saved-views navigator** | `list_views`, `delete_view` | Lists published view symlinks under `views/`; delete removes a view symlink. |
| **Card/result-grid mode** | `record_frontmatter` | `RecordCard.qml` shows title/tags/rating via on-demand `Record::frontmatter_json()` expansion. |

### Post-review hardening (working tree — pending commit)

Two correctness/security findings from the final peer review, fixed:

- **`delete_view` path-traversal sink** — the invokable joined an
  unsanitized `view_name` onto `views/`, so an absolute or `..` argument
  escaped the directory (arbitrary-file deletion). Now gated on
  `is_safe_view_name()` (rejects separators, dotfiles, traversal) **and**
  requires the target to actually be a symlink. (`apps/grexa-gui/src/qobjects/db.rs`)
- **`schema_json` omits `range`** — the original note specified
  `{name, type, required, range}`; the shipped invokable dropped `range`.
  Now emits `"range": [min, max]` for numeric fields with a range, else
  `null`. (`apps/grexa-gui/src/qobjects/db.rs`)

> **Note:** the new GUI reads (`schema_json`, `list_views`,
> `record_frontmatter`) run synchronously on the UI thread, unlike the four
> threaded methods. Acceptable for single small reads; revisit if card mode
> fans out many `record_frontmatter` calls.

## Phase 3 — Dogfooding + Publish ✅

| Deliverable | Commit | What |
|-------------|--------|------|
| `RecentPathsDb` | `0959e7d` | Replaces `RecentPathStore` in Workspace |
| `SearchHistoryDb` | `cb6da24` | Replaces `SearchHistoryStore` in Workspace |
| `SearchProfilesDb` | `d51da6e` | Replaces `SearchProfileStore` in Workspace |
| JSON migrations | `adbbe35`, `1fe9ce8` | Auto-import all 3 JSON files on startup |
| `serde_yaml_ng` | `8e55335` | Package rename from archived `serde_yaml` |
| `publish = true` | `adbbe35` | Crate ready for crates.io |
| Critical YAML fix | `d0f882e` | `make_frontmatter()` via `serde_yaml::to_string` |
| Over-engineering cleanup | `26e6d19` | -77 lines (dead frontmatter module, is_optional, VERSION test, as_f64 dedup) |

**8 round-trip tests** for the two new stores (SearchHistoryDb: 4,
SearchProfilesDb: 4). All three stores use proper YAML serialization.

## Deferred to Phase 1+ / Future

These items are documented in `docs/grexa-db-design.md` as future work
and were **never part of the committed roadmap**:

| Item | Design doc section | Why deferred |
|------|-------------------|--------------|
| Cross-process `flock` | Concurrency model | Phase 0 uses in-process Mutex; flock is Phase 1+ |
| Watch mode (inotify) | View materialization | Stale-by-default is the Phase 0 contract |
| External sort for `order_by` | Streaming vs buffering | Current buffering handles the 250k ceiling |
| On-disk indexes | Streaming vs buffering | No indexed fields yet; linear scan is fast enough |
| WASM build | Distribution model | Phase 3+; needs `serde_yaml_ng` WASM compat check |
| Split into separate repo | Distribution model | Phase 3+; stays in grexa workspace until API stabilizes |
| Actual `cargo publish` | Distribution model | `publish = true` is set; manual step requiring API token |
| `distinct` operator | Query builder | Not in the original API sketch; add when needed |

## Test counts

Measured via `cargo test --workspace` (counts include the post-review
hardening tests: `ref_wrong_collection_rejected`, `ref_existence_diagnostics`,
`view_name_safety`).

| Suite | Tests |
|-------|-------|
| grexa-db | 121 unit + 1 doctest |
| grexa-db-cli | 0 (integration tested via CLI smoke tests) |
| grexa-core (db module) | 16 |
| grexa-core (frontmatter — deleted) | 0 |
| **Total (workspace, passing)** | **547** |

## Commit history

```
(working tree, uncommitted) Post-review hardening: delete_view path guard, ref<T> collection + existence/escape diagnostics (Severity::{Error,Warning})
f849b5c Complete Phase 2: schema browser, structured filters, view navigator, record cards
26e6d19 Cut over-engineering: dead frontmatter module, is_optional, VERSION test, as_f64 dedup
1fe9ce8 Complete final polish: thread materialize_view, migrations, round-trip tests
d0f882e Fix CRITICAL: serde_yaml serialization instead of hand-formatted YAML
d51da6e Complete all deferred: SearchProfilesDb, frontmatter utilities, serde_yaml_ng re-export
cb6da24 Dogfood search history: SearchHistoryDb replaces SearchHistoryStore
adbbe35 Add JSON migration, publish=true, README, fmt/clippy fixes
e406a5c Thread DbController: record_paths and validate on worker threads
8e55335 Migrate serde_yaml (archived) to serde_yaml_ng via package rename
b9672c2 Fix CLI filter validation + update stale recent_paths.json docs
7b26b75 Complete remaining wiring: production RecentPathsDb, DbController threading, CLI query
449013c Fix Phase 3 review: atomic writes, filename collision, single-writer doc
0959e7d Phase 3: dogfooding — grexa-core uses grexa-db for recent paths storage
df03c7e Fix Phase 2 review: BasicListItem→ItemDelegate, tracing::warn, row cap
c969fd1 Phase 2: grexa-db GUI integration — DbController qobject + DatabasePage
ce7536f Fix Phase 1 review findings: null=missing, date validation, NaN, double-error
28d11ca Phase 1: schema validation + grexa-db-cli
22c0bcf Clarify record() security doc comment
70d5d2c Fix intermediate-directory symlink escape in Collection::record()
5db8276 Fix 4 remaining review findings: ungrouped sentinel bypass, clippy gate
9aab5b6 Fix all 20 peer-review findings (claude) in grexa-db
3dcd3be Implement grexa-db query builder and view materialization
ee86431 Implement grexa-db read stack: frontmatter, schema, Record, Collection, Db
9c9daaf Add grexa-db crate scaffolding and design spec
```
