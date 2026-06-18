# grexa-db Design Spec

Status: **DRAFT — Phase 0 design.** Revised after adversarial peer review
. Review findings are incorporated below; the original three open
questions are resolved in "Peer review resolutions."

## Purpose

`grexa-db` is a flat-file database engine where records are plain files in a
directory tree and relational joins materialize as directories of symlinks.
The filesystem is the interface: any tool that reads files (`rg`, `grep`,
editors, file managers) is a client without knowing the database exists.

It is the storage layer beneath the Grexa search app, and also an embeddable
library (`Apache-2.0`) for other applications that want a relational layer
over plain-text data they refuse to lock into a binary format.

## Goals

1. **Records are files.** Human-editable, grep-able, diff-able, ownable.
2. **Joins are symlink directories.** Materialized query results live as
   folders of symlinks; the OS is the query executor.
3. **Embeddable.** Pure sync Rust, `Apache-2.0`, `Send + Sync`, no daemon,
   no I/O polls, no GPL deps.
4. **Lazy and fast.** Opening a 100k-record DB must be near-instant.
5. **Plain files in, plain files out.** No proprietary format, ever. If the
   engine disappears tomorrow, every record is still readable.

## Non-goals

1. **No high write throughput.** The engine is read-mostly; writes happen
   via the user's editor.
2. **No multi-writer concurrency control.** Single-process embedding; if
   two processes write, last writer wins at the filesystem level.
3. **No scale past ~250k records per collection.** Past that, filesystems
   get cranky and a real DB wins. Documented hard limit.
4. **No network/replication.** Replication is `git` or `syncthing` — not
   our problem.
5. **No SQL.** The query API is a typed Rust builder, not a SQL parser.
6. **No referential integrity enforcement.** The engine is read-only; it
   can *detect* broken refs but cannot *repair* them. Dangling refs and
   stale views are normal, observable states (see "Path semantics").
7. **No automatic schema migrations.** A separate `validate` command flags
   violations; nothing is rewritten automatically.
8. **No schema enforcement in Phase 0.** Schemas are parsed and accessible
   via `Collection::schema()`, but record fields are NOT validated against
   them at read time. `validate_all()` and typed field access are Phase 1.
   Phase 0 schemas are descriptive (used by view materialization for
   grouping) but not prescriptive.
8. **No tag lifecycle operations.** No tag rename, no tag delete, no tag
   registry. Tags are plain `array<string>` values; editing them is an
   editor operation, not an engine operation.
9. **No transitive joins across more than 2 collections** in Phase 0.
10. **No auto-refresh of materialized views.** Stale views are the default;
    re-materialization is caller-driven. (Future: optional `watch` mode.)

## Storage layout

```
my-db/                         ← Db root (any directory)
  notes/                       ← a Collection (dir + schema.md)
    schema.md                  ← schema + human docs
    2024-transformers.md       ← a Record
    2024-attention.md
  bookmarks/
    schema.md
    rust-lang.org.md
  views/                       ← materialized queries (engine-managed)
    .generations/              ← versioned view content
      gen-a3f1b2/              ← one generation (the actual dir tree)
        transformers/
          2024-transformers.md -> ../../../../notes/2024-transformers.md
      gen-9c4e7d/              ← previous generation (pending GC)
    notes-by-tag -> .generations/gen-a3f1b2   ← symlink (atomic swap target)
  .grexa-db.lock               ← reserved for Phase 1+ cross-process flock (not yet implemented)
```

**Path IS the record identity** (not a stable PK — see "Path semantics"
below for the distinction). Paths are editable, diffable, git-friendly.
There is no separate `id` field.

**One record = one collection.** No multi-membership; cross-collection
queries happen via views.

**Schema discovery is implicit.** Any directory with a `schema.md` is a
collection. No root registry — Unix-y, composable, `cp -r` works.

## Path semantics

Peer review finding #3: the spec contradicted itself by calling path both
"stable" and "breakable on rename." Resolution — be explicit:

- **Path is record identity**, not a stable primary key. A record IS its
  path; renaming a record file creates a new identity (the old path is
  gone). This is delete-plus-insert, not an update.
- **Dangling refs are a normal state.** If record A references record B
  via `ref<T>`, and B is renamed or deleted, the ref in A dangles. The
  engine detects this during validation/query; it does not repair it.
- **Stale views are a normal state.** Materialized views reflect the state
  of the world at materialization time. If records change, the view is
  stale until explicitly re-materialized.
- **Caller responsibility.** Any rename/delete workflow that needs ref or
  view repair must be handled by the caller (a future `migrate`/`repair`
  tool), not by the engine.

## Schema.md format

Schema lives in YAML frontmatter; body is free-form human documentation.
This dogfoods the same frontmatter parser that records use.

```markdown
---
collection: notes
fields:
  - { name: title,   type: string,           required: true }
  - { name: tags,    type: "array<string>" }
  - { name: rating,  type: integer,          range: [1, 5] }
  - { name: read_at, type: date }
  - { name: source,  type: "ref<bookmarks>", optional: true }
---

# Notes

Human-only docs. Schema lives in frontmatter; this body is free-form
explanation that ships with the schema.
```

## Field types

| Type | Example | Notes |
|------|---------|-------|
| `string` | `"hello"` | default |
| `integer` / `float` | `42` / `3.14` | i64 / f64 |
| `boolean` | `true` | |
| `date` / `datetime` | `2024-03-15` | ISO 8601 / RFC 3339 |
| `array<T>` | `[1, 2, 3]` | homogeneous |
| `enum<a\|b\|c>` | `a` | inline |
| `ref<collection>` | `"bookmarks/rust-lang.org.md"` | **weak** DB-root-relative path reference (see below) |

### `ref<T>` is a WEAK reference, not a foreign key

Peer review finding #4: with a read-only engine, `ref<T>` cannot enforce
integrity — it can only *detect* violations. The spec is now honest about
this:

- **Format:** DB-root-relative path with the target collection prefix,
  e.g. `bookmarks/rust-lang.org.md`. See "Reference path safety" for the
  full validation rules.
- **Guarantee:** structural validation on read (correct shape, within
  declared collection, no escapes). Target existence is checked and
  reported as a *diagnostic warning*, not an error — dangling refs are
  valid states.
- **No enforcement.** The engine will not write to records to repair refs,
  cascade deletes, or update paths on rename. Those are caller operations.

## Reference path safety

Peer review finding #5: the original spec didn't define what a ref path is
relative to, and didn't forbid escapes. Full rules:

1. **DB-root-relative.** Refs are resolved against the DB root, not the
   current record's collection.
2. **Must target the declared collection.** A `ref<bookmarks>` value must
   begin with `bookmarks/`. Anything else is a validation error.
3. **No absolute paths.** Starting with `/` is rejected.
4. **No `..` components.** Any path segment equal to `..` is rejected.
5. **No symlink escapes at resolution time.** After resolving, the
   canonical path must remain within the DB root. Symlinks pointing outside
   are rejected.
6. **No backslash separators.** `/` is the only allowed separator.

## Public API (sketch)

```rust
let db = grexa_db::open("my-db")?;
let notes = db.collection("notes")?;                  // parses schema.md lazily

let r = notes.record("2024-transformers.md")?;
let title: &str = r.field("title")?;                  // typed access
let tags: &[String] = r.field("tags")?;
let body: &str = r.body();                            // raw markdown after frontmatter

// Streaming filter query (O(1) memory — yields record by record)
for record in notes.query()
    .filter("rating").ge(4)?
    .filter("read_at").after("2024-01-01")?
{
    // each record loaded lazily; no buffering
}

// Array membership operators (for tags and other array<T> fields)
let tagged = notes.query()
    .filter("tags").contains_any(&["ai", "ml"])?      // any tag matches
    .filter("tags").contains_all(&["rust", "2024"])?; // all tags present

// Buffering query — order_by forces full materialization before yielding
let recent: Vec<Record> = notes.query()
    .filter("rating").ge(4)?
    .order_by("read_at").desc()                       // ← buffers all matches
    .collect()?;

// Materialize a view → directory of symlinks on disk
db.materialize_view(
    "notes-by-tag",           // view name (target: views/notes-by-tag)
    notes.query().filter("rating").ge(4)?,
    "tags",                   // group_by: one subdir per encoded tag value
)?;
```

### Streaming vs buffering (honest semantics)

Filter-only queries read record *content* lazily (O(1) per record body).
File paths are collected eagerly during the directory walk (O(n) in path
count) — this is the path-list phase, not the content phase. `order_by`
forces full materialization of matching records before yielding.

- **Filter-only (lazy content reads):** `filter`, field access. Paths are
  collected up front; file *content* is read one at a time as the iterator
  advances.
- **Buffering operators (O(n) memory):** `order_by`, grouped
  `materialize_view`, and joins. These materialize the full matching set
  before yielding. A query with `order_by` over 100k matching records will
  buffer 100k records — that's inherent to sorting.
- **Future (Phase 1+):** optional external-sort for `order_by` on very
  large result sets, lazy directory walking, and on-disk indexes that make
  `order_by` streaming for indexed fields.

## View materialization semantics

Peer review findings #1, #2: the original "rename over existing dir"
algorithm fails (`ENOTEMPTY` on Linux), and the concurrency model didn't
handle overlapping paths. Both fixed below.

### Atomic swap via generation directories

`rename(2)` over a non-empty directory fails with `ENOTEMPTY`. The
correct pattern is symlink-swap:

1. Build the new view content in `views/.generations/gen-<random>/`.
2. Create a temp symlink: `views/.<name>.swap-<random>` →
   `.generations/gen-<random>`.
3. `rename(2)` the temp symlink over the published name
   (`views/<name>`). Renaming a symlink over an existing symlink (or
   absent target) is **atomic** on Linux/macOS.
4. Readers see either the old generation or the new one — never a missing
   or half-built view.
5. Old generations are garbage-collected once no longer referenced by any
   published symlink.

### Other semantics

- **Idempotent:** re-materializing the same view name replaces it cleanly
  via the swap above.
- **Read-only by convention:** the engine never writes records *through*
  symlinks; views are queries frozen on disk.
- **Stale by default:** symlinks don't auto-update. Caller decides when to
  re-materialize.
- **Last-write-wins** for the same view name (within the locking discipline
  below).

## Concurrency model

Peer review finding #2: a per-`Db` Mutex doesn't protect across multiple
`Db` instances pointing at the same root, and overlapping target paths can
clobber each other.

### Phase 0: single-process, registry-locked

- A process-global registry maps **canonicalized DB root → `Mutex`**.
  All `Db` instances opened against the same root share one mutex.
- View materialization acquires the root's mutex for the full
  build-and-swap. This serializes all view writes within one process.
- **Overlap detection:** before building, check the target view path is
  neither a prefix of, nor prefixed by, any other in-flight materialization
  target. Overlapping targets are rejected with a clear error.

### Phase 1+: cross-process, file-locked

- `flock(2)` on `<db-root>/.grexa-db.lock` serializes view writes across
  processes.
- Documented as the way to embed `grexa-db` in multi-process scenarios.
  Phase 0 explicitly does **not** support safe multi-process view writes.

## Group-by path encoding

Peer review finding #8: tag/field values become directory names, but the
spec never defined how to handle `/`, `..`, empty strings, Unicode, case,
or collisions. Rules:

1. **Percent-encode** any byte not in the URL-safe set `[a-zA-Z0-9._~-]`.
   So `/` → `%2F`, space → `%20`, etc. Reversible.
2. **Reject values that encode to `.` or `..`** as a full segment. These
   are validation errors, not silently mangled.
3. **Reject empty strings.** A tag value of `""` cannot become a directory.
4. **Case-sensitive by default.** `Rust` and `rust` are different
   directories (matches Linux FS semantics). Configurable case-folding is
   a future option.
5. **Length cap:** reject encoded values exceeding 240 bytes (well under
   the 255-byte FS limit, leaving room for any suffix). Phase 1+ may add
   hash-based names with a sidecar mapping for longer values.
6. **Collisions:** with full percent-encoding, two different values only
   collide if they're byte-identical, which is correct behavior.

## Distribution model

Peer review finding #9: `publish = false` in `Cargo.toml` seems to
contradict the "embeddable" goal. It doesn't — it's phased:

- **Phase 0–2:** `publish = false`. Embedding is via path dependency
  (`grexa-db = { path = "..." }`) or git dependency
  (`grexa-db = { git = "..." }`). The crate is not yet on crates.io.
- **Phase 3+:** once the API stabilizes at 1.0, flip `publish = true` and
  release to crates.io. The Apache-2.0 license is what makes this possible.

## Decisions

These are locked (peer-reviewed):

| Decision | Rationale |
|----------|-----------|
| **Engine is read-only.** No `write_record`, no `update_field`. Writes happen via the user's editor; the engine only reads records and materializes views (symlinks only). | Dramatically simpler API, matches "your editor is the write path." Ref repair, tag rename, schema migration are all non-goals for Phase 0. |
| **Path is record identity** (not stable PK). Renames = delete + insert. | Stable, editable, diffable, git-friendly. Dangling refs and stale views are accepted normal states. |
| **Schema discovery is implicit** (any dir with `schema.md` is a collection). | Unix-y, composable, `cp -r` works, no central registry to drift. |
| **Lazy validation.** `Db::open` does no schema validation; `validate_all()` is opt-in. | Fast opens on big DBs. |
| **Queries: streaming filters, buffering terminals.** `filter`/`map` are O(1); `order_by`/`distinct`/`group_by` buffer. | Honest about memory; no false "streaming everything" claim. |
| **`ref<T>` is a WEAK reference.** Detects, never enforces. | Consistent with read-only engine. |
| **Ref format: DB-root-relative path.** Not name-based. | Transparent, inspectable, no engine needed to resolve. |
| **Tags: plain `array<string>`.** Not first-class. | Zero special-casing; grouping works for any array field. |
| **Atomicity: symlink-swap pattern** with generation directories. | `rename(2)` over a non-empty dir fails; symlink-over-symlink is atomic. |
| **Concurrency: per-canonical-root in-process Mutex** (Phase 0); `flock` deferred to Phase 1+. | Serializes view writes within one process; cross-process is NOT yet safe. |
| **Group-by encoding: percent-encode** with `.`/`..`/empty rejection. | Reversible, traversal-safe, collision-free. |
| **`&Db` is `Sync`.** | Embedders expect `Send + Sync`. |
| **License: Apache-2.0** (sole permissive crate in the GPL-3.0 Grexa workspace). | Must stay embeddable in proprietary apps. No GPL deps allowed. |

## Peer review resolutions

The three original open questions, resolved by peer review:

### Q1: Reference format — RESOLVED: path (with safety rules)

**Decision:** DB-root-relative path with collection prefix
(`bookmarks/rust-lang.org.md`).

**Reasoning:** Path refs break on rename, but they're transparent and
compatible with editor-driven writes. Name refs are worse — they require
uniqueness rules, hidden resolution, duplicate handling, and a scan/index
to resolve. Name refs also don't actually fix rename (users rename files
outside the engine). If rename-safe identity matters later, neither path
nor name is enough — you need stable IDs or an engine write/rename API,
which is a Phase 1+ decision.

**Required addition:** the reference path safety rules in
"Reference path safety" above.

### Q2: Tags — RESOLVED: `array<string>` (with operators + encoding)

**Decision:** Plain `array<string>`, not first-class.

**Reasoning:** First-class tags imply lifecycle operations (rename tag,
delete tag, maintain a registry/index, handle orphans) that conflict with
read-only records. Plain arrays support multi-tag queries with explicit
operators (`contains`, `contains_any`, `contains_all`), and grouping fans
out symlinks per value.

**Required additions:** array membership operators in the query API, and
the group-by path encoding rules in "Group-by path encoding" above.

### Q3: Read-only engine — RESOLVED: yes for Phase 0 (with honest non-goals)

**Decision:** Read-only records + write-only-symlinks (views).

**Reasoning:** For Phase 0, read-only is viable: scan files, validate on
demand, query, and explicitly materialize views. Ref integrity on rename,
schema migrations, tag rename/delete, and auto-repair of stale views all
require writes — so they're explicitly listed as non-goals.

A narrow write API (`rename_record`, `rewrite_refs`, `migrate_schema`,
tag edits) is a Phase 1+ decision, to be made when concrete demand
materializes from real Grexa integration.
