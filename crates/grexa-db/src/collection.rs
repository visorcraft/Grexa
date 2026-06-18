// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! A collection — a directory with a `schema.md` and zero or more record
//! files.

use crate::record::{Record, RecordError};
use crate::schema::Schema;
use std::fs;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectionError {
    #[error("failed to read schema.md: {0}")]
    ReadSchema(String),
    #[error("schema error: {0}")]
    Schema(#[from] crate::schema::SchemaError),
}

/// A typed collection of records within a [`Db`](crate::db::Db).
pub struct Collection {
    root: PathBuf,
    schema: Schema,
}

impl Collection {
    /// Open a collection directory. The directory must contain a
    /// `schema.md` file.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, CollectionError> {
        let dir = dir.as_ref();
        let schema_path = dir.join("schema.md");
        let schema_content = fs::read_to_string(&schema_path)
            .map_err(|e| CollectionError::ReadSchema(e.to_string()))?;
        let schema = Schema::from_markdown(&schema_content)?;
        Ok(Self {
            root: dir.to_path_buf(),
            schema,
        })
    }

    /// Collection name (from the schema's `collection` field).
    pub fn name(&self) -> &str {
        &self.schema.collection
    }

    /// The on-disk root directory of this collection.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The parsed schema.
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Read a single record by its collection-relative path.
    ///
    /// The path is validated against traversal attacks: absolute paths,
    /// `..` components, backslashes, and symlinks are rejected.
    pub fn record(&self, relative_path: &str) -> Result<Record, RecordError> {
        validate_relative_path(relative_path)?;
        let full = self.root.join(relative_path);
        let metadata = fs::symlink_metadata(&full).map_err(|e| RecordError::ReadFile {
            path: relative_path.to_string(),
            reason: e.to_string(),
        })?;
        if metadata.file_type().is_symlink() {
            return Err(RecordError::InvalidPath(format!(
                "`{relative_path}` is a symlink; symlinks are not followed"
            )));
        }
        let content = fs::read_to_string(&full).map_err(|e| RecordError::ReadFile {
            path: relative_path.to_string(),
            reason: e.to_string(),
        })?;
        Record::from_content(relative_path, &content)
    }

    /// Iterate all records in this collection, lazily reading each file.
    ///
    /// File paths are collected eagerly (directory walk); file *content* is
    /// read lazily as the iterator advances. Files named `schema.md`,
    /// hidden files, symlinks, and common noise directories
    /// (`node_modules`, `target`, `__pycache__`) are skipped.
    pub fn records(&self) -> RecordIter<'_> {
        let paths = collect_record_paths(&self.root);
        RecordIter {
            collection: self,
            paths: paths.into_iter(),
        }
    }

    /// Begin a typed query over this collection's records.
    pub fn query(&self) -> crate::query::Query<'_> {
        crate::query::Query::new(self)
    }
}

/// Lazy iterator over records in a collection.
pub struct RecordIter<'a> {
    collection: &'a Collection,
    paths: std::vec::IntoIter<PathBuf>,
}

impl Iterator for RecordIter<'_> {
    type Item = Result<Record, RecordError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.next().map(|full| {
            let relative = full
                .strip_prefix(&self.collection.root)
                .unwrap_or(&full)
                .to_string_lossy()
                .replace('\\', "/");
            let content = fs::read_to_string(&full).map_err(|e| RecordError::ReadFile {
                path: relative.clone(),
                reason: e.to_string(),
            })?;
            Record::from_content(relative, &content)
        })
    }
}

impl FusedIterator for RecordIter<'_> {}

fn validate_relative_path(path: &str) -> Result<(), RecordError> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(RecordError::InvalidPath(path.to_string()));
    }
    for component in path.split('/') {
        if component == ".." {
            return Err(RecordError::InvalidPath(path.to_string()));
        }
    }
    Ok(())
}

fn collect_record_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                if matches!(name, "node_modules" | "target" | "__pycache__") {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if name == "schema.md" {
                continue;
            }
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    const NOTES_SCHEMA: &str = "---\ncollection: notes\nfields:\n  - { name: title, type: string, required: true }\n  - { name: tags, type: \"array<string>\" }\n  - { name: rating, type: integer, range: [1, 5] }\n---\n\n# Notes\n";

    fn make_collection() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(
            dir.path().join("note-a.md"),
            "---\ntitle: Alpha\ntags: [rust, db]\nrating: 5\n---\nAlpha body.\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("note-b.md"),
            "---\ntitle: Beta\ntags: [python]\nrating: 3\n---\nBeta body.\n",
        )
        .unwrap();
        fs::write(dir.path().join("readme.txt"), "No frontmatter, still a record.\n").unwrap();
        dir
    }

    #[test]
    fn open_reads_schema() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        assert_eq!(coll.name(), "notes");
        assert_eq!(coll.schema().fields.len(), 3);
    }

    #[test]
    fn open_missing_schema_errors() {
        let dir = TempDir::new().unwrap();
        assert!(Collection::open(dir.path()).is_err());
    }

    #[test]
    fn read_single_record() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        let r = coll.record("note-a.md").unwrap();
        assert_eq!(r.field("title").unwrap().as_str(), Some("Alpha"));
        assert_eq!(r.body(), "Alpha body.\n");
    }

    #[test]
    fn read_nonexistent_record_errors() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        assert!(coll.record("nonexistent.md").is_err());
    }

    #[test]
    fn path_traversal_rejected() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        assert!(coll.record("../etc/passwd").is_err());
        assert!(coll.record("/etc/passwd").is_err());
        assert!(coll.record("..\\..\\secret").is_err());
        assert!(coll.record("notes/../secret").is_err());
        assert!(coll.record("").is_err());
    }

    #[test]
    fn record_rejects_symlink() {
        let dir = make_collection();
        symlink("/etc/passwd", dir.path().join("evil.md")).unwrap();
        let coll = Collection::open(dir.path()).unwrap();
        assert!(coll.record("evil.md").is_err());
    }

    #[test]
    fn nested_subdirectory_record_accessible() {
        let dir = make_collection();
        fs::create_dir_all(dir.path().join("2024").join("03")).unwrap();
        fs::write(
            dir.path().join("2024").join("03").join("deep.md"),
            "---\ntitle: Deep\n---\nbody\n",
        )
        .unwrap();
        let coll = Collection::open(dir.path()).unwrap();
        let r = coll.record("2024/03/deep.md").unwrap();
        assert_eq!(r.field("title").unwrap().as_str(), Some("Deep"));
    }

    #[test]
    fn iterate_all_records() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(records.len(), 3);
        let names: Vec<&str> = records.iter().map(|r| r.path()).collect();
        assert!(names.contains(&"note-a.md"));
        assert!(names.contains(&"note-b.md"));
        assert!(names.contains(&"readme.txt"));
    }

    #[test]
    fn schema_md_is_excluded_from_records() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert!(!records.iter().any(|r| r.path() == "schema.md"));
    }

    #[test]
    fn hidden_files_are_excluded() {
        let dir = make_collection();
        fs::write(dir.path().join(".hidden.md"), "secret\n").unwrap();
        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert!(!records.iter().any(|r| r.path().contains(".hidden")));
    }

    #[test]
    fn nested_subdirectories_are_walked() {
        let dir = make_collection();
        fs::create_dir_all(dir.path().join("2024").join("03")).unwrap();
        fs::write(
            dir.path().join("2024").join("03").join("deep.md"),
            "---\ntitle: Deep\n---\nbody\n",
        )
        .unwrap();
        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert!(records.iter().any(|r| r.path() == "2024/03/deep.md"));
    }

    #[test]
    fn noise_directories_are_skipped() {
        let dir = make_collection();
        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        fs::write(dir.path().join("node_modules").join("junk.md"), "noise\n").unwrap();
        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert!(!records.iter().any(|r| r.path().contains("node_modules")));
    }

    #[test]
    fn symlink_cycle_does_not_infinite_loop() {
        let dir = make_collection();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        symlink(".", dir.path().join("sub").join("loop")).unwrap();
        fs::write(dir.path().join("sub").join("real.md"), "---\ntitle: Real\n---\nbody\n").unwrap();

        let coll = Collection::open(dir.path()).unwrap();
        let records: Vec<_> = coll.records().collect::<Result<_, _>>().unwrap();
        assert!(records.iter().any(|r| r.path() == "sub/real.md"));
    }

    #[test]
    fn records_without_frontmatter_pass_through() {
        let dir = make_collection();
        let coll = Collection::open(dir.path()).unwrap();
        let r = coll.record("readme.txt").unwrap();
        assert!(!r.has_frontmatter());
        assert_eq!(r.body(), "No frontmatter, still a record.\n");
    }
}
