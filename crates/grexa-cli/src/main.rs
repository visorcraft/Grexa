use std::io;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use grexa_core::{
    AppPaths, CancelToken, OutputFormat, SearchOptions, SearchResult, SizeLimitType, SizeUnit,
    StringComparisonMode, UnicodeNormalizationMode, search_with,
};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[derive(Debug, Parser)]
#[command(name = "grexa-cli")]
#[command(version)]
#[command(about = "Grexa - fast Linux file content search")]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    /// Top-level command. When omitted, the positional `path`/`term` are used
    /// for a one-shot search.
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    search: Option<SearchArgs>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print shell completion script for the requested shell.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Print the man page in roff format. Pipe through `gzip -c > grexa-cli.1.gz`
    /// or save as `grexa-cli.1` for installation under `/usr/share/man/man1`.
    Manpage,
}

#[derive(Debug, Parser, Clone)]
struct SearchArgs {
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

    /// Include hidden files and directories. Also accepts the `--hidden`
    /// alias from `rg` for ergonomics.
    #[arg(short = 'H', long = "include-hidden", visible_alias = "hidden")]
    include_hidden: bool,

    /// Include searchable binary/document files.
    #[arg(short = 'b', long = "include-binary")]
    include_binary: bool,

    /// Include system/dependency directories such as .git and node_modules.
    /// `--no-ignore` is accepted as a `rg`-style alias.
    #[arg(short = 's', long = "include-system", visible_alias = "no-ignore")]
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

    /// String comparison mode for plain-text search.
    #[arg(long = "comparison", default_value = "ordinal")]
    comparison: CliComparisonMode,

    /// Unicode normalization to apply before comparison.
    #[arg(long = "normalization", default_value = "none")]
    normalization: CliNormalizationMode,

    /// Strip diacritics before comparison (e.g. `café` matches `cafe`).
    #[arg(long = "ignore-diacritics")]
    ignore_diacritics: bool,

    /// Selected culture override (BCP-47 / ICU locale tag) for
    /// `--comparison current-culture`. Ignored for other modes.
    #[arg(long = "culture")]
    culture: Option<String>,

    /// Seed candidates from the Linux file index (Baloo) when available.
    /// `--no-index` forces the walker even when Baloo would respond.
    #[arg(long = "use-index", conflicts_with = "no_index")]
    use_index: bool,

    /// Disable Baloo seeding even when the runtime would otherwise enable
    /// it via the user-defaults setting.
    #[arg(long = "no-index")]
    no_index: bool,

    /// Run the search inside a container instead of on the local
    /// filesystem. Requires `--container` to identify the target; the
    /// positional `path` argument is then interpreted as a container
    /// path. Mutually exclusive with the standard local-search flags
    /// that don't apply to a container target.
    #[arg(long = "container")]
    container: Option<String>,

    /// Container runtime to use when `--container` is set.
    #[arg(long = "runtime", default_value = "auto", requires = "container")]
    runtime: CliRuntimeKind,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliRuntimeKind {
    Auto,
    Docker,
    Podman,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliComparisonMode {
    Ordinal,
    CurrentCulture,
    InvariantCulture,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliNormalizationMode {
    None,
    FormC,
    FormD,
    FormKc,
    FormKd,
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
    let _log_guard = init_tracing();
    let cli = Cli::parse();

    match dispatch(cli) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            tracing::error!(error = %err, "grexa-cli error");
            eprintln!("Error: {err}");
            std::process::exit(2);
        }
    }
}

/// Install the global tracing subscriber. Logs always go to stderr at WARN
/// (override with `GREXA_LOG=...`). A rolling JSON appender mirrors every
/// event to `$XDG_STATE_HOME/grexa/grexa.log` when the state directory is
/// writable; otherwise only stderr logging stays on. The returned guard
/// must live until `main` exits or the background writer flushes early.
fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter =
        EnvFilter::try_from_env("GREXA_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr);

    let paths = AppPaths::from_env();
    let log_path = paths.state_dir.join("grexa.log");
    let (file_layer, guard) = match std::fs::create_dir_all(&paths.state_dir) {
        Ok(()) => match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => {
                let (writer, guard) = tracing_appender::non_blocking(file);
                let layer = tracing_subscriber::fmt::layer()
                    .with_writer(writer)
                    .with_target(true)
                    .with_ansi(false)
                    .with_level(true);
                (Some(layer), Some(guard))
            }
            Err(err) => {
                eprintln!("grexa-cli: log file unavailable ({err}); stderr only");
                (None, None)
            }
        },
        Err(err) => {
            eprintln!("grexa-cli: state dir unavailable ({err}); stderr only");
            (None, None)
        }
    };

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer);
    if let Some(layer) = file_layer {
        registry.with(layer).init();
    } else {
        registry.init();
    }

    guard
}

fn dispatch(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        Some(Command::Completions { shell }) => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
            Ok(0)
        }
        Some(Command::Manpage) => {
            let cmd = Cli::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut io::stdout())?;
            Ok(0)
        }
        None => {
            let search = cli.search.ok_or_else(|| {
                anyhow::anyhow!("missing required <path> <term> arguments; run `grexa-cli --help`")
            })?;
            run_search(search)
        }
    }
}

fn run_search(args: SearchArgs) -> anyhow::Result<i32> {
    if args.container.is_some() {
        return run_container_search(args);
    }
    let mut options = SearchOptions::new(&args.path, &args.term);
    options.regex = args.regex;
    options.case_sensitive = args.case_sensitive;
    options.respect_gitignore = args.gitignore;
    options.include_hidden = args.include_hidden;
    options.include_binary = args.include_binary;
    options.include_system = args.include_system;
    options.include_subfolders = !args.no_subfolders;
    options.include_symlinks = args.include_symlinks;
    options.match_file_names = args.match_files.unwrap_or_default();
    options.exclude_dirs = args.exclude_dirs.unwrap_or_default();
    options.size_limit_kb = args
        .size_limit
        .map(|value| convert_to_kb(value, args.size_unit));
    options.size_unit = args.size_unit.into();
    options.size_limit_type = args.size_type.into();
    options.string_comparison_mode = args.comparison.into();
    options.unicode_normalization_mode = args.normalization.into();
    options.diacritic_sensitive = !args.ignore_diacritics;
    options.culture = args.culture.clone();
    // `--use-index` enables Baloo seeding, `--no-index` forces it off; default
    // is unchanged from the user's stored setting (false by default).
    if args.use_index {
        options.use_file_index = true;
    }
    if args.no_index {
        options.use_file_index = false;
    }

    let cancel = CancelToken::new();
    let handler_token = cancel.clone();
    let _ = ctrlc::set_handler(move || {
        handler_token.cancel();
    });

    let summary = search_with(&options, &cancel, None)?;
    if summary.cancelled {
        eprintln!("grexa-cli: search cancelled; partial results follow");
    }

    if args.quiet {
        return Ok(if summary.results.is_empty() { 1 } else { 0 });
    }

    if args.count {
        println!("{}", summary.matches);
        return Ok(if summary.results.is_empty() { 1 } else { 0 });
    }

    if args.files_only {
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

    match args.format.into() {
        OutputFormat::Text => print_text(&summary.results),
        OutputFormat::Json => print_json(&summary.results)?,
        OutputFormat::Csv => print_csv(&summary.results),
    }

    Ok(if summary.results.is_empty() { 1 } else { 0 })
}

fn run_container_search(args: SearchArgs) -> anyhow::Result<i32> {
    use grexa_containers::{
        ContainerInfo, ContainerRuntime, ContainerRuntimeKind, ContainerSearchOptions, LiveProbe,
        RuntimeOperations, SystemCommandRunner, detect_runtimes, search_container,
    };

    let probe = LiveProbe;
    let runtimes = detect_runtimes(&probe);
    let runtime = match args.runtime {
        CliRuntimeKind::Auto => runtimes
            .into_iter()
            .find(ContainerRuntime::is_available)
            .ok_or_else(|| anyhow::anyhow!("no Docker or Podman runtime detected"))?,
        CliRuntimeKind::Docker => runtimes
            .into_iter()
            .find(|r| r.kind == ContainerRuntimeKind::Docker)
            .ok_or_else(|| anyhow::anyhow!("Docker runtime not detected"))?,
        CliRuntimeKind::Podman => runtimes
            .into_iter()
            .find(|r| r.kind == ContainerRuntimeKind::Podman)
            .ok_or_else(|| anyhow::anyhow!("Podman runtime not detected"))?,
    };

    let cli = grexa_containers::CliRuntime::new(runtime, SystemCommandRunner);
    let container = ContainerInfo {
        runtime: cli.kind(),
        id: args.container.clone().expect("container set"),
        name: args.container.clone().unwrap_or_default(),
        image: String::new(),
        status: String::new(),
        state: String::new(),
    };

    let opts = ContainerSearchOptions {
        container_path: args.path.to_string_lossy().to_string(),
        pattern: args.term.clone(),
        case_sensitive: args.case_sensitive,
        regex: args.regex,
    };
    let summary = search_container(&cli, &container, &opts)?;
    if summary.used_mirror {
        eprintln!("grexa-cli: used mirror fallback (no grep in container)");
    }
    if args.count {
        println!("{}", summary.hits.len());
        return Ok(if summary.hits.is_empty() { 1 } else { 0 });
    }
    if args.files_only {
        let mut paths: Vec<_> = summary
            .hits
            .iter()
            .map(|hit| hit.container_path.clone())
            .collect();
        paths.sort();
        paths.dedup();
        for path in paths {
            println!("{path}");
        }
        return Ok(if summary.hits.is_empty() { 1 } else { 0 });
    }
    for hit in &summary.hits {
        println!("{}:{}:{}", hit.container_path, hit.line_number, hit.line_content);
    }
    Ok(if summary.hits.is_empty() { 1 } else { 0 })
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

impl From<CliComparisonMode> for StringComparisonMode {
    fn from(value: CliComparisonMode) -> Self {
        match value {
            CliComparisonMode::Ordinal => Self::Ordinal,
            CliComparisonMode::CurrentCulture => Self::CurrentCulture,
            CliComparisonMode::InvariantCulture => Self::InvariantCulture,
        }
    }
}

impl From<CliNormalizationMode> for UnicodeNormalizationMode {
    fn from(value: CliNormalizationMode) -> Self {
        match value {
            CliNormalizationMode::None => Self::None,
            CliNormalizationMode::FormC => Self::FormC,
            CliNormalizationMode::FormD => Self::FormD,
            CliNormalizationMode::FormKc => Self::FormKC,
            CliNormalizationMode::FormKd => Self::FormKD,
        }
    }
}
