# grexa-db

A flat-file database engine where records are plain files in a directory
tree and relational joins materialize as directories of symlinks. The
filesystem is the interface: any tool that reads files (`rg`, `grep`,
editors, file managers) is a client without knowing the database exists.

## Features

- **Records are files** — human-editable, grep-able, diff-able, git-friendly
- **Joins are symlink directories** — materialized query results live as
  folders of symlinks; the OS is the query executor
- **Typed schema** — YAML frontmatter fields with types (string, integer,
  float, boolean, date, array, enum, ref)
- **Query builder** — filter (`eq`, `ne`, `lt`, `le`, `gt`, `ge`,
  `contains`, `contains_any`, `contains_all`), `order_by`, streaming
- **View materialization** — atomic symlink-swap publishing with
  generation directories and garbage collection
- **Schema validation** — type checking, range enforcement, required fields
- **Embeddable** — pure sync Rust, Apache-2.0, `Send + Sync`, no daemon

## Quick start

```rust
use grexa_db::Db;

let db = Db::open("my-db")?;
let notes = db.collection("notes")?;

// Query with filters
for record in notes.query().filter("rating").ge(4) {
    let r = record?;
    println!("{}", r.field("title").unwrap().as_str().unwrap());
}

// Materialize a view — creates a directory of symlinks
db.materialize_view("high-rated", notes.query().filter("rating").ge(4), Some("tags"))?;
```

## Storage layout

```
my-db/
  notes/
    schema.md           ← schema (YAML frontmatter + human docs)
    2024-transformers.md ← a record
  views/
    .generations/
      gen-<id>/         ← versioned view content
    high-rated -> .generations/gen-<id>  ← published symlink (atomic swap)
```

## CLI

```bash
grexa-db-cli my-db collections
grexa-db-cli my-db query notes --filter rating:ge:4 --order-by rating --direction desc
grexa-db-cli my-db validate
grexa-db-cli my-db materialize notes by-tag --group-by tags
```

## License

Apache-2.0 — embeddable in proprietary applications.
