// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `DbController` — bridges `grexa-db` to QML with threaded I/O.
//!
//! All filesystem-heavy operations (`record_paths`, `validate`) run on
//! worker threads via [`cxx_qt::Threading`]; results are posted back
//! through qproperty updates + signals so the GUI never freezes.

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
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
        #[qproperty(QString, record_paths_result)]
        #[qproperty(QString, validate_result)]
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
        ) -> bool;

        #[qsignal]
        fn record_paths_ready(self: Pin<&mut DbController>);

        #[qsignal]
        fn validate_ready(self: Pin<&mut DbController>);
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
    busy: bool,
    db: Option<grexa_db::Db>,
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
                self.rust_mut().db = Some(db);
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
                                .map(|e| format!("{}: {}: {}", e.record_path, e.field, e.message))
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
        match db.materialize_view(&view, query, group_opt) {
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
