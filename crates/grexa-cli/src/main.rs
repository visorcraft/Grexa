use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use grexa_core::{OutputFormat, SearchOptions, SearchResult, SizeLimitType, SizeUnit, search};

#[derive(Debug, Parser)]
#[command(name = "grexa-cli")]
#[command(about = "Grexa - fast Linux file content search")]
struct Cli {
    /// Directory path to search.
    path: PathBuf,

    /// Search term or regex pattern.
    term: String,

    /// Treat search term as a regex pattern.
    #[arg(short = 'E', long = "regex")]
    regex: bool,

    /// Case-sensitive search.
    #[arg(short = 'i', long = "case-sensitive")]
    case_sensitive: bool,

    /// Respect .gitignore and related ignore files.
    #[arg(short = 'g', long = "gitignore")]
    gitignore: bool,

    /// Include hidden files and directories.
    #[arg(short = 'H', long = "include-hidden")]
    include_hidden: bool,

    /// Include searchable binary/document files.
    #[arg(short = 'b', long = "include-binary")]
    include_binary: bool,

    /// Include system/dependency directories such as .git and node_modules.
    #[arg(short = 's', long = "include-system")]
    include_system: bool,

    /// Do not recurse into subdirectories.
    #[arg(short = 'd', long = "no-subfolders")]
    no_subfolders: bool,

    /// Follow symbolic links.
    #[arg(short = 'L', long = "include-symlinks")]
    include_symlinks: bool,

    /// File name pattern, e.g. '*.rs;*.toml|-target*'.
    #[arg(short = 'm', long = "match-files")]
    match_files: Option<String>,

    /// Directories to exclude, comma/semicolon names or regex.
    #[arg(short = 'x', long = "exclude-dirs")]
    exclude_dirs: Option<String>,

    /// File size limit value.
    #[arg(long = "size-limit")]
    size_limit: Option<u64>,

    /// Size unit.
    #[arg(long = "size-unit", default_value = "kb")]
    size_unit: CliSizeUnit,

    /// Size comparison type.
    #[arg(long = "size-type", default_value = "less")]
    size_type: CliSizeLimitType,

    /// Output format.
    #[arg(short = 'f', long = "format", default_value = "text")]
    format: CliOutputFormat,

    /// Only print total match count.
    #[arg(short = 'c', long = "count")]
    count: bool,

    /// Only print file names with matches.
    #[arg(short = 'l', long = "files-only")]
    files_only: bool,

    /// Suppress output; exit code indicates match.
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliSizeUnit {
    KB,
    MB,
    GB,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliSizeLimitType {
    Less,
    Equal,
    Greater,
    None,
}

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(2);
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    let mut options = SearchOptions::new(&cli.path, &cli.term);
    options.regex = cli.regex;
    options.case_sensitive = cli.case_sensitive;
    options.respect_gitignore = cli.gitignore;
    options.include_hidden = cli.include_hidden;
    options.include_binary = cli.include_binary;
    options.include_system = cli.include_system;
    options.include_subfolders = !cli.no_subfolders;
    options.include_symlinks = cli.include_symlinks;
    options.match_file_names = cli.match_files.unwrap_or_default();
    options.exclude_dirs = cli.exclude_dirs.unwrap_or_default();
    options.size_limit_kb = cli
        .size_limit
        .map(|value| convert_to_kb(value, cli.size_unit));
    options.size_unit = cli.size_unit.into();
    options.size_limit_type = cli.size_type.into();

    let summary = search(&options)?;

    if cli.quiet {
        return Ok(if summary.results.is_empty() { 1 } else { 0 });
    }

    if cli.count {
        println!("{}", summary.matches);
        return Ok(if summary.results.is_empty() { 1 } else { 0 });
    }

    if cli.files_only {
        let mut files: Vec<_> = summary
            .results
            .iter()
            .map(|result| result.full_path.clone())
            .collect();
        files.sort();
        files.dedup();
        for file in files {
            println!("{}", file.display());
        }
        return Ok(if summary.results.is_empty() { 1 } else { 0 });
    }

    match cli.format.into() {
        OutputFormat::Text => print_text(&summary.results),
        OutputFormat::Json => print_json(&summary.results)?,
        OutputFormat::Csv => print_csv(&summary.results),
    }

    Ok(if summary.results.is_empty() { 1 } else { 0 })
}

fn print_text(results: &[SearchResult]) {
    for result in results {
        println!(
            "{}:{}:{}:{}",
            result.full_path.display(),
            result.line_number,
            result.column_number,
            result.line_content
        );
    }
}

fn print_json(results: &[SearchResult]) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(results)?);
    Ok(())
}

fn print_csv(results: &[SearchResult]) {
    println!("File,Line,Column,Content,FullPath,MatchCount");
    for result in results {
        println!(
            "{},{},{},{},{},{}",
            csv_escape(&result.file_name),
            result.line_number,
            result.column_number,
            csv_escape(&result.line_content),
            csv_escape(&result.full_path.display().to_string()),
            result.match_count
        );
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn convert_to_kb(value: u64, unit: CliSizeUnit) -> u64 {
    match unit {
        CliSizeUnit::KB => value,
        CliSizeUnit::MB => value * 1024,
        CliSizeUnit::GB => value * 1024 * 1024,
    }
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Text => Self::Text,
            CliOutputFormat::Json => Self::Json,
            CliOutputFormat::Csv => Self::Csv,
        }
    }
}

impl From<CliSizeUnit> for SizeUnit {
    fn from(value: CliSizeUnit) -> Self {
        match value {
            CliSizeUnit::KB => Self::KB,
            CliSizeUnit::MB => Self::MB,
            CliSizeUnit::GB => Self::GB,
        }
    }
}

impl From<CliSizeLimitType> for SizeLimitType {
    fn from(value: CliSizeLimitType) -> Self {
        match value {
            CliSizeLimitType::Less => Self::LessThan,
            CliSizeLimitType::Equal => Self::EqualTo,
            CliSizeLimitType::Greater => Self::GreaterThan,
            CliSizeLimitType::None => Self::NoLimit,
        }
    }
}
