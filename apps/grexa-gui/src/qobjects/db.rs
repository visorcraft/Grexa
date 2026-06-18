// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `DbController` — bridges `grexa-db` to QML.
//!
//! Exposes database open, collection listing, record browsing, schema
//! validation, and view materialization to the QML UI. List data crosses
//! the boundary as newline-separated `QString`s (Phase 2 simplicity).

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

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
        type DbController = super::DbControllerRust;

        #[qinvokable]
        fn open_db(self: Pin<&mut DbController>, path: &QString) -> bool;

        #[qinvokable]
        fn collection_names(self: Pin<&mut DbController>) -> QString;

        #[qinvokable]
        fn record_paths(self: Pin<&mut DbController>, collection: &QString) -> QString;

        #[qinvokable]
        fn validate(self: Pin<&mut DbController>, collection: &QString) -> QString;

        #[qinvokable]
        fn materialize_view(
            self: Pin<&mut DbController>,
            collection: &QString,
            view_name: &QString,
            group_by: &QString,
        ) -> bool;
    }

    impl cxx_qt::Threading for DbController {}
}

#[derive(Default)]
pub struct DbControllerRust {
    db_path: QString,
    is_open: bool,
    status_message: QString,
    db: Option<grexa_db::Db>,
}

impl ffi::DbController {
    fn open_db(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path_str = {
            let s = path.to_string();
            if s.starts_with("~") {
                if let Some(home) = std::env::var_os("HOME") {
                    format!("{}{}", home.to_string_lossy(), &s[1..])
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
                    .set_status_message(QString::from("Database opened successfully"));
                self.rust_mut().db = Some(db);
                true
            }
            Err(e) => {
                tracing::warn!("DbController: failed to open `{path_str}`: {e}");
                self.as_mut()
                    .set_status_message(QString::from(&format!("Error: {e}")));
                false
            }
        }
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

    fn record_paths(self: Pin<&mut Self>, collection: &QString) -> QString {
        let rust = self.rust();
        if let Some(db) = &rust.db {
            match db.collection(&collection.to_string()) {
                Ok(coll) => {
                    let mut paths: Vec<String> = Vec::new();
                    for result in coll.records().take(500) {
                        match result {
                            Ok(r) => paths.push(r.path().to_string()),
                            Err(e) => tracing::warn!("DbController: record read failed: {e}"),
                        }
                    }
                    return QString::from(&paths.join("\n"));
                }
                Err(e) => tracing::warn!("DbController: collection open failed: {e}"),
            }
        }
        QString::from("")
    }

    fn validate(self: Pin<&mut Self>, collection: &QString) -> QString {
        let rust = self.rust();
        let Some(db) = &rust.db else {
            return QString::from("No database open");
        };
        let coll_name = collection.to_string();
        let Ok(coll) = db.collection(&coll_name) else {
            return QString::from("Cannot open collection");
        };
        let errors = coll.validate_all();
        if errors.is_empty() {
            return QString::from("All records valid");
        }
        let report: Vec<String> = errors
            .iter()
            .map(|e| format!("{}: {}: {}", e.record_path, e.field, e.message))
            .collect();
        QString::from(&report.join("\n"))
    }

    fn materialize_view(
        mut self: Pin<&mut Self>,
        collection: &QString,
        view_name: &QString,
        group_by: &QString,
    ) -> bool {
        let rust = self.rust();
        let Some(db) = &rust.db else {
            return false;
        };
        let coll_name = collection.to_string();
        let Ok(coll) = db.collection(&coll_name) else {
            return false;
        };
        let view = view_name.to_string();
        let group = group_by.to_string();
        let group_opt = if group.is_empty() {
            None
        } else {
            Some(group.as_str())
        };
        let query = coll.query();
        let result = db.materialize_view(&view, query, group_opt);
        match result {
            Ok(()) => {
                self.as_mut()
                    .set_status_message(QString::from("View materialized"));
                true
            }
            Err(e) => {
                self.as_mut()
                    .set_status_message(QString::from(&format!("Materialize error: {e}")));
                false
            }
        }
    }
}
