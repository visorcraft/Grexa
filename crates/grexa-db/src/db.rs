// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! A database — a root directory containing zero or more collections.

use crate::collection::Collection;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("`{0}` is not a directory")]
    NotADirectory(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("invalid collection name `{0}`: path separators and `..` are not allowed")]
    InvalidCollectionName(String),
    #[error("collection error: {0}")]
    Collection(#[from] crate::collection::CollectionError),
}

static ROOT_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A flat-file database rooted at a directory.
///
/// Each subdirectory containing a `schema.md` is a
/// [`Collection`]. The root itself is not a collection.
pub struct Db {
    root: PathBuf,
    view_lock: Arc<Mutex<()>>,
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
        let canonical = root
            .canonicalize()
            .map_err(|e| DbError::NotADirectory(e.to_string()))?;
        let view_lock = {
            let mut registry = ROOT_LOCKS.lock().unwrap();
            registry
                .entry(canonical)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        Ok(Self {
            root: root.to_path_buf(),
            view_lock,
        })
    }

    /// The database root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Open a named collection (a subdirectory with a `schema.md`).
    ///
    /// The name is validated: path separators (`/`, `\`) and `..` are
    /// rejected to prevent traversal outside the DB root.
    pub fn collection(&self, name: &str) -> Result<Collection, DbError> {
        if name.is_empty()
            || name.contains('/')
            || name.contains('\\')
            || name.split('/').any(|c| c == ".." || c == ".")
            || name == ".."
            || name == "."
        {
            return Err(DbError::InvalidCollectionName(name.to_string()));
        }
        let path = self.root.join(name);
        Ok(Collection::open(path)?)
    }

    /// Discover all collection names (subdirectories containing
    /// `schema.md`). Returns a sorted list. Hidden directories (starting
    /// with `.`) and the reserved `views` directory are excluded.
    pub fn collections(&self) -> Result<Vec<String>, DbError> {
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| DbError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| DbError::Io(e.to_string()))?;
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
        Ok(names)
    }

    /// Materialize a query result as a directory of symlinks under
    /// `views/<view_name>`. See [`view`](crate::view) for semantics.
    ///
    /// View materialization is serialized per DB root via an internal
    /// mutex, so concurrent calls on the same root are safe.
    pub fn materialize_view(
        &self,
        view_name: &str,
        query: crate::query::Query<'_>,
        group_by: Option<&str>,
    ) -> Result<(), crate::view::MaterializeError> {
        let _guard = self.view_lock.lock().unwrap();
        crate::view::materialize(&self.root, view_name, query, group_by)
    }

    /// Validate all collections. Returns `(collection_name, errors)` for
    /// each collection that has at least one validation error.
    pub fn validate_all(
        &self,
    ) -> Result<Vec<(String, Vec<crate::validation::ValidationError>)>, DbError> {
        let mut all = Vec::new();
        for name in self.collections()? {
            let coll = self.collection(&name)?;
            let errors = coll.validate_all();
            if !errors.is_empty() {
                all.push((name, errors));
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;
    use std::thread;
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
        let colls = db.collections().unwrap();
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
    fn collection_name_traversal_rejected() {
        let dir = make_db();
        let db = Db::open(dir.path()).unwrap();
        assert!(db.collection("../other").is_err());
        assert!(db.collection("a/b").is_err());
        assert!(db.collection("..").is_err());
        assert!(db.collection("").is_err());
    }

    #[test]
    fn plain_subdirs_without_schema_are_not_collections() {
        let dir = make_db();
        fs::create_dir_all(dir.path().join("plain")).unwrap();
        fs::write(dir.path().join("plain").join("file.md"), "no schema here\n").unwrap();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections().unwrap();
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
        let colls = db.collections().unwrap();
        assert!(!colls.contains(&".hidden".to_string()));
    }

    #[test]
    fn views_directory_excluded_from_collections() {
        let dir = make_db();
        fs::create_dir_all(dir.path().join("views").join("notes-by-tag")).unwrap();
        let db = Db::open(dir.path()).unwrap();
        let colls = db.collections().unwrap();
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

    #[test]
    fn concurrent_materialize_does_not_corrupt() {
        let dir = make_db();
        let db = Arc::new(Db::open(dir.path()).unwrap());

        let db1 = db.clone();
        let db2 = db.clone();
        let h1 = thread::spawn(move || {
            let notes = db1.collection("notes").unwrap();
            db1.materialize_view("view-1", notes.query(), None)
        });
        let h2 = thread::spawn(move || {
            let notes = db2.collection("notes").unwrap();
            db2.materialize_view("view-2", notes.query(), None)
        });
        h1.join().unwrap().unwrap();
        h2.join().unwrap().unwrap();

        assert!(db.root().join("views").join("view-1").is_symlink());
        assert!(db.root().join("views").join("view-2").is_symlink());
    }
}
