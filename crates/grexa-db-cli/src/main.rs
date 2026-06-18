// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: Apache-2.0

//! `grexa-db-cli` — standalone CLI for grexa-db databases.
//!
//! Usage: `grexa-db-cli <db-path> <command>`
//!
//! Commands:
//! - `collections` — list all collections
//! - `records <collection>` — list record paths
//! - `validate [collection]` — validate records against schema
//! - `query <collection> [--filter field:op:value]... [--order-by field]` — query with filters

use clap::{Parser, Subcommand};
use grexa_db::{Db, IntoValue};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "grexa-db-cli", about = "CLI for grexa-db flat-file databases")]
struct Cli {
    /// Path to the database root directory.
    db_path: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all collections in the database.
    Collections,
    /// List record paths in a collection.
    Records { collection: String },
    /// Query records with filters. Example: --filter rating:ge:4 --filter tags:contains:rust
    Query {
        collection: String,
        #[arg(long, value_name = "FIELD:OP:VALUE")]
        filter: Vec<String>,
        #[arg(long)]
        order_by: Option<String>,
        #[arg(long, default_value = "asc")]
        direction: String,
    },
    /// Validate records against their schema.
    Validate { collection: Option<String> },
    /// Materialize a query result as a directory of symlinks.
    Materialize {
        collection: String,
        view_name: String,
        #[arg(long)]
        group_by: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::open(&cli.db_path)?;

    match &cli.command {
        Command::Collections => {
            for name in db.collections()? {
                println!("{name}");
            }
        }

        Command::Records { collection } => {
            let coll = db.collection(collection)?;
            let mut count = 0;
            for result in coll.records() {
                let record = result?;
                println!("{}", record.path());
                count += 1;
            }
            eprintln!("{count} records");
        }

        Command::Query {
            collection,
            filter,
            order_by,
            direction,
        } => {
            let coll = db.collection(collection)?;
            let mut query = coll.query();
            for f in filter {
                let parts: Vec<&str> = f.splitn(3, ':').collect();
                if parts.len() == 3 {
                    query = apply_filter(query, parts[0], parts[1], parts[2]);
                }
            }
            if let Some(field) = order_by {
                query = if direction == "desc" {
                    query.order_by(field).desc()
                } else {
                    query.order_by(field).asc()
                };
            }
            let mut count = 0;
            for result in query {
                let record = result?;
                println!("{}", record.path());
                count += 1;
            }
            eprintln!("{count} records");
        }

        Command::Validate { collection } => {
            let total_errors = if let Some(name) = collection {
                let coll = db.collection(name)?;
                let errors = coll.validate_all();
                let count = errors.len();
                for e in &errors {
                    println!("{name}/{}: {}: {}", e.record_path, e.field, e.message);
                }
                count
            } else {
                let reports = db.validate_all()?;
                let mut total = 0;
                for (coll_name, errors) in &reports {
                    for e in errors {
                        println!("{coll_name}/{}: {}: {}", e.record_path, e.field, e.message);
                        total += 1;
                    }
                }
                total
            };
            if total_errors == 0 {
                eprintln!("all records valid");
            } else {
                return Err(format!("{total_errors} validation error(s)").into());
            }
        }

        Command::Materialize {
            collection,
            view_name,
            group_by,
        } => {
            let coll = db.collection(collection)?;
            db.materialize_view(view_name, coll.query(), group_by.as_deref())?;
            let view_path = db.root().join("views").join(view_name);
            println!("{}", view_path.display());
        }
    }

    Ok(())
}

fn apply_filter<'a>(
    query: grexa_db::Query<'a>,
    field: &str,
    op: &str,
    value: &str,
) -> grexa_db::Query<'a> {
    let builder = query.filter(field);
    if let Ok(i) = value.parse::<i64>() {
        return apply_op(builder, op, i);
    }
    if let Ok(f) = value.parse::<f64>() {
        return apply_op(builder, op, f);
    }
    if let Ok(b) = value.parse::<bool>() {
        return apply_op(builder, op, b);
    }
    apply_op(builder, op, value)
}

fn apply_op<'a, V: IntoValue>(
    builder: grexa_db::FilterBuilder<'a>,
    op: &str,
    value: V,
) -> grexa_db::Query<'a> {
    match op {
        "eq" => builder.eq(value),
        "ne" => builder.ne(value),
        "lt" => builder.lt(value),
        "le" => builder.le(value),
        "gt" => builder.gt(value),
        "ge" => builder.ge(value),
        "contains" => builder.contains(value),
        _ => builder.eq(value),
    }
}
