// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `DbController` — bridges `grexa-db` to QML with threaded I/O.
//!
//! All filesystem-heavy operations (`record_paths`, `validate`) run on
//! worker threads via [`cxx_qt::Threading`]; results are posted back
//! through qproperty updates + signals so the GUI never freezes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use grexa_db::Index;
use notify::Watcher as _;

/// Per-collection in-memory indexes, shared between the query worker threads
/// (read) and the filesystem watcher (reconcile).
type SharedIndexes = Arc<RwLock<HashMap<String, Index>>>;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, db_path)]
        #[qproperty(bool, is_open)]
        #[qproperty(QString, status_message)]
        #[qproperty(QString, record_paths_result)]
        #[qproperty(QString, validate_result)]
        #[qproperty(QString, query_result)]
        #[qproperty(bool, busy)]
        type DbController = super::DbControllerRust;

        #[qinvokable]
        fn open_db(self: Pin<&mut DbController>, path: &QString) -> bool;

        #[qinvokable]
        fn collection_names(self: Pin<&mut DbController>) -> QString;

        #[qinvokable]
        fn record_paths(self: Pin<&mut DbController>, collection: &QString);

        #[qinvokable]
        fn validate(self: Pin<&mut DbController>, collection: &QString);

        #[qinvokable]
        fn materialize_view(
            self: Pin<&mut DbController>,
            collection: &QString,
            view_name: &QString,
            group_by: &QString,
        );

        #[qinvokable]
        fn schema_json(self: Pin<&mut DbController>, collection: &QString) -> QString;

        #[qinvokable]
        fn query_records(self: Pin<&mut DbController>, collection: &QString, filter_json: &QString);

        #[qinvokable]
        fn list_views(self: Pin<&mut DbController>) -> QString;

        #[qinvokable]
        fn record_frontmatter(
            self: Pin<&mut DbController>,
            collection: &QString,
            record_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn delete_view(self: Pin<&mut DbController>, view_name: &QString) -> bool;

        #[qsignal]
        fn record_paths_ready(self: Pin<&mut DbController>);

        #[qsignal]
        fn validate_ready(self: Pin<&mut DbController>);

        #[qsignal]
        fn query_ready(self: Pin<&mut DbController>);
    }

    impl cxx_qt::Threading for DbController {}
}

#[derive(Default)]
pub struct DbControllerRust {
    db_path: QString,
    is_open: bool,
    status_message: QString,
    record_paths_result: QString,
    validate_result: QString,
    query_result: QString,
    busy: bool,
    db: Option<grexa_db::Db>,
    /// Held index per collection; queries read it, the watcher reconciles it.
    indexes: SharedIndexes,
    /// Kept alive to keep watching; replaced on each `open_db`.
    watcher: Option<notify::RecommendedWatcher>,
}

impl ffi::DbController {
    fn open_db(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_str = {
            let s = path.to_string();
            if let Some(rest) = s.strip_prefix('~') {
                if let Some(home) = std::env::var_os("HOME") {
                    format!("{}{}", home.to_string_lossy(), rest)
                } else {
                    s
                }
            } else {
                s
            }
        };
        match grexa_db::Db::open(&path_str) {
            Ok(db) => {
                self.as_mut().set_db_path(QString::from(&path_str));
                self.as_mut().set_is_open(true);
                self.as_mut()
                    .set_status_message(QString::from("Database opened"));
                self.as_mut().rust_mut().db = Some(db);
                self.as_mut().start_indexing(&path_str);
                true
            }
            Err(e) => {
                tracing::warn!("DbController: open failed `{path_str}`: {e}");
                self.as_mut()
                    .set_status_message(QString::from(&format!("Error: {e}")));
                false
            }
        }
    }

    /// Start the secondary-index machinery for a freshly opened database: build
    /// every collection's index in the background (so the first selective query
    /// is fast) and watch the tree to keep the indexes current. Both are
    /// best-effort; queries fall back to a scan whenever an index is absent.
    fn start_indexing(mut self: Pin<&mut Self>, root: &str) {
        let root = PathBuf::from(root);
        let indexes = self.rust().indexes.clone();

        // Background build — never blocks the UI; queries scan until it lands.
        {
            let indexes = indexes.clone();
            let root = root.clone();
            std::thread::spawn(move || build_all_indexes(&root, &indexes));
        }

        // Watch for out-of-band edits and reconcile the touched collection's
        // index. Verify-on-read keeps queries correct even before this fires.
        let watch_indexes = indexes;
        let watch_root = root.clone();
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                reconcile_event(&watch_root, &watch_indexes, &event.paths);
            }
        })
        .ok()
        .and_then(|mut w| {
            w.watch(&root, notify::RecursiveMode::Recursive)
                .ok()
                .map(|()| w)
        });
        if watcher.is_none() {
            tracing::warn!("DbController: no filesystem watcher; index won't auto-refresh");
        }
        self.rust_mut().watcher = watcher;
    }

    fn collection_names(self: Pin<&mut Self>) -> QString {
        let rust = self.rust();
        if let Some(db) = &rust.db {
            match db.collections() {
                Ok(names) => return QString::from(&names.join("\n")),
                Err(e) => tracing::warn!("DbController: collections() failed: {e}"),
            }
        }
        QString::from("")
    }

    fn record_paths(mut self: Pin<&mut Self>, collection: &QString) {
        let (db_path, coll_name) = {
            let qself = self.as_ref();
            let rust = qself.rust();
            let Some(db) = &rust.db else {
                return;
            };
            (db.root().to_path_buf(), collection.to_string())
        };

        self.as_mut().set_busy(true);
        let thread = self.qt_thread();

        std::thread::spawn(move || {
            let result = match grexa_db::Db::open(&db_path) {
                Ok(db) => match db.collection(&coll_name) {
                    Ok(coll) => {
                        let mut paths = Vec::new();
                        for r in coll.records().take(500) {
                            match r {
                                Ok(r) => paths.push(r.path().to_string()),
                                Err(e) => tracing::warn!("record read: {e}"),
                            }
                        }
                        paths.join("\n")
                    }
                    Err(e) => {
                        tracing::warn!("collection open: {e}");
                        String::new()
                    }
                },
                Err(e) => {
                    tracing::warn!("db open: {e}");
                    String::new()
                }
            };

            let _ = thread.queue(move |mut pin| {
                pin.as_mut().set_record_paths_result(QString::from(&result));
                pin.as_mut().set_busy(false);
                pin.as_mut().record_paths_ready();
            });
        });
    }

    fn validate(mut self: Pin<&mut Self>, collection: &QString) {
        let (db_path, coll_name) = {
            let qself = self.as_ref();
            let rust = qself.rust();
            let Some(db) = &rust.db else {
                return;
            };
            (db.root().to_path_buf(), collection.to_string())
        };

        self.as_mut().set_busy(true);
        let thread = self.qt_thread();

        std::thread::spawn(move || {
            let result = match grexa_db::Db::open(&db_path) {
                Ok(db) => match db.collection(&coll_name) {
                    Ok(coll) => {
                        let errors = coll.validate_all();
                        if errors.is_empty() {
                            "All records valid".to_string()
                        } else {
                            errors
                                .iter()
                                .map(|e| {
                                    let tag = match e.severity {
                                        grexa_db::Severity::Error => "error",
                                        grexa_db::Severity::Warning => "warning",
                                    };
                                    format!("[{tag}] {}: {}: {}", e.record_path, e.field, e.message)
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    }
                    Err(e) => format!("Cannot open collection: {e}"),
                },
                Err(e) => format!("Cannot open db: {e}"),
            };

            let _ = thread.queue(move |mut pin| {
                pin.as_mut().set_validate_result(QString::from(&result));
                pin.as_mut().set_busy(false);
                pin.as_mut().validate_ready();
            });
        });
    }

    fn materialize_view(
        mut self: Pin<&mut Self>,
        collection: &QString,
        view_name: &QString,
        group_by: &QString,
    ) {
        let (db_path, coll_name, view, group) = {
            let qself = self.as_ref();
            let rust = qself.rust();
            let Some(db) = &rust.db else {
                return;
            };
            (
                db.root().to_path_buf(),
                collection.to_string(),
                view_name.to_string(),
                group_by.to_string(),
            )
        };

        self.as_mut().set_busy(true);
        let thread = self.qt_thread();

        std::thread::spawn(move || {
            let result = (|| {
                let db = grexa_db::Db::open(&db_path).map_err(|e| e.to_string())?;
                let coll = db.collection(&coll_name).map_err(|e| e.to_string())?;
                let group_opt = if group.is_empty() {
                    None
                } else {
                    Some(group.as_str())
                };
                db.materialize_view(&view, coll.query(), group_opt)
                    .map_err(|e| e.to_string())
            })();

            let msg = match &result {
                Ok(()) => "View materialized".to_string(),
                Err(e) => format!("Materialize error: {e}"),
            };

            let _ = thread.queue(move |mut pin| {
                pin.as_mut().set_status_message(QString::from(&msg));
                pin.as_mut().set_busy(false);
            });
        });
    }

    fn schema_json(self: Pin<&mut Self>, collection: &QString) -> QString {
        let rust = self.rust();
        let Some(db) = &rust.db else {
            return QString::from("[]");
        };
        match db.collection(&collection.to_string()) {
            Ok(coll) => {
                let fields: Vec<serde_json::Value> = coll
                    .schema()
                    .fields
                    .iter()
                    .map(|f| {
                        serde_json::json!({
                            "name": f.name,
                            "type": f.field_type.to_string(),
                            "required": f.required,
                            // Per the Phase 2 spec note: {name, type, required, range}.
                            // `[min, max]` for numeric fields with a range, else null.
                            "range": match f.range {
                                Some((min, max)) => serde_json::json!([min, max]),
                                None => serde_json::Value::Null,
                            },
                        })
                    })
                    .collect();
                QString::from(&serde_json::to_string(&fields).unwrap_or_else(|_| "[]".into()))
            }
            Err(_) => QString::from("[]"),
        }
    }

    fn query_records(mut self: Pin<&mut Self>, collection: &QString, filter_json: &QString) {
        let (db_path, coll_name, filters_raw) = {
            let qself = self.as_ref();
            let rust = qself.rust();
            let Some(db) = &rust.db else {
                return;
            };
            (db.root().to_path_buf(), collection.to_string(), filter_json.to_string())
        };

        self.as_mut().set_busy(true);
        let thread = self.qt_thread();
        let indexes = self.rust().indexes.clone();

        std::thread::spawn(move || {
            let result = (|| {
                let db = grexa_db::Db::open(&db_path).map_err(|e| e.to_string())?;
                let coll = db.collection(&coll_name).map_err(|e| e.to_string())?;
                let parsed: Vec<(String, String, String)> =
                    serde_json::from_str(&filters_raw).unwrap_or_default();
                let mut query = coll.query();
                for (field, op, value) in &parsed {
                    query = apply_query_filter(query, field, op, value);
                }
                // Use the held index only when a filter it can serve is present
                // (eq/contains) and the collection has been indexed — that's the
                // selective case it wins on. Otherwise stream the first 500
                // (cheap early-exit; no whole-result materialization).
                let serviceable = parsed.iter().any(|(_, op, _)| op == "eq" || op == "contains");
                let guard = indexes.read().map_err(|_| "index lock poisoned".to_string())?;
                let paths: Vec<String> = match guard.get(&coll_name) {
                    Some(idx) if serviceable => query
                        .using_index(idx)
                        .collect_par()
                        .map_err(|e| e.to_string())?
                        .into_iter()
                        .take(500)
                        .map(|r| r.path().to_string())
                        .collect(),
                    _ => {
                        drop(guard);
                        let mut v = Vec::new();
                        for r in query.take(500) {
                            match r {
                                Ok(r) => v.push(r.path().to_string()),
                                Err(e) => tracing::warn!("query record: {e}"),
                            }
                        }
                        v
                    }
                };
                Ok::<String, String>(paths.join("\n"))
            })();

            let output = result.unwrap_or_default();
            let _ = thread.queue(move |mut pin| {
                pin.as_mut().set_query_result(QString::from(&output));
                pin.as_mut().set_busy(false);
                pin.as_mut().query_ready();
            });
        });
    }

    fn list_views(self: Pin<&mut Self>) -> QString {
        let rust = self.rust();
        let Some(db) = &rust.db else {
            return QString::from("");
        };
        let views_dir = db.root().join("views");
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&views_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_symlink()).unwrap_or(false)
                    && let Some(name) = entry.file_name().to_str()
                    && !name.starts_with('.')
                {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        QString::from(&names.join("\n"))
    }

    fn record_frontmatter(
        self: Pin<&mut Self>,
        collection: &QString,
        record_path: &QString,
    ) -> QString {
        let rust = self.rust();
        let Some(db) = &rust.db else {
            return QString::from("{}");
        };
        match db.collection(&collection.to_string()) {
            Ok(coll) => match coll.record(&record_path.to_string()) {
                Ok(record) => QString::from(&record.frontmatter_json()),
                Err(_) => QString::from("{}"),
            },
            Err(_) => QString::from("{}"),
        }
    }

    fn delete_view(mut self: Pin<&mut Self>, view_name: &QString) -> bool {
        let name = view_name.to_string();
        // Without this guard `join` escapes the views dir: an absolute arg
        // (`/etc/...`) replaces the base entirely and `..`/separators walk
        // out — an arbitrary-file-deletion sink. Mirror grexa-db's own
        // view-name validation (reject separators, dotfiles, traversal).
        if !is_safe_view_name(&name) {
            self.as_mut()
                .set_status_message(QString::from("Delete error: invalid view name"));
            return false;
        }

        let view_path = {
            let rust = self.rust();
            let Some(db) = &rust.db else {
                return false;
            };
            db.root().join("views").join(&name)
        };

        // Defense in depth: only ever remove a genuine view symlink, never
        // a real file or directory that happens to live under views/.
        let is_symlink = view_path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if !is_symlink {
            self.as_mut()
                .set_status_message(QString::from("Delete error: not a view"));
            return false;
        }

        match std::fs::remove_file(&view_path) {
            Ok(()) => {
                self.as_mut()
                    .set_status_message(QString::from("View deleted"));
                true
            }
            Err(e) => {
                self.as_mut()
                    .set_status_message(QString::from(&format!("Delete error: {e}")));
                false
            }
        }
    }
}

/// A view name is safe iff it names a single entry directly under `views/`:
/// no path separators, no `.`-prefixed names (dotfiles / `.` / `..`), no
/// empties. Absolute paths start with `/`, so they're rejected here too.
fn is_safe_view_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains('/') && !name.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::{collection_and_rel, is_safe_view_name};
    use std::path::Path;

    #[test]
    fn collection_and_rel_maps_records_only() {
        let root = Path::new("/db");
        let cr = |p| collection_and_rel(root, Path::new(p));
        assert_eq!(cr("/db/notes/a.md"), Some(("notes".into(), "a.md".into())));
        assert_eq!(cr("/db/notes/2024/03/b.md"), Some(("notes".into(), "2024/03/b.md".into())));
        // schema, the index dir, views, and paths outside a collection are skipped.
        assert_eq!(cr("/db/notes/schema.md"), None);
        assert_eq!(cr("/db/.grexa-index/index.json"), None);
        assert_eq!(cr("/db/views/by-tag/x.md"), None);
        assert_eq!(cr("/db/loose.md"), None);
        assert_eq!(cr("/elsewhere/notes/a.md"), None);
    }

    #[test]
    fn view_name_safety() {
        assert!(is_safe_view_name("notes-by-tag"));
        assert!(is_safe_view_name("high_rated"));

        // Traversal / absolute / dotfile / separator escapes all rejected.
        assert!(!is_safe_view_name(""));
        assert!(!is_safe_view_name("/etc/passwd"));
        assert!(!is_safe_view_name("../../etc/passwd"));
        assert!(!is_safe_view_name(".."));
        assert!(!is_safe_view_name("."));
        assert!(!is_safe_view_name(".generations"));
        assert!(!is_safe_view_name("a/b"));
        assert!(!is_safe_view_name("a\\b"));
    }
}

/// Build an index for every collection in the database (best-effort).
fn build_all_indexes(root: &Path, indexes: &SharedIndexes) {
    let Ok(db) = grexa_db::Db::open(root) else {
        return;
    };
    let Ok(names) = db.collections() else {
        return;
    };
    for name in names {
        let Ok(coll) = db.collection(&name) else {
            continue;
        };
        match Index::build(&coll) {
            Ok(idx) => {
                if let Ok(mut map) = indexes.write() {
                    map.insert(name, idx);
                }
            }
            Err(e) => tracing::warn!("DbController: index build `{name}`: {e}"),
        }
    }
}

/// Reconcile the affected collections' indexes for a batch of changed paths.
fn reconcile_event(root: &Path, indexes: &SharedIndexes, paths: &[PathBuf]) {
    let mut by_coll: HashMap<String, Vec<String>> = HashMap::new();
    for p in paths {
        if let Some((coll, rel)) = collection_and_rel(root, p) {
            by_coll.entry(coll).or_default().push(rel);
        }
    }
    if by_coll.is_empty() {
        return;
    }
    let Ok(db) = grexa_db::Db::open(root) else {
        return;
    };
    for (coll_name, rels) in by_coll {
        let Ok(coll) = db.collection(&coll_name) else {
            continue;
        };
        if let Ok(mut map) = indexes.write()
            && let Some(idx) = map.get_mut(&coll_name)
            && let Err(e) = idx.reconcile(&coll, &rels)
        {
            tracing::warn!("DbController: index reconcile `{coll_name}`: {e}");
        }
    }
}

/// Map a changed absolute path to `(collection, record-relative-path)`, or
/// `None` for the index dir, views, schema files, or anything outside a
/// collection.
fn collection_and_rel(root: &Path, changed: &Path) -> Option<(String, String)> {
    let rel = changed.strip_prefix(root).ok()?;
    let mut comps = rel.components();
    let coll = comps.next()?.as_os_str().to_str()?.to_string();
    if coll.starts_with('.') || coll == "views" {
        return None;
    }
    let rest = comps.as_path().to_string_lossy().replace('\\', "/");
    if rest.is_empty() || rest == "schema.md" {
        return None;
    }
    Some((coll, rest))
}

fn apply_query_filter<'a>(
    query: grexa_db::Query<'a>,
    field: &str,
    op: &str,
    value: &str,
) -> grexa_db::Query<'a> {
    let builder = query.filter(field);
    if let Ok(i) = value.parse::<i64>() {
        return match op {
            "ne" => builder.ne(i),
            "lt" => builder.lt(i),
            "le" => builder.le(i),
            "gt" => builder.gt(i),
            "ge" => builder.ge(i),
            "contains" => builder.contains(i),
            _ => builder.eq(i),
        };
    }
    if let Ok(f) = value.parse::<f64>() {
        return match op {
            "ne" => builder.ne(f),
            "lt" => builder.lt(f),
            "le" => builder.le(f),
            "gt" => builder.gt(f),
            "ge" => builder.ge(f),
            "contains" => builder.contains(f),
            _ => builder.eq(f),
        };
    }
    match op {
        "ne" => builder.ne(value),
        "lt" => builder.lt(value),
        "le" => builder.le(value),
        "gt" => builder.gt(value),
        "ge" => builder.ge(value),
        "contains" => builder.contains(value),
        _ => builder.eq(value),
    }
}
