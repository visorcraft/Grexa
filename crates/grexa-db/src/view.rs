// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! View materialization — the engine's novel feature.
//!
//! A view materializes the result of a query as a directory of symlinks on
//! disk. Any file-reading tool (`rg`, `grep`, a file manager) can then
//! browse the query result without knowing the engine exists.
//!
//! ## Atomicity
//!
//! Views use the **symlink-swap pattern**: content is built in a temporary
//! generation directory, then an atomic `rename(2)` of a symlink publishes
//! it. Readers never see a half-built view. Re-materializing replaces
//! cleanly. If the build fails midway, the generation directory is removed.
//!
//! ## Layout
//!
//! ```text
//! views/
//!   .generations/
//!     gen-<id>/            ← actual view content (built here)
//!       <group>/           ← one subdir per group value (if grouped)
//!         record.md -> ../../../../<collection>/record.md
//!   my-view -> .generations/gen-<id>   ← published symlink (atomic swap)
//! ```

use crate::query::Query;
use crate::record::Record;
use serde_yaml::Value;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

const UNGROUPED: &str = "_ungrouped";

#[derive(Debug, Error)]
pub enum MaterializeError {
    #[error("invalid view name: {0}")]
    InvalidViewName(String),
    #[error("group value is empty")]
    EmptyGroupValue,
    #[error("group value `{0}` is invalid (encodes to `.` or `..`)")]
    InvalidGroupValue(String),
    #[error("group value `{0}` encodes to the reserved name `{UNGROUPED}`")]
    ReservedGroupValue(String),
    #[error("group value `{0}` is too long after encoding (max 240 bytes)")]
    GroupValueTooLong(String),
    #[error("collection directory is not under the database root")]
    CollectionOutsideDb,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("record error: {0}")]
    Record(#[from] crate::record::RecordError),
}

static GEN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Materialize a query result as a directory of symlinks under
/// `views/<view_name>`.
///
/// If `group_by` is `Some(field)`, records are grouped into subdirectories
/// named after the field's value(s). Records missing the field, or with an
/// empty array, go into `_ungrouped/`. If `group_by` is `None`, symlinks
/// are flat. Duplicate group values within a single record are de-duplicated.
///
/// The view is published atomically via the symlink-swap pattern. If the
/// build fails midway, the temporary generation is cleaned up.
pub fn materialize(
    db_root: &Path,
    view_name: &str,
    mut query: Query<'_>,
    group_by: Option<&str>,
) -> Result<(), MaterializeError> {
    validate_view_name(view_name)?;

    let collection_dir = query
        .collection_root()
        .strip_prefix(db_root)
        .map_err(|_| MaterializeError::CollectionOutsideDb)?
        .to_string_lossy()
        .replace('\\', "/");

    let views_dir = db_root.join("views");
    let generations_dir = views_dir.join(".generations");
    fs::create_dir_all(&generations_dir)?;

    let gen_id = gen_id();
    let gen_dir = generations_dir.join(&gen_id);
    fs::create_dir_all(&gen_dir)?;

    let build_result = build_generation(&gen_dir, &collection_dir, &mut query, group_by);

    if let Err(e) = build_result {
        let _ = fs::remove_dir_all(&gen_dir);
        return Err(e);
    }

    let temp_name = format!(".{view_name}.swap-{gen_id}");
    let temp_link = views_dir.join(&temp_name);
    let gen_relative = format!(".generations/{gen_id}");
    symlink(&gen_relative, &temp_link)?;

    let published = views_dir.join(view_name);
    fs::rename(&temp_link, &published)?;

    gc_generations(&views_dir);

    Ok(())
}

fn build_generation(
    gen_dir: &Path,
    collection_dir: &str,
    query: &mut Query<'_>,
    group_by: Option<&str>,
) -> Result<(), MaterializeError> {
    for result in query.by_ref() {
        let record = result?;
        let record_path = record.path();

        let group_values: Vec<GroupValue> = match group_by {
            Some(field) => extract_group_values(&record, field),
            None => vec![GroupValue::Flat],
        };

        for gv in group_values {
            let (group_dir_name, grouped) = match gv {
                GroupValue::Missing => (UNGROUPED.to_string(), true),
                GroupValue::Present(val) => (encode_segment(&val)?, true),
                GroupValue::Flat => (String::new(), false),
            };

            let group_dir = if grouped {
                gen_dir.join(&group_dir_name)
            } else {
                gen_dir.to_path_buf()
            };
            fs::create_dir_all(&group_dir)?;

            let link_path = group_dir.join(record_path);
            if let Some(parent) = link_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let target = symlink_target(grouped, collection_dir, record_path);
            symlink(&target, &link_path)?;
        }
    }

    Ok(())
}

fn validate_view_name(name: &str) -> Result<(), MaterializeError> {
    if name.is_empty() {
        return Err(MaterializeError::InvalidViewName("view name is empty".into()));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(MaterializeError::InvalidViewName(format!(
            "view name `{name}` contains a path separator"
        )));
    }
    if name.starts_with('.') {
        return Err(MaterializeError::InvalidViewName(format!(
            "view name `{name}` starts with `.`"
        )));
    }
    Ok(())
}

fn gen_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = GEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("gen-{nanos:x}-{counter}")
}

enum GroupValue {
    Missing,
    Present(String),
    Flat,
}

fn extract_group_values(record: &Record, field: &str) -> Vec<GroupValue> {
    let raw: Vec<String> = match record.field(field) {
        Some(Value::Sequence(seq)) => seq.iter().filter_map(value_to_string).collect(),
        Some(value) => value_to_string(value).into_iter().collect(),
        None => return vec![GroupValue::Missing],
    };
    if raw.is_empty() {
        return vec![GroupValue::Missing];
    }
    let mut seen: HashSet<String> = HashSet::new();
    raw.into_iter()
        .filter(|v| seen.insert(v.clone()))
        .map(GroupValue::Present)
        .collect()
}

fn value_to_string(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        Some(s.to_string())
    } else if let Some(i) = v.as_i64() {
        Some(i.to_string())
    } else if let Some(f) = v.as_f64() {
        Some(f.to_string())
    } else {
        v.as_bool().map(|b| b.to_string())
    }
}

fn encode_segment(value: &str) -> Result<String, MaterializeError> {
    if value.is_empty() {
        return Err(MaterializeError::EmptyGroupValue);
    }
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    if encoded == "." || encoded == ".." {
        return Err(MaterializeError::InvalidGroupValue(value.into()));
    }
    if encoded == UNGROUPED {
        return Err(MaterializeError::ReservedGroupValue(value.into()));
    }
    if encoded.len() > 240 {
        return Err(MaterializeError::GroupValueTooLong(value.into()));
    }
    Ok(encoded)
}

fn symlink_target(grouped: bool, collection: &str, record_path: &str) -> String {
    let base_depth = if grouped { 4 } else { 3 };
    let extra_depth = record_path.matches('/').count();
    let total_depth = base_depth + extra_depth;
    let ups = "../".repeat(total_depth);
    format!("{ups}{collection}/{record_path}")
}

fn gc_generations(views_dir: &Path) {
    let generations_dir = views_dir.join(".generations");
    let Ok(entries) = fs::read_dir(&generations_dir) else {
        return;
    };

    let mut referenced: HashSet<String> = HashSet::new();
    if let Ok(view_entries) = fs::read_dir(views_dir) {
        for entry in view_entries.flatten() {
            if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false)
                && let Ok(target) = fs::read_link(entry.path())
                && let Some(name) = target.file_name()
            {
                referenced.insert(name.to_string_lossy().into_owned());
            }
        }
    }

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name.starts_with("gen-") && !referenced.contains(&*name) {
                let _ = fs::remove_dir_all(&path);
            }
        }
    }

    if let Ok(view_entries) = fs::read_dir(views_dir) {
        for entry in view_entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') && name.contains(".swap-") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    const NOTES_SCHEMA: &str = "---\ncollection: notes\nfields:\n  - { name: title, type: string }\n  - { name: tags, type: \"array<string>\" }\n  - { name: rating, type: integer }\n---\n";

    fn make_db() -> TempDir {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(
            notes.join("alpha.md"),
            "---\ntitle: Alpha\ntags: [rust, db]\nrating: 5\n---\nAlpha body.\n",
        )
        .unwrap();
        fs::write(
            notes.join("beta.md"),
            "---\ntitle: Beta\ntags: [python]\nrating: 3\n---\nBeta body.\n",
        )
        .unwrap();
        fs::write(
            notes.join("gamma.md"),
            "---\ntitle: Gamma\ntags: [rust, ai]\nrating: 4\n---\nGamma body.\n",
        )
        .unwrap();
        fs::write(
            notes.join("delta.md"),
            "---\ntitle: Delta\nrating: 1\n---\nDelta body (no tags).\n",
        )
        .unwrap();
        dir
    }

    fn resolve_view(dir: &TempDir, name: &str) -> PathBuf {
        let link = dir.path().join("views").join(name);
        let target = fs::read_link(&link).unwrap();
        dir.path().join("views").join(&target)
    }

    #[test]
    fn flat_view_creates_symlinks() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("all-notes", notes.query(), None)
            .unwrap();

        let view = resolve_view(&dir, "all-notes");
        assert!(view.join("alpha.md").exists());
        assert!(view.join("beta.md").exists());
        assert!(view.join("gamma.md").exists());
        assert!(view.join("delta.md").exists());
    }

    #[test]
    fn grouped_view_by_tags() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("by-tag", notes.query(), Some("tags"))
            .unwrap();

        let view = resolve_view(&dir, "by-tag");
        assert!(view.join("rust").join("alpha.md").exists());
        assert!(view.join("rust").join("gamma.md").exists());
        assert!(view.join("python").join("beta.md").exists());
        assert!(view.join("ai").join("gamma.md").exists());
        assert!(view.join("db").join("alpha.md").exists());
    }

    #[test]
    fn grouped_view_missing_field_goes_to_ungrouped() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("by-tag", notes.query(), Some("tags"))
            .unwrap();

        let view = resolve_view(&dir, "by-tag");
        assert!(view.join("_ungrouped").join("delta.md").exists());
    }

    #[test]
    fn symlinks_resolve_to_correct_files() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("all-notes", notes.query(), None)
            .unwrap();

        let view = resolve_view(&dir, "all-notes");
        let alpha_content = fs::read_to_string(view.join("alpha.md")).unwrap();
        assert!(alpha_content.contains("Alpha body."));
    }

    #[test]
    fn re_materialize_replaces_atomically() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("my-view", notes.query(), None).unwrap();
        let first_target = fs::read_link(dir.path().join("views").join("my-view")).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(2));

        db.materialize_view("my-view", notes.query(), None).unwrap();
        let second_target = fs::read_link(dir.path().join("views").join("my-view")).unwrap();

        assert_ne!(first_target, second_target);
    }

    #[test]
    fn filtered_view_only_includes_matches() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("high-rated", notes.query().filter("rating").ge(4), None)
            .unwrap();

        let view = resolve_view(&dir, "high-rated");
        assert!(view.join("alpha.md").exists());
        assert!(view.join("gamma.md").exists());
        assert!(!view.join("beta.md").exists());
        assert!(!view.join("delta.md").exists());
    }

    #[test]
    fn empty_result_set_creates_empty_view() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("empty", notes.query().filter("rating").ge(100), None)
            .unwrap();

        assert!(dir.path().join("views").join("empty").is_symlink());
    }

    #[test]
    fn view_name_validation_rejects_path_traversal() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        assert!(
            db.materialize_view("../escape", notes.query(), None)
                .is_err()
        );
        assert!(db.materialize_view("sub/dir", notes.query(), None).is_err());
        assert!(db.materialize_view(".hidden", notes.query(), None).is_err());
    }

    #[test]
    fn group_value_encoding_handles_special_chars() {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(
            notes.join("special.md"),
            "---\ntags: [\"a/b\", \"x y\", \"café\"]\nrating: 1\n---\nbody\n",
        )
        .unwrap();

        let db = Db::open(dir.path()).unwrap();
        let coll = db.collection("notes").unwrap();

        db.materialize_view("encoded", coll.query(), Some("tags"))
            .unwrap();

        let view = resolve_view(&dir, "encoded");
        assert!(view.join("a%2Fb").join("special.md").exists());
        assert!(view.join("x%20y").join("special.md").exists());
        assert!(view.join("caf%C3%A9").join("special.md").exists());
    }

    #[test]
    fn duplicate_group_values_dont_error() {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(notes.join("dupe.md"), "---\ntags: [rust, rust, rust]\nrating: 5\n---\nbody\n")
            .unwrap();

        let db = Db::open(dir.path()).unwrap();
        let coll = db.collection("notes").unwrap();

        db.materialize_view("deduped", coll.query(), Some("tags"))
            .unwrap();

        let view = resolve_view(&dir, "deduped");
        assert!(view.join("rust").join("dupe.md").exists());
    }

    #[test]
    fn empty_array_goes_to_ungrouped() {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(notes.join("empty.md"), "---\ntags: []\nrating: 5\n---\nbody\n").unwrap();

        let db = Db::open(dir.path()).unwrap();
        let coll = db.collection("notes").unwrap();

        db.materialize_view("test", coll.query(), Some("tags"))
            .unwrap();

        let view = resolve_view(&dir, "test");
        assert!(view.join("_ungrouped").join("empty.md").exists());
    }

    #[test]
    fn literal_ungrouped_tag_is_rejected() {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(notes.join("literal.md"), "---\ntags: [_ungrouped]\nrating: 5\n---\nbody\n")
            .unwrap();

        let db = Db::open(dir.path()).unwrap();
        let coll = db.collection("notes").unwrap();

        let result = db.materialize_view("test", coll.query(), Some("tags"));
        assert!(
            result.is_err(),
            "literal `_ungrouped` tag must be rejected, not silently bucketed"
        );
    }

    #[test]
    fn old_generations_are_garbage_collected() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view("my-view", notes.query(), None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        db.materialize_view("my-view", notes.query(), None).unwrap();

        let generations_dir = dir.path().join("views").join(".generations");
        let gen_count = fs::read_dir(&generations_dir)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("gen-")
            })
            .count();

        assert_eq!(gen_count, 1, "old generation should be GC'd");
    }

    #[test]
    fn collection_dir_name_not_schema_name() {
        let dir = TempDir::new().unwrap();
        let my_notes = dir.path().join("my-notes");
        fs::create_dir(&my_notes).unwrap();
        fs::write(
            my_notes.join("schema.md"),
            "---\ncollection: notes\nfields:\n  - { name: title, type: string }\n---\n",
        )
        .unwrap();
        fs::write(my_notes.join("a.md"), "---\ntitle: Alpha\n---\nAlpha body.\n").unwrap();

        let db = Db::open(dir.path()).unwrap();
        let coll = db.collection("my-notes").unwrap();
        assert_eq!(coll.name(), "notes");

        db.materialize_view("test", coll.query(), None).unwrap();

        let view = resolve_view(&dir, "test");
        let content = fs::read_to_string(view.join("a.md")).unwrap();
        assert!(content.contains("Alpha body."));
    }

    #[test]
    fn grouped_filtered_view_combines_correctly() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();

        db.materialize_view(
            "rust-high-rated",
            notes
                .query()
                .filter("rating")
                .ge(4)
                .filter("tags")
                .contains_any(&["rust"]),
            Some("tags"),
        )
        .unwrap();

        let view = resolve_view(&dir, "rust-high-rated");
        assert!(view.join("rust").join("alpha.md").exists());
        assert!(view.join("rust").join("gamma.md").exists());
        assert!(!view.join("python").exists());
    }

    #[test]
    fn symlink_target_depth_is_correct() {
        assert_eq!(symlink_target(true, "notes", "alpha.md"), "../../../../notes/alpha.md");
        assert_eq!(symlink_target(false, "notes", "alpha.md"), "../../../notes/alpha.md");
        assert_eq!(
            symlink_target(true, "notes", "2024/03/deep.md"),
            "../../../../../../notes/2024/03/deep.md"
        );
    }

    #[test]
    fn encode_segment_rejects_dots_and_reserved() {
        assert!(encode_segment(".").is_err());
        assert!(encode_segment("..").is_err());
        assert!(encode_segment("").is_err());
        assert!(encode_segment("_ungrouped").is_err());
    }

    #[test]
    fn encode_segment_preserves_safe_chars() {
        assert_eq!(encode_segment("rust").unwrap(), "rust");
        assert_eq!(encode_segment("hello-world").unwrap(), "hello-world");
        assert_eq!(encode_segment("file_v2.0").unwrap(), "file_v2.0");
        assert_eq!(encode_segment("café").unwrap(), "caf%C3%A9");
    }
}
