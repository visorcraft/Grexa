// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use grexa_core::{
    AppPaths, CancelToken, OutputFormat, RegexEngine, ReplaceOptions, SearchOptions, SearchResult,
    SizeLimitType, SizeUnit, StringComparisonMode, UnicodeNormalizationMode, replace_with,
    search_with,
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
    /// Replace matches in files. Runs a search first, then rewrites every
    /// matched file with the replacement string applied. Supports the same
    /// search flags as the default search command. Supports `--dry-run` to
    /// preview changes without writing to disk.
    Replace {
        /// Directory path to search.
        path: PathBuf,

        /// Search term or regex pattern.
        term: String,

        /// Replacement string. For regex mode, `$1` / `$name` / `${name}`
        /// capture references are expanded.
        replacement: String,

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
        #[arg(short = 'H', long = "include-hidden", visible_alias = "hidden")]
        include_hidden: bool,

        /// Include searchable binary/document files.
        #[arg(short = 'b', long = "include-binary")]
        include_binary: bool,

        /// Include system/dependency directories.
        #[arg(short = 's', long = "include-system", visible_alias = "no-ignore")]
        include_system: bool,

        /// Do not recurse into subdirectories.
        #[arg(short = 'd', long = "no-subfolders")]
        no_subfolders: bool,

        /// Follow symbolic links.
        #[arg(short = 'L', long = "include-symlinks")]
        include_symlinks: bool,

        /// Match whole words only (surrounded by non-word characters).
        #[arg(short = 'w', long = "whole-word")]
        whole_word: bool,

        /// Force a specific regex engine. `auto` (default) picks the fast
        /// engine and falls back to the extended engine when needed.
        #[arg(long = "regex-engine", default_value = "auto")]
        regex_engine: CliRegexEngine,

        /// Strip diacritics before comparison (e.g. `café` matches `cafe`).
        #[arg(long = "ignore-diacritics")]
        ignore_diacritics: bool,

        /// Unicode normalization to apply before comparison.
        #[arg(long = "normalization", default_value = "none")]
        normalization: CliNormalizationMode,

        /// String comparison mode for plain-text search.
        #[arg(long = "comparison", default_value = "ordinal")]
        comparison: CliComparisonMode,

        /// Culture override (BCP-47 / ICU locale tag) for comparison mode.
        #[arg(long = "culture")]
        culture: Option<String>,

        /// File name pattern, e.g. '*.rs;*.toml|-target*'.
        #[arg(short = 'm', long = "match-files")]
        match_files: Option<String>,

        /// Directories to exclude, comma/semicolon names.
        #[arg(short = 'x', long = "exclude-dirs")]
        exclude_dirs: Option<String>,

        /// Preview changes without modifying files.
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
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

    /// Force a specific regex engine. `auto` (default) picks the fast
    /// engine and falls back to the extended engine when needed; `fast`
    /// and `extended` pin one engine and error if the pattern is unsupported.
    #[arg(long = "regex-engine", default_value = "auto")]
    regex_engine: CliRegexEngine,

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

    /// Match whole words only (surrounded by non-word characters).
    #[arg(short = 'w', long = "whole-word")]
    whole_word: bool,

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

    /// Maximum number of matching lines to return. When unset, all matches
    /// are returned. Useful for large codebases where a common term produces
    /// hundreds of thousands of results.
    #[arg(long = "max-results")]
    max_results: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliRegexEngine {
    Auto,
    Fast,
    Extended,
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
        Some(Command::Replace {
            path,
            term,
            replacement,
            regex,
            case_sensitive,
            gitignore,
            include_hidden,
            include_binary,
            include_system,
            no_subfolders,
            include_symlinks,
            whole_word,
            regex_engine,
            ignore_diacritics,
            normalization,
            comparison,
            culture,
            match_files,
            exclude_dirs,
            dry_run,
        }) => run_replace(
            path,
            term,
            replacement,
            regex,
            case_sensitive,
            gitignore,
            include_hidden,
            include_binary,
            include_system,
            no_subfolders,
            include_symlinks,
            whole_word,
            regex_engine,
            ignore_diacritics,
            normalization,
            comparison,
            culture,
            match_files,
            exclude_dirs,
            dry_run,
        ),
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
    if options.search_term.is_empty() {
        anyhow::bail!("search term must not be empty");
    }
    if options.search_term.len() > 4096 {
        anyhow::bail!("search term exceeds maximum length of 4096 characters");
    }
    options.regex = args.regex;
    options.regex_engine = match args.regex_engine {
        CliRegexEngine::Auto => RegexEngine::Auto,
        CliRegexEngine::Fast => RegexEngine::Fast,
        CliRegexEngine::Extended => RegexEngine::Extended,
    };
    options.case_sensitive = args.case_sensitive;
    options.respect_gitignore = args.gitignore;
    options.include_hidden = args.include_hidden;
    options.include_binary = args.include_binary;
    options.include_system = args.include_system;
    options.include_subfolders = !args.no_subfolders;
    options.include_symlinks = args.include_symlinks;
    options.whole_word = args.whole_word;
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
    options.max_results = args.max_results;

    let cancel = CancelToken::new();
    let handler_token = cancel.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        handler_token.cancel();
    }) {
        tracing::warn!("Ctrl+C handler not installed: {err}");
    }

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

#[allow(clippy::too_many_arguments)]
fn run_replace(
    path: PathBuf,
    term: String,
    replacement: String,
    regex: bool,
    case_sensitive: bool,
    gitignore: bool,
    include_hidden: bool,
    include_binary: bool,
    include_system: bool,
    no_subfolders: bool,
    include_symlinks: bool,
    whole_word: bool,
    regex_engine: CliRegexEngine,
    ignore_diacritics: bool,
    normalization: CliNormalizationMode,
    comparison: CliComparisonMode,
    culture: Option<String>,
    match_files: Option<String>,
    exclude_dirs: Option<String>,
    dry_run: bool,
) -> anyhow::Result<i32> {
    let mut search = SearchOptions::new(&path, &term);
    if search.search_term.is_empty() {
        anyhow::bail!("search term must not be empty");
    }
    if search.search_term.len() > 4096 {
        anyhow::bail!("search term exceeds maximum length of 4096 characters");
    }
    search.regex = regex;
    search.case_sensitive = case_sensitive;
    search.whole_word = whole_word;
    search.regex_engine = match regex_engine {
        CliRegexEngine::Auto => RegexEngine::Auto,
        CliRegexEngine::Fast => RegexEngine::Fast,
        CliRegexEngine::Extended => RegexEngine::Extended,
    };
    search.respect_gitignore = gitignore;
    search.include_hidden = include_hidden;
    search.include_binary = include_binary;
    search.include_system = include_system;
    search.include_subfolders = !no_subfolders;
    search.include_symlinks = include_symlinks;
    search.match_file_names = match_files.unwrap_or_default();
    search.exclude_dirs = exclude_dirs.unwrap_or_default();
    search.diacritic_sensitive = !ignore_diacritics;
    search.unicode_normalization_mode = match normalization {
        CliNormalizationMode::None => UnicodeNormalizationMode::None,
        CliNormalizationMode::FormC => UnicodeNormalizationMode::FormC,
        CliNormalizationMode::FormD => UnicodeNormalizationMode::FormD,
        CliNormalizationMode::FormKc => UnicodeNormalizationMode::FormKC,
        CliNormalizationMode::FormKd => UnicodeNormalizationMode::FormKD,
    };
    search.string_comparison_mode = match comparison {
        CliComparisonMode::Ordinal => StringComparisonMode::Ordinal,
        CliComparisonMode::CurrentCulture => StringComparisonMode::CurrentCulture,
        CliComparisonMode::InvariantCulture => StringComparisonMode::InvariantCulture,
    };
    search.culture = culture;

    if dry_run {
        let cancel = CancelToken::new();
        let summary = search_with(&search, &cancel, None)?;
        if summary.results.is_empty() {
            eprintln!("grexa-cli: no matches found");
            return Ok(1);
        }
        eprintln!(
            "grexa-cli: dry run — {} matches in {} files would be replaced with {:?}",
            summary.matches, summary.files_matched, replacement
        );
        for result in &summary.results {
            println!(
                "{}:{}:{}",
                result.full_path.display(),
                result.line_number,
                result.line_content
            );
        }
        return Ok(0);
    }

    let options = ReplaceOptions {
        search,
        replacement,
    };
    let cancel = CancelToken::new();
    let handler_token = cancel.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        handler_token.cancel();
    }) {
        tracing::warn!("Ctrl+C handler not installed: {err}");
    }

    let summary = replace_with(&options, &cancel, None)?;
    if summary.cancelled {
        eprintln!("grexa-cli: replace cancelled; partial results follow");
    }

    eprintln!(
        "grexa-cli: {} files modified, {} matches replaced, {} files unchanged, {} failures",
        summary.files_modified,
        summary.matches_replaced,
        summary.files_unchanged,
        summary.failures.len()
    );

    for report in &summary.reports {
        println!("{}: {} replacements", report.path.display(), report.matches_replaced);
    }
    for failure in &summary.failures {
        eprintln!("{}: {}", failure.path.display(), failure.error);
    }

    Ok(if summary.failures.is_empty() && summary.files_modified > 0 {
        0
    } else if summary.files_modified == 0 {
        1
    } else {
        2
    })
}

/// Resolve a user-supplied `--container` value against the live container
/// list, matching by exact id, exact name, or id prefix. Returns `None` when
/// nothing matches, so a bogus or flag-shaped value (e.g. `--privileged`) can
/// never be forwarded to the runtime as a container identifier. An empty
/// request never matches (an empty prefix would otherwise match everything).
fn select_container<'a>(
    requested: &str,
    available: &'a [grexa_containers::ContainerInfo],
) -> Option<&'a grexa_containers::ContainerInfo> {
    if requested.is_empty() {
        return None;
    }
    available
        .iter()
        .find(|c| c.id == requested || c.name == requested || c.id.starts_with(requested))
}

fn run_container_search(args: SearchArgs) -> anyhow::Result<i32> {
    use grexa_containers::{
        ContainerRuntime, ContainerRuntimeKind, ContainerSearchOptions, LiveProbe,
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

    // Resolve the requested container against the live list rather than
    // trusting the raw `--container` string. This guarantees the id forwarded
    // to `docker/podman exec` is a real container identifier (the `--`
    // terminator in the runtime layer is the second line of defense).
    let requested = args.container.clone().expect("container set");
    let available = cli
        .list_containers()
        .map_err(|err| anyhow::anyhow!("failed to list containers: {err}"))?;
    let container = select_container(&requested, &available).ok_or_else(|| {
        anyhow::anyhow!(
            "no running container matches '{requested}'; pass an id or name from the container list"
        )
    })?;

    let opts = ContainerSearchOptions {
        container_path: args.path.to_string_lossy().to_string(),
        pattern: args.term.clone(),
        case_sensitive: args.case_sensitive,
        regex: args.regex,
        diacritic_sensitive: !args.ignore_diacritics,
        unicode_normalization_mode: match args.normalization {
            CliNormalizationMode::None => UnicodeNormalizationMode::None,
            CliNormalizationMode::FormC => UnicodeNormalizationMode::FormC,
            CliNormalizationMode::FormD => UnicodeNormalizationMode::FormD,
            CliNormalizationMode::FormKc => UnicodeNormalizationMode::FormKC,
            CliNormalizationMode::FormKd => UnicodeNormalizationMode::FormKD,
        },
        string_comparison_mode: match args.comparison {
            CliComparisonMode::Ordinal => StringComparisonMode::Ordinal,
            CliComparisonMode::CurrentCulture => StringComparisonMode::CurrentCulture,
            CliComparisonMode::InvariantCulture => StringComparisonMode::InvariantCulture,
        },
        culture: args.culture.clone(),
    };
    let summary = search_container(&cli, container, &opts)?;
    let _ = grexa_containers::prune_mirrors(3600);
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
    let tty = io::stdout().is_terminal();
    for hit in &summary.hits {
        println!(
            "{}:{}:{}",
            sanitize_for_terminal(&hit.container_path, tty),
            hit.line_number,
            sanitize_for_terminal(&hit.line_content, tty)
        );
    }
    Ok(if summary.hits.is_empty() { 1 } else { 0 })
}

/// Neutralize terminal control sequences in attacker-controlled output (a
/// matched line, or a crafted file path) before printing to a TTY. Matched
/// file content can contain ANSI/OSC escapes that rewrite the screen, change
/// the window title, or worse; ripgrep/git guard against this the same way.
/// When stdout is *not* a terminal (piped/redirected), bytes pass through
/// unchanged so downstream tools see faithful output.
fn sanitize_for_terminal(value: &str, is_tty: bool) -> std::borrow::Cow<'_, str> {
    // A control char is anything dangerous to a terminal: C0 (incl. ESC, BEL,
    // CR, LF, NUL), DEL, and C1. Tabs are kept — they're benign and common.
    let is_dangerous = |c: char| c.is_control() && c != '\t';

    if !is_tty || !value.chars().any(is_dangerous) {
        return std::borrow::Cow::Borrowed(value);
    }
    std::borrow::Cow::Owned(
        value
            .chars()
            .map(|c| if is_dangerous(c) { '\u{FFFD}' } else { c })
            .collect(),
    )
}

fn print_text(results: &[SearchResult]) {
    let tty = io::stdout().is_terminal();
    for result in results {
        println!(
            "{}:{}:{}:{}",
            sanitize_for_terminal(&result.full_path.display().to_string(), tty),
            result.line_number,
            result.column_number,
            sanitize_for_terminal(&result.line_content, tty)
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
    let value = neutralize_spreadsheet_formula(value);
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn neutralize_spreadsheet_formula(value: &str) -> String {
    if value
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '=' | '+' | '-' | '@' | '\t' | '\r' | '\n'))
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

fn convert_to_kb(value: u64, unit: CliSizeUnit) -> u64 {
    // Saturate rather than overflow: clap accepts the full u64 range for
    // `--size-limit`, so `value * 1024 * 1024` would panic in debug builds and
    // silently wrap in release. A saturated ceiling is a correct size limit.
    match unit {
        CliSizeUnit::KB => value,
        CliSizeUnit::MB => value.saturating_mul(1024),
        CliSizeUnit::GB => value.saturating_mul(1024).saturating_mul(1024),
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

#[cfg(test)]
mod tests {
    use super::*;
    use grexa_containers::{ContainerInfo, ContainerRuntimeKind};

    fn ci(id: &str, name: &str) -> ContainerInfo {
        ContainerInfo {
            runtime: ContainerRuntimeKind::Docker,
            id: id.to_string(),
            name: name.to_string(),
            image: String::new(),
            status: String::new(),
            state: String::new(),
        }
    }

    #[test]
    fn convert_to_kb_saturates_instead_of_overflowing() {
        assert_eq!(convert_to_kb(5, CliSizeUnit::MB), 5 * 1024);
        // u64::MAX GiB would overflow `* 1024 * 1024` (panic in debug); it must
        // saturate to a valid (huge) ceiling instead.
        assert_eq!(convert_to_kb(u64::MAX, CliSizeUnit::GB), u64::MAX);
    }

    #[test]
    fn sanitize_for_terminal_strips_controls_on_tty_passes_through_otherwise() {
        let evil = "ok\u{1b}[31mRED\u{7}\r\nmore";

        // On a TTY: ANSI ESC, BEL, CR and LF are neutralized.
        let cleaned = sanitize_for_terminal(evil, true);
        assert!(!cleaned.contains('\u{1b}'), "ESC must be stripped");
        assert!(!cleaned.contains('\u{7}'), "BEL must be stripped");
        assert!(!cleaned.contains('\r'), "CR must be stripped");
        assert!(!cleaned.contains('\n'), "LF must be stripped");
        assert!(cleaned.contains("ok") && cleaned.contains("RED") && cleaned.contains("more"));

        // Tabs are preserved (legitimate, harmless).
        assert_eq!(sanitize_for_terminal("a\tb", true), "a\tb");

        // Not a TTY: output is byte-faithful for piping into other tools.
        assert_eq!(sanitize_for_terminal(evil, false), evil);
    }

    #[test]
    fn select_container_matches_by_id_name_or_prefix_and_rejects_bogus() {
        let available = vec![ci("abc123def", "web"), ci("999fff000", "db")];

        assert_eq!(
            select_container("web", &available).map(|c| c.id.as_str()),
            Some("abc123def"),
            "exact name should match"
        );
        assert_eq!(
            select_container("abc123def", &available).map(|c| c.id.as_str()),
            Some("abc123def"),
            "exact id should match"
        );
        assert_eq!(
            select_container("abc", &available).map(|c| c.id.as_str()),
            Some("abc123def"),
            "id prefix should match"
        );

        // A flag-shaped value must never resolve to a real container.
        assert!(select_container("--privileged", &available).is_none());
        assert!(select_container("nonexistent", &available).is_none());
        // An empty request must not match everything via the empty prefix.
        assert!(select_container("", &available).is_none());
    }
}
