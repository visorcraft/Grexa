// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Dogfooding: grexa-core uses grexa-db to store its own data.
//!
//! This module wraps grexa-db to provide a `RecentPathsDb` store that
//! replaces the JSON-based `RecentPathStore` for recent search paths.
//! Writes go directly to the filesystem (the "editor is the write path"
//! philosophy); reads go through grexa-db's typed query layer.
//!
//! This proves the engine works by using it for Grexa's own data.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::NamedTempFile;
use thiserror::Error;

static WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

const COLLECTION_DIR: &str = "recent_paths";
const SCHEMA: &str = "---\ncollection: recent_paths\nfields:\n  - { name: path, type: string, required: true }\n  - { name: added_at, type: integer }\n---\n\n# Recent search paths\n";

#[derive(Debug, Error)]
pub enum DbStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("grexa-db error: {0}")]
    Db(#[from] grexa_db::DbError),
    #[error("collection error: {0}")]
    Collection(#[from] grexa_db::CollectionError),
    #[error("record error: {0}")]
    Record(#[from] grexa_db::RecordError),
}

/// Recent-paths store backed by grexa-db. Each path is a markdown record
/// with frontmatter; the engine reads them via typed queries.
pub struct RecentPathsDb {
    db: grexa_db::Db,
    limit: usize,
}

impl RecentPathsDb {
    /// Create a recent-paths store for the given app paths. Infallible —
    /// falls back to a temp directory if the data dir is unwritable.
    pub fn new(paths: &crate::storage::AppPaths) -> Self {
        let db_root = paths.data_dir.join("db");
        let coll_dir = db_root.join(COLLECTION_DIR);
        let _ = fs::create_dir_all(&coll_dir);
        let schema_path = coll_dir.join("schema.md");
        if !schema_path.exists() {
            let _ = fs::write(&schema_path, SCHEMA);
        }
        let db = grexa_db::Db::open(&db_root).unwrap_or_else(|e| {
            tracing::warn!("RecentPathsDb: data dir failed ({e}), using temp");
            let fallback = std::env::temp_dir().join("grexa-db-fallback");
            let _ = fs::create_dir_all(&fallback);
            grexa_db::Db::open(&fallback).expect("grexa-db must open in /tmp")
        });
        let store = Self { db, limit: 20 };
        migrate_from_json(&paths.data_dir, &store);
        store
    }

    /// Open (or create) a recent-paths database under `data_dir/db/`.
    pub fn open(data_dir: &Path) -> Result<Self, DbStoreError> {
        let db_root = data_dir.join("db");
        let coll_dir = db_root.join(COLLECTION_DIR);
        fs::create_dir_all(&coll_dir)?;
        let schema_path = coll_dir.join("schema.md");
        if !schema_path.exists() {
            fs::write(&schema_path, SCHEMA)?;
        }
        let db = grexa_db::Db::open(&db_root)?;
        Ok(Self { db, limit: 20 })
    }

    /// Add a path to the store. If it already exists, the old entry is
    /// removed first (dedup by path value).
    ///
    /// Single-writer assumed: the GUI's storage path is not called from
    /// multiple threads simultaneously.
    pub fn add(&self, path: impl AsRef<Path>) -> Result<(), DbStoreError> {
        let path = path.as_ref();
        self.remove(path)?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let counter = WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let filename = format!("entry-{nanos:019}-{counter}.md");
        let content = format!("---\npath: {}\nadded_at: {nanos}\n---\n", path.display());
        let coll_dir = self.db.root().join(COLLECTION_DIR);
        let record_path = coll_dir.join(&filename);
        let mut temp = NamedTempFile::new_in(&coll_dir)?;
        temp.write_all(content.as_bytes())?;
        temp.persist(&record_path)
            .map_err(|e| DbStoreError::Io(e.error))?;
        self.enforce_limit()?;
        Ok(())
    }

    /// Remove a path from the store (no-op if absent).
    pub fn remove(&self, path: &Path) -> Result<(), DbStoreError> {
        let target = path.display().to_string();
        let coll = self.db.collection(COLLECTION_DIR)?;
        for result in coll.records() {
            let record = result?;
            if record.field("path").and_then(|v| v.as_str()) == Some(&target) {
                let full = self.db.root().join(COLLECTION_DIR).join(record.path());
                let _ = fs::remove_file(&full);
            }
        }
        Ok(())
    }

    /// Load all stored paths, newest first.
    pub fn load(&self) -> Result<Vec<PathBuf>, DbStoreError> {
        let coll = self.db.collection(COLLECTION_DIR)?;
        let mut entries: Vec<(String, PathBuf)> = Vec::new();
        for result in coll.records() {
            let record = result?;
            if let Some(path_str) = record.field("path").and_then(|v| v.as_str()) {
                entries.push((record.path().to_string(), PathBuf::from(path_str)));
            }
        }
        entries.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(entries
            .into_iter()
            .map(|(_, p)| p)
            .take(self.limit)
            .collect())
    }

    fn enforce_limit(&self) -> Result<(), DbStoreError> {
        let coll = self.db.collection(COLLECTION_DIR)?;
        let mut entries: Vec<String> = coll
            .records()
            .filter_map(|r| r.ok().map(|r| r.path().to_string()))
            .collect();
        entries.sort();
        entries.reverse();
        for name in entries.into_iter().skip(self.limit) {
            let full = self.db.root().join(COLLECTION_DIR).join(&name);
            let _ = fs::remove_file(&full);
        }
        Ok(())
    }
}

fn migrate_from_json(data_dir: &Path, store: &RecentPathsDb) {
    let json_path = data_dir.join("recent_paths.json");
    if !json_path.exists() {
        return;
    }
    if !store.load().unwrap_or_default().is_empty() {
        return;
    }
    match fs::read_to_string(&json_path) {
        Ok(content) => {
            if let Ok(paths) = serde_json::from_str::<Vec<PathBuf>>(&content) {
                tracing::info!("Migrating {} paths from recent_paths.json", paths.len());
                for path in paths {
                    let _ = store.add(&path);
                }
                let backup = data_dir.join("recent_paths.json.bak");
                let _ = fs::rename(&json_path, &backup);
            }
        }
        Err(e) => tracing::warn!("Migration: cannot read recent_paths.json: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trip_add_and_load() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        store.add(PathBuf::from("/home/user/project-a")).unwrap();
        store.add(PathBuf::from("/home/user/project-b")).unwrap();

        let paths = store.load().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&PathBuf::from("/home/user/project-a")));
        assert!(paths.contains(&PathBuf::from("/home/user/project-b")));
    }

    #[test]
    fn newest_first() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        store.add(PathBuf::from("/old")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        store.add(PathBuf::from("/new")).unwrap();

        let paths = store.load().unwrap();
        assert_eq!(paths[0], PathBuf::from("/new"));
        assert_eq!(paths[1], PathBuf::from("/old"));
    }

    #[test]
    fn dedup_on_re_add() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        store.add(PathBuf::from("/project")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        store.add(PathBuf::from("/project")).unwrap();

        let paths = store.load().unwrap();
        assert_eq!(paths.len(), 1, "re-adding should dedup");
    }

    #[test]
    fn remove_path() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        store.add(PathBuf::from("/a")).unwrap();
        store.add(PathBuf::from("/b")).unwrap();
        store.remove(&PathBuf::from("/a")).unwrap();

        let paths = store.load().unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from("/b"));
    }

    #[test]
    fn limit_enforced() {
        let dir = TempDir::new().unwrap();
        let mut store = RecentPathsDb::open(dir.path()).unwrap();
        store.limit = 3;
        for i in 0..5 {
            store.add(PathBuf::from(format!("/p{i}"))).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let paths = store.load().unwrap();
        assert_eq!(paths.len(), 3, "limit should be enforced");
        assert!(paths.contains(&PathBuf::from("/p4")));
        assert!(!paths.contains(&PathBuf::from("/p0")));
    }

    #[test]
    fn schema_validation_works() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        store.add(PathBuf::from("/test")).unwrap();
        let coll = store.db.collection(COLLECTION_DIR).unwrap();
        let errors = coll.validate_all();
        assert!(errors.is_empty(), "validation errors: {errors:?}");
    }

    #[test]
    fn open_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let store1 = RecentPathsDb::open(dir.path()).unwrap();
        store1.add(PathBuf::from("/persist")).unwrap();
        drop(store1);

        let store2 = RecentPathsDb::open(dir.path()).unwrap();
        let paths = store2.load().unwrap();
        assert!(paths.contains(&PathBuf::from("/persist")));
    }

    #[test]
    fn empty_store_loads_empty() {
        let dir = TempDir::new().unwrap();
        let store = RecentPathsDb::open(dir.path()).unwrap();
        assert!(store.load().unwrap().is_empty());
    }
}
