// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! `grexa-db` — a flat-file database engine where records are plain files in
//! a directory tree and relational joins materialize as directories of
//! symlinks. The filesystem is the interface: any tool that reads files
//! (`rg`, `grep`, editors, file managers) is a client without knowing the
//! database exists.
//!
//! ## License divergence from the rest of the Grexa workspace
//!
//! Unlike every other Grexa crate, which is `GPL-3.0-only`, this crate is
//! `Apache-2.0` so that it may be embedded in proprietary applications.
//! Apache-2.0 is one-way compatible with GPL-3.0: `grexa-core` and the Grexa
//! GUI may depend on `grexa-db` freely, but the reverse dependency direction
//! would contaminate this crate's license and is forbidden. Do not pull
//! GPL-only dependencies into this crate — the permissive license must be
//! preserved.

/// Crate version. Mirrors the workspace version until `grexa-db` cuts its
/// own release line.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod collection;
pub mod db;
pub mod frontmatter;
pub mod query;
pub mod record;
pub mod schema;
pub mod validation;
pub mod view;

pub use collection::{Collection, CollectionError};
pub use db::{Db, DbError};
pub use frontmatter::{FrontmatterError, Split, split};
pub use query::{FilterBuilder, IntoValue, OrderBuilder, Query};
pub use record::{Record, RecordError};
pub use schema::{FieldDef, FieldType, Schema, SchemaError};
pub use serde_yaml::Value;
pub use validation::ValidationError;
pub use view::MaterializeError;
