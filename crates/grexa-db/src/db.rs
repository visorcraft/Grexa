// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! A database — a root directory containing zero or more collections.

use crate::collection::Collection;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("`{0}` is not a directory")]
    NotADirectory(String),
    #[error("collection error: {0}")]
    Collection(#[from] crate::collection::CollectionError),
}

/// A flat-file database rooted at a directory.
///
/// Each subdirectory containing a `schema.md` is a
/// [`Collection`]. The root itself is not a collection.
pub struct Db {
    root: PathBuf,
}

impl Db {
    /// Open a database rooted at `root`. The path must be an existing
    /// directory. No collections are scanned at open time — discovery is
    /// deferred to [`collections()`](Self::collections).
    pub fn open(root: impl AsRef<Path>) -> Result<Self, DbError> {
        let root = root.as_ref();
        if !root.is_dir() {
            return Err(DbError::NotADirectory(root.display().to_string()));
        }
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    /// The database root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Open a named collection (a subdirectory with a `schema.md`).
    pub fn collection(&self, name: &str) -> Result<Collection, DbError> {
        let path = self.root.join(name);
        Ok(Collection::open(path)?)
    }

    /// Discover all collection names (subdirectories containing
    /// `schema.md`). Returns a sorted list. Hidden directories (starting
    /// with `.`) and the reserved `views` directory are excluded.
    pub fn collections(&self) -> Vec<String> {
        let mut names = Vec::new();
        let Ok(entries) = fs::read_dir(&self.root) else {
            return names;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if name.starts_with('.') || name == "views" {
                continue;
            }
            let path = entry.path();
            if path.is_dir() && path.join("schema.md").exists() {
                names.push(name.to_string());
            }
        }
        names.sort();
        names
    }

    /// Materialize a query result as a directory of symlinks under
    /// `views/<view_name>`. See [`view`](crate::view) for semantics.
    pub fn materialize_view(
        &self,
        view_name: &str,
        query: crate::query::Query<'_>,
        group_by: Option<&str>,
    ) -> Result<(), crate::view::MaterializeError> {
        crate::view::materialize(&self.root, view_name, query, group_by)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const NOTES_SCHEMA: &str =
        "---\ncollection: notes\nfields:\n  - { name: title, type: string }\n---\n";
    const BOOKMARKS_SCHEMA: &str =
        "---\ncollection: bookmarks\nfields:\n  - { name: url, type: string }\n---\n";

    fn make_db() -> TempDir {
        let dir = TempDir::new().unwrap();
        let notes = dir.path().join("notes");
        let bookmarks = dir.path().join("bookmarks");
        fs::create_dir(&notes).unwrap();
        fs::create_dir(&bookmarks).unwrap();
        fs::write(notes.join("schema.md"), NOTES_SCHEMA).unwrap();
        fs::write(bookmarks.join("schema.md"), BOOKMARKS_SCHEMA).unwrap();
        fs::write(notes.join("a.md"), "---\ntitle: Alpha\n---\nbody\n").unwrap();
        fs::write(bookmarks.join("rust.md"), "---\nurl: https://rust-lang.org\n---\nbody\n")
            .unwrap();
        dir
    }

    #[test]
    fn open_existing_directory() {
        let dir = TempDir::new().unwrap();
        let db = Db::open(dir.path()).unwrap();
        assert_eq!(db.root(), dir.path());
    }

    #[test]
    fn open_nonexistent_path_errors() {
        assert!(Db::open("/nonexistent/path/xyz").is_err());
    }

    #[test]
    fn open_file_not_directory_errors() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("notadir.txt");
        fs::write(&file, "content").unwrap();
        assert!(Db::open(&file).is_err());
    }

    #[test]
    fn discover_collections() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections();
        assert_eq!(colls, vec!["bookmarks", "notes"]);
    }

    #[test]
    fn open_collection_by_name() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();
        assert_eq!(notes.name(), "notes");
        assert_eq!(notes.schema().fields.len(), 1);
    }

    #[test]
    fn open_nonexistent_collection_errors() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        assert!(db.collection("nonexistent").is_err());
    }

    #[test]
    fn plain_subdirs_without_schema_are_not_collections() {
        let dir = make_db();
        fs::create_dir_all(dir.path().join("plain")).unwrap();
        fs::write(dir.path().join("plain").join("file.md"), "no schema here\n").unwrap();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections();
        assert!(!colls.contains(&"plain".to_string()));
    }

    #[test]
    fn hidden_directories_excluded() {
        let dir = make_db();
        fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        fs::write(
            dir.path().join(".hidden").join("schema.md"),
            "---\ncollection: hidden\nfields: []\n---\n",
        )
        .unwrap();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections();
        assert!(!colls.contains(&".hidden".to_string()));
    }

    #[test]
    fn views_directory_excluded_from_collections() {
        let dir = make_db();
        fs::create_dir_all(dir.path().join("views").join("notes-by-tag")).unwrap();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections();
        assert!(!colls.contains(&"views".to_string()));
    }

    #[test]
    fn full_stack_db_to_record() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();
        let r = notes.record("a.md").unwrap();
        assert_eq!(r.field("title").unwrap().as_str(), Some("Alpha"));
        assert_eq!(r.body(), "body\n");
    }

    #[test]
    fn iterate_records_through_db() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        let notes = db.collection("notes").unwrap();
        let records: Vec<_> = notes.records().collect::<Result<_, _>>().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].field("title").unwrap().as_str(), Some("Alpha"));
    }
}
