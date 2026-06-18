# grexa-db Implementation Plan

Status: **Phase 0–1 + 3 complete. Phase 2 partially complete. Future items
deferred.**

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

## Phase 2 — GUI Integration ⚠️ Partially complete

### Done ✅

| Deliverable | Commit | What |
|-------------|--------|------|
| `DbController` qobject | `c969fd1` | cxx-qt bridge; openDb, collectionNames, recordPaths, validate, materializeView |
| Threading | `e406a5c` | All 3 methods async via `qt_thread().queue()` |
| `DatabasePage.qml` | `c969fd1` | Collection list, record list, validate, materialize |
| Navigation | `c969fd1` | Wired into Main.qml sidebar |
| Review fixes | `df03c7e` | `ItemDelegate` (KF6), `tracing::warn`, row cap |

### Not done ❌

| Deliverable | What it is | Where it goes |
|-------------|------------|---------------|
| **Schema browser page** | Browse a collection's schema.md — show field names, types, required flags, ranges in a readable table. | `qml/DatabasePage.qml` — expand when a collection is selected |
| **Structured-filter sidebar** | Let the user build a query visually: pick a field from the schema, pick an operator (eq/ge/contains), type a value, see filtered results live. | `qml/DatabasePage.qml` — `Controls.Drawer` or inline `ColumnLayout` above the record list |
| **Saved-views navigator** | List existing materialized views (symlinks under `views/`); allow re-materializing or deleting. | `qml/DatabasePage.qml` — a section below materialize, or a separate sub-page |
| **Card/result-grid mode** | Show records as cards with title/tags/rating from frontmatter instead of a flat filename list. | `qml/DatabasePage.qml` — replace the `Repeater` of `ItemDelegate` with a `GridView` or card `Delegate` |

### Implementation notes for completing Phase 2

All four items are **QML-only changes** — no new Rust code needed. The
`DbController` already exposes everything:

- **Schema browser**: `dbController` doesn't expose schema fields yet. Add a
  `#[qinvokable] fn schema_json(collection: &QString) -> QString` to
  `apps/grexa-gui/src/qobjects/db.rs` that returns a JSON array of
  `{name, type, required, range}` from `Collection::schema()`. Parse in QML
  with `JSON.parse()`.

- **Structured-filter sidebar**: Add `#[qinvokable] fn query_records(collection:
  &QString, filterJson: &QString) -> QString` that takes a JSON array of
  `{field, op, value}` objects, builds a `Query` with chained `.filter()`
  calls, and returns newline-separated paths. The QML sidebar collects
  filter rows and calls this on each change.

- **Saved-views navigator**: Read `views/` directory in QML via a new
  `#[qinvokable] fn list_views() -> QString` that returns the symlink names
  under `db_root/views/`. Add a delete button that removes the symlink.

- **Card mode**: The record paths are already available via
  `recordPathsResult`. To show frontmatter fields, add `#[qinvokable] fn
  record_frontmatter(collection: &QString, recordPath: &QString) -> QString`
  that returns the parsed frontmatter as JSON. QML renders cards from the
  parsed fields.

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

| Suite | Tests |
|-------|-------|
| grexa-db | 121 |
| grexa-db-cli | 0 (integration tested via smoke tests) |
| grexa-core (db module) | 20 |
| grexa-core (frontmatter — deleted) | 0 |
| **Total** | **544** |

## Commit history

```
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
