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
//! - `materialize <collection> <view-name> [--group-by <field>]` — materialize a view

use clap::{Parser, Subcommand};
use grexa_db::Db;
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
                eprintln!("{total_errors} validation error(s)");
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
