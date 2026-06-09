// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Container search.
//!
//! Two strategies, chosen automatically per container:
//!
//! 1. **Direct grep** — if the container has `grep` on `$PATH`, exec
//!    `grep -rnH <pattern> <path>` (or its BusyBox-compatible form) and
//!    parse the stdout. This is the preferred path: zero local disk, no
//!    side effects.
//! 2. **Mirror fallback** — if `grep` is missing (distroless, minimal
//!    image), `docker cp` the requested path to a cache directory under
//!    `$XDG_CACHE_HOME/grexa/container-mirrors/<runtime>/<container>/`
//!    and run the local grexa-core search engine against that mirror.
//!    Result paths are rewritten back to their in-container form so the
//!    user never sees the mirror directory.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use grexa_core::{AppPaths, ContextPreviewResult, SearchOptions, context_preview, search};
use serde::{Deserialize, Serialize};

use crate::runtime::{RuntimeError, RuntimeOperations};
use crate::{ContainerInfo, ContainerRuntimeKind};

const CONTAINER_SEARCH_TIMEOUT_SECS: u64 = 30;

/// Single grep hit produced by either backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerSearchHit {
    pub runtime: ContainerRuntimeKind,
    pub container_id: String,
    /// Path *inside the container*, even when produced by the mirror
    /// fallback.
    pub container_path: String,
    pub line_number: usize,
    /// 1-based byte column where the match starts. `1` when the parser
    /// runs without a pattern (back-compat) or when the pattern was
    /// not found inside the reported line.
    #[serde(default = "default_column")]
    pub column_number: usize,
    pub line_content: String,
}

fn default_column() -> usize {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerSearchSummary {
    pub hits: Vec<ContainerSearchHit>,
    pub used_mirror: bool,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ContainerSearchOptions {
    pub container_path: String,
    pub pattern: String,
    pub case_sensitive: bool,
    pub regex: bool,
    pub whole_word: bool,
    pub max_results: Option<usize>,
    pub diacritic_sensitive: bool,
    pub unicode_normalization_mode: grexa_core::UnicodeNormalizationMode,
    pub string_comparison_mode: grexa_core::StringComparisonMode,
    pub culture: Option<String>,
}

impl ContainerSearchOptions {
    pub fn new(path: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            container_path: path.into(),
            pattern: pattern.into(),
            case_sensitive: false,
            regex: false,
            whole_word: false,
            max_results: None,
            diacritic_sensitive: true,
            unicode_normalization_mode: grexa_core::UnicodeNormalizationMode::None,
            string_comparison_mode: grexa_core::StringComparisonMode::Ordinal,
            culture: None,
        }
    }

    fn needs_normalization(&self) -> bool {
        !self.diacritic_sensitive
            || self.unicode_normalization_mode != grexa_core::UnicodeNormalizationMode::None
            || self.string_comparison_mode != grexa_core::StringComparisonMode::Ordinal
            || self.culture.is_some()
    }
}

/// Search inside one running container. Picks direct-grep when available
/// and falls back to a local mirror when not. Both paths produce the same
/// summary shape so the GUI never has to branch.
pub fn search_container<R: RuntimeOperations>(
    runtime: &R,
    container: &ContainerInfo,
    options: &ContainerSearchOptions,
) -> Result<ContainerSearchSummary, RuntimeError> {
    let started = std::time::Instant::now();
    let has_grep = runtime.has_grep(&container.id).unwrap_or(false);

    let (mut hits, used_mirror) = if has_grep && !options.needs_normalization() {
        (direct_grep(runtime, container, options)?, false)
    } else {
        if has_grep {
            tracing::debug!(
                "normalization-affecting options set; using mirror search instead of container grep"
            );
        }
        (mirror_search(runtime, container, options)?, true)
    };
    if let Some(max) = options.max_results {
        hits.truncate(max);
    }

    Ok(ContainerSearchSummary {
        hits,
        used_mirror,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

/// Exec `grep` inside the container and parse line-tagged output.
fn direct_grep<R: RuntimeOperations>(
    runtime: &R,
    container: &ContainerInfo,
    options: &ContainerSearchOptions,
) -> Result<Vec<ContainerSearchHit>, RuntimeError> {
    let mut argv: Vec<&str> = vec!["grep", "-rnHZ"];
    if options.regex {
        argv.push("-E");
    } else {
        argv.push("-F");
    }
    if !options.case_sensitive {
        argv.push("-i");
    }
    if options.whole_word {
        argv.push("-w");
    }
    argv.push("--");
    argv.push(&options.pattern);
    argv.push(&options.container_path);

    let start = std::time::Instant::now();
    let mut result = runtime.exec_capture(&container.id, &argv)?;
    if result.status == 2 && grep_rejected_option(&result.stderr) {
        // BusyBox grep has no -Z; retry with colon-delimited output, which
        // the parser handles via its colon-splitting fallback.
        argv[1] = "-rnH";
        result = runtime.exec_capture(&container.id, &argv)?;
    }
    tracing::debug!(
        elapsed_ms = start.elapsed().as_millis(),
        timeout_secs = CONTAINER_SEARCH_TIMEOUT_SECS,
        "direct_grep completed"
    );
    // grep exits 1 when nothing matched — that's a successful empty search.
    if result.status != 0 && result.status != 1 {
        return Err(RuntimeError::Cli {
            cli: PathBuf::from(format!("{:?}", runtime.kind())),
            status: result.status,
            stderr: String::from_utf8_lossy(&result.stderr).into_owned(),
        });
    }

    Ok(parse_grep_output_with_pattern(
        &String::from_utf8_lossy(&result.stdout),
        runtime.kind(),
        &container.id,
        Some(GrepPattern {
            needle: &options.pattern,
            regex: options.regex,
            case_sensitive: options.case_sensitive,
        }),
    ))
}

fn grep_rejected_option(stderr: &[u8]) -> bool {
    let text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    text.contains("unrecognized option")
        || text.contains("invalid option")
        || text.contains("usage:")
}

/// What `parse_grep_output_with_pattern` needs to re-scan each line
/// and emit one hit per match occurrence + correct column numbers.
#[derive(Debug, Clone, Copy)]
pub struct GrepPattern<'a> {
    pub needle: &'a str,
    pub regex: bool,
    pub case_sensitive: bool,
}

/// Parse colon-delimited `grep -rnH` output. Back-compat shim that emits
/// one hit per matched line at column 1. Callers that have the pattern
/// available should prefer [`parse_grep_output_with_pattern`].
pub fn parse_grep_output(
    stdout: &str,
    kind: ContainerRuntimeKind,
    container_id: &str,
) -> Vec<ContainerSearchHit> {
    parse_grep_output_with_pattern(stdout, kind, container_id, None)
}

/// Parse colon-delimited `grep -rnH` output. The first colon separates
/// the file path from the line number; the second separates line number
/// from content. Both halves may themselves contain colons; we split
/// greedily on the first two colons only.
///
/// When `pattern` is supplied, the parser re-scans each line for all
/// occurrences of the pattern and emits one [`ContainerSearchHit`] per
/// match with the correct `column_number`. When `pattern` is `None`,
/// the parser falls back to one hit per matched line at column 1
/// (which is what `grep` reports by default).
pub fn parse_grep_output_with_pattern(
    stdout: &str,
    kind: ContainerRuntimeKind,
    container_id: &str,
    pattern: Option<GrepPattern<'_>>,
) -> Vec<ContainerSearchHit> {
    let compiled = pattern.and_then(|p| {
        if p.regex {
            regex::RegexBuilder::new(p.needle)
                .case_insensitive(!p.case_sensitive)
                .build()
                .ok()
        } else {
            // Literal mode: build a literal-match regex so we get one
            // pass over the line content regardless of case.
            let escaped = regex::escape(p.needle);
            regex::RegexBuilder::new(&escaped)
                .case_insensitive(!p.case_sensitive)
                .build()
                .ok()
        }
    });

    let mut hits = Vec::new();
    for line in stdout.lines() {
        let (path, lineno_str, content) = if let Some(null_pos) = line.find('\0') {
            let p = &line[..null_pos];
            let rest = &line[null_pos + 1..];
            let colon_pos = match rest.find(':') {
                Some(pos) => pos,
                None => continue,
            };
            (p, &rest[..colon_pos], &rest[colon_pos + 1..])
        } else {
            let mut first_colon = None;
            let mut second_colon = None;
            for (i, byte) in line.bytes().enumerate() {
                if byte == b':' {
                    if first_colon.is_none() {
                        first_colon = Some(i);
                    } else if second_colon.is_none() {
                        second_colon = Some(i);
                        break;
                    }
                }
            }
            let (Some(first), Some(second)) = (first_colon, second_colon) else {
                continue;
            };
            (&line[..first], &line[first + 1..second], &line[second + 1..])
        };
        let Ok(line_number) = lineno_str.parse::<usize>() else {
            continue;
        };

        if let Some(re) = compiled.as_ref() {
            let mut emitted = false;
            for mat in re.find_iter(content) {
                emitted = true;
                hits.push(ContainerSearchHit {
                    runtime: kind,
                    container_id: container_id.to_string(),
                    container_path: path.to_string(),
                    line_number,
                    column_number: content
                        .get(..mat.start())
                        .map(|prefix| prefix.chars().count() + 1)
                        .unwrap_or(mat.start() + 1),
                    line_content: content.to_string(),
                });
            }
            if !emitted {
                // grep matched the line but our local regex didn't.
                // Surface the line at column 1 so the user still sees
                // the hit; differs from grep only when the pattern is
                // a regex feature we can't compile locally.
                hits.push(ContainerSearchHit {
                    runtime: kind,
                    container_id: container_id.to_string(),
                    container_path: path.to_string(),
                    line_number,
                    column_number: 1,
                    line_content: content.to_string(),
                });
            }
        } else {
            hits.push(ContainerSearchHit {
                runtime: kind,
                container_id: container_id.to_string(),
                container_path: path.to_string(),
                line_number,
                column_number: 1,
                line_content: content.to_string(),
            });
        }
    }
    hits
}

/// Mirror the requested path into a local cache directory and run the
/// in-process Grexa search engine against it. Result paths are rewritten so
/// they keep the `container_path` form the user typed.
fn mirror_search<R: RuntimeOperations>(
    runtime: &R,
    container: &ContainerInfo,
    options: &ContainerSearchOptions,
) -> Result<Vec<ContainerSearchHit>, RuntimeError> {
    let dest = container_mirror_dir(&container.id, runtime.kind());
    let local = runtime.archive_path(&container.id, &options.container_path, &dest)?;

    let mut search_opts = SearchOptions::new(&local, &options.pattern);
    search_opts.regex = options.regex;
    search_opts.case_sensitive = options.case_sensitive;
    search_opts.whole_word = options.whole_word;
    search_opts.max_results = options.max_results;
    search_opts.diacritic_sensitive = options.diacritic_sensitive;
    search_opts.unicode_normalization_mode = options.unicode_normalization_mode;
    search_opts.string_comparison_mode = options.string_comparison_mode;
    search_opts.culture = options.culture.clone();
    search_opts.include_subfolders = true;
    let summary = search(&search_opts).map_err(|err| RuntimeError::Unsupported(err.to_string()))?;

    let local_str = local.to_string_lossy().to_string();
    let mut hits = Vec::new();
    for result in summary.results {
        let container_path = rewrite_path(&result.full_path, &local_str, &options.container_path);
        hits.push(ContainerSearchHit {
            runtime: runtime.kind(),
            container_id: container.id.clone(),
            container_path,
            line_number: result.line_number,
            column_number: result.column_number,
            line_content: result.line_content,
        });
    }
    Ok(hits)
}

fn container_mirror_dir(container_id: &str, kind: ContainerRuntimeKind) -> PathBuf {
    let paths = AppPaths::from_env();
    let runtime_label = match kind {
        ContainerRuntimeKind::Docker => "docker",
        ContainerRuntimeKind::Podman => "podman",
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    paths
        .cache_dir
        .join("container-mirrors")
        .join(runtime_label)
        .join(container_id)
        .join(stamp)
}

/// `local` is the absolute path of the materialized mirror. Rewrite a file
/// path that sits beneath `local` into the in-container form by joining its
/// suffix onto `container_path`.
fn rewrite_path(full_path: &Path, local_root: &str, container_root: &str) -> String {
    let path_str = full_path.to_string_lossy();
    let local_root = local_root.trim_end_matches('/');
    let suffix = if path_str.as_ref() == local_root {
        ""
    } else if let Some(stripped) = path_str.strip_prefix(local_root) {
        stripped.trim_start_matches('/')
    } else {
        return path_str.into_owned();
    };

    if suffix.is_empty() {
        container_root.to_string()
    } else {
        let trimmed_container_root = container_root.trim_end_matches('/');
        format!("{trimmed_container_root}/{suffix}")
    }
}

/// Build a context preview for a file *inside* a container. Mirrors the
/// in-container path to a temp directory via `archive_path`, runs the
/// standard `grexa_core::context_preview`, and rewrites the result paths
/// so the UI still shows the user the in-container path.
///
/// This is the function the Phase 14 preview UI calls when a result row
/// originated from a container search.
pub fn container_context_preview<R: RuntimeOperations>(
    runtime: &R,
    container: &ContainerInfo,
    container_path: &str,
    line_number: usize,
    lines_before: u8,
    lines_after: u8,
) -> Result<ContextPreviewResult, RuntimeError> {
    let dest = container_mirror_dir(&container.id, runtime.kind()).join("preview");
    let local = runtime.archive_path(&container.id, container_path, &dest)?;
    let mut preview = context_preview(&local, line_number, lines_before, lines_after)
        .map_err(|err| RuntimeError::Unsupported(err.to_string()))?;
    preview.full_path = PathBuf::from(container_path);
    Ok(preview)
}

/// Prune mirrors older than `max_age_secs`. Called on startup and after each
/// container search; missing or empty cache directories are not an error.
pub fn prune_mirrors(max_age_secs: u64) -> std::io::Result<()> {
    let paths = AppPaths::from_env();
    prune_mirrors_under(&paths.cache_dir.join("container-mirrors"), max_age_secs)
}

/// `prune_mirrors` against an explicit mirrors root, so tests can target a
/// tempdir instead of the user's real cache.
fn prune_mirrors_under(root: &Path, max_age_secs: u64) -> std::io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let cutoff = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().saturating_sub(max_age_secs))
        .unwrap_or_default();

    for runtime_entry in std::fs::read_dir(root)? {
        let runtime_entry = runtime_entry?;
        for container_entry in std::fs::read_dir(runtime_entry.path())? {
            let container_entry = container_entry?;
            for stamp_entry in std::fs::read_dir(container_entry.path())? {
                let stamp_entry = stamp_entry?;
                let stamp_name = stamp_entry.file_name();
                let stamp: u64 = stamp_name.to_string_lossy().parse().unwrap_or(0);
                if stamp == 0 || stamp < cutoff {
                    let _ = std::fs::remove_dir_all(stamp_entry.path());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use crate::runtime::{CliRuntime, CommandResult, MockCommandRunner};
    use crate::{ContainerInfo, ContainerRuntime, ContainerRuntimeKind};

    use super::*;

    fn fake_container() -> ContainerInfo {
        ContainerInfo {
            runtime: ContainerRuntimeKind::Podman,
            id: "abc123".to_string(),
            name: "web".to_string(),
            image: "alpine".to_string(),
            status: "Up".to_string(),
            state: "running".to_string(),
        }
    }

    fn cli_runtime(runner: MockCommandRunner) -> CliRuntime<MockCommandRunner> {
        CliRuntime::new(
            ContainerRuntime {
                kind: ContainerRuntimeKind::Podman,
                socket_path: None,
                cli_path: Some(PathBuf::from("/usr/bin/podman")),
                rootless: true,
            },
            runner,
        )
    }

    #[test]
    fn parse_grep_output_basic() {
        let hits = parse_grep_output(
            "/etc/hostname:1:my-host\n/etc/hosts:3:127.0.0.1 localhost\n",
            ContainerRuntimeKind::Podman,
            "abc",
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].container_path, "/etc/hostname");
        assert_eq!(hits[0].line_number, 1);
        assert_eq!(hits[0].column_number, 1);
        assert_eq!(hits[1].line_content, "127.0.0.1 localhost");
    }

    #[test]
    fn parse_grep_output_with_pattern_emits_one_hit_per_match() {
        let stdout = "/var/log/syslog:7:TODO ship it; TODO TODO again\n";
        let hits = parse_grep_output_with_pattern(
            stdout,
            ContainerRuntimeKind::Podman,
            "abc",
            Some(GrepPattern {
                needle: "TODO",
                regex: false,
                case_sensitive: true,
            }),
        );
        assert_eq!(hits.len(), 3, "got {hits:?}");
        let columns: Vec<_> = hits.iter().map(|h| h.column_number).collect();
        assert_eq!(columns, vec![1, 15, 20]);
    }

    #[test]
    fn parse_grep_output_with_pattern_handles_case_insensitive() {
        let stdout = "/etc/issue:1:Ubuntu OS ubuntu LTS\n";
        let hits = parse_grep_output_with_pattern(
            stdout,
            ContainerRuntimeKind::Docker,
            "abc",
            Some(GrepPattern {
                needle: "ubuntu",
                regex: false,
                case_sensitive: false,
            }),
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].column_number, 1);
        assert_eq!(hits[1].column_number, 11);
    }

    #[test]
    fn parse_grep_output_with_pattern_uses_regex() {
        let stdout = "/data/notes:42:abc-1 abc-22 xyz abc-333\n";
        let hits = parse_grep_output_with_pattern(
            stdout,
            ContainerRuntimeKind::Podman,
            "abc",
            Some(GrepPattern {
                needle: r"abc-\d+",
                regex: true,
                case_sensitive: true,
            }),
        );
        assert_eq!(hits.len(), 3);
        let cols: Vec<_> = hits.iter().map(|h| h.column_number).collect();
        // "abc-1 abc-22 xyz abc-333" — matches at columns 1, 7, 18.
        assert_eq!(cols, vec![1, 7, 18]);
    }

    #[test]
    fn parse_grep_output_skips_unparseable_lines() {
        let hits = parse_grep_output(
            "broken line with no colons\n/path:42:ok\nweird::\n",
            ContainerRuntimeKind::Docker,
            "abc",
        );
        // Only the well-formed line survives; "weird::" parses as (weird, "", "") but lineno fails.
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line_number, 42);
    }

    #[test]
    fn direct_grep_invocation_uses_argv_array() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        runner
            .push(CommandResult::success("/etc/hostname:1:my-host\n/etc/hosts:5:another match\n"));
        let runtime = cli_runtime(runner.clone());

        let opts = ContainerSearchOptions::new("/etc", "host");
        let summary = search_container(&runtime, &fake_container(), &opts).unwrap();
        assert!(!summary.used_mirror);
        assert_eq!(summary.hits.len(), 2);

        let inv = runner.invocations();
        // First invocation: has_grep probe via `which grep`.
        // Args are `exec -- <container> which grep`, so `which` is at index 3.
        assert_eq!(inv[0].args[3], OsString::from("which"));
        // Second invocation: the actual grep call.
        let grep_args = &inv[1].args;
        assert!(grep_args.iter().any(|a| a == &OsString::from("grep")));
        assert!(grep_args.iter().any(|a| a == &OsString::from("-rnHZ")));
        assert!(grep_args.iter().any(|a| a == &OsString::from("-F")));
        assert!(grep_args.iter().any(|a| a == &OsString::from("-i")));
        assert!(grep_args.iter().any(|a| a == &OsString::from("host")));
    }

    #[test]
    fn direct_grep_exit_one_is_empty_search_not_error() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        // grep returns 1 when no match found.
        runner.push(CommandResult {
            status: 1,
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
        let runtime = cli_runtime(runner);
        let summary = search_container(
            &runtime,
            &fake_container(),
            &ContainerSearchOptions::new("/etc", "missing"),
        )
        .unwrap();
        assert!(summary.hits.is_empty());
    }

    #[test]
    fn direct_grep_retries_without_z_for_busybox_grep() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        runner.push(CommandResult {
            status: 2,
            stdout: Vec::new(),
            stderr: b"grep: unrecognized option: Z\nBusyBox v1.36.1 multi-call binary.\nUsage: grep ...\n".to_vec(),
        });
        runner.push(CommandResult::success("/etc/hostname:1:my-host\n"));
        let runtime = cli_runtime(runner.clone());

        let summary = search_container(
            &runtime,
            &fake_container(),
            &ContainerSearchOptions::new("/etc", "host"),
        )
        .unwrap();
        assert!(!summary.used_mirror);
        assert_eq!(summary.hits.len(), 1);
        assert_eq!(summary.hits[0].container_path, "/etc/hostname");

        let inv = runner.invocations();
        assert!(inv[1].args.iter().any(|a| a == &OsString::from("-rnHZ")));
        assert!(inv[2].args.iter().any(|a| a == &OsString::from("-rnH")));
        assert!(!inv[2].args.iter().any(|a| a == &OsString::from("-rnHZ")));
    }

    #[test]
    fn direct_grep_whole_word_passes_w_flag() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        runner.push(CommandResult::success("/etc/hostname:1:my-host\n"));
        let runtime = cli_runtime(runner.clone());

        let mut opts = ContainerSearchOptions::new("/etc", "host");
        opts.whole_word = true;
        let summary = search_container(&runtime, &fake_container(), &opts).unwrap();
        assert!(!summary.used_mirror);

        let grep_args = &runner.invocations()[1].args;
        assert!(grep_args.iter().any(|a| a == &OsString::from("-w")));
    }

    #[test]
    fn search_container_truncates_to_max_results() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        runner.push(CommandResult::success(
            "/etc/hostname:1:my-host\n/etc/hosts:5:another host\n/etc/issue:2:host again\n",
        ));
        let runtime = cli_runtime(runner);

        let mut opts = ContainerSearchOptions::new("/etc", "host");
        opts.max_results = Some(2);
        let summary = search_container(&runtime, &fake_container(), &opts).unwrap();
        assert_eq!(summary.hits.len(), 2);
    }

    #[test]
    fn normalization_options_force_mirror_even_with_grep() {
        let runner = MockCommandRunner::default();
        // has_grep probe says grep exists; normalization options must still
        // route to the mirror path.
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        // archive_path cp.
        runner.push(CommandResult::success(""));
        let runtime = cli_runtime(runner.clone());

        let mut opts = ContainerSearchOptions::new("/data", "TODO");
        opts.diacritic_sensitive = false;
        let _ = search_container(&runtime, &fake_container(), &opts);

        let inv = runner.invocations();
        assert!(
            inv.iter()
                .any(|i| i.args.iter().any(|a| a == &OsString::from("cp"))),
            "mirror path must call `podman cp`"
        );
        assert!(
            !inv.iter()
                .any(|i| i.args.iter().any(|a| a == &OsString::from("-rnHZ"))),
            "grep must never be exec'd when normalization options are set"
        );
    }

    #[test]
    fn mirror_search_when_no_grep() {
        let dir = tempfile::tempdir().unwrap();
        let runner = MockCommandRunner::default();
        // has_grep probe — distroless / busybox without grep.
        runner.push(CommandResult::success(""));
        // archive_path cp — empty stdout, success status.
        runner.push(CommandResult::success(""));
        let runtime = cli_runtime(runner.clone());

        // Build a synthetic mirror directory under the cache root so the
        // search engine has something to walk. Container mirror dir is
        // computed dynamically per call; spy on the archive_path argv to
        // recover the temp dir and seed it.
        let mut container = fake_container();
        container.id = "mirrordrop".to_string();
        let opts = ContainerSearchOptions::new("/data", "TODO");

        // We can't run the live `archive_path` because the mock doesn't
        // materialize files. Instead, test the parse_grep_output path
        // directly via direct_grep (already covered), and test
        // `mirror_search` end-to-end is left to an integration test in a
        // future revision.
        // This test pins the behavior that has_grep returning false
        // attempts the archive path.
        let _ = search_container(&runtime, &container, &opts);
        let inv = runner.invocations();
        assert!(
            inv.iter()
                .any(|i| i.args.iter().any(|a| a == &OsString::from("cp"))),
            "mirror path must call `podman cp`"
        );
        drop(dir);
    }

    #[test]
    fn rewrite_path_strips_local_root() {
        let local = "/tmp/cache/mirror/abc/123";
        assert_eq!(
            rewrite_path(Path::new("/tmp/cache/mirror/abc/123/foo.txt"), local, "/data"),
            "/data/foo.txt"
        );
        assert_eq!(rewrite_path(Path::new("/tmp/cache/mirror/abc/123"), local, "/data"), "/data");
        assert_eq!(
            rewrite_path(Path::new("/tmp/cache/mirror/abc/123/a/b"), local, "/data/"),
            "/data/a/b"
        );
    }

    #[test]
    fn parse_grep_output_with_colon_in_filename() {
        let stdout = "path:with:colons:42:matched content\n";
        let hits = parse_grep_output(stdout, ContainerRuntimeKind::Docker, "abc123");
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn parse_grep_output_null_delimited_handles_colons_in_filename() {
        let stdout = "path:with:colons\x0042:matched content\n";
        let hits = parse_grep_output_with_pattern(
            stdout,
            ContainerRuntimeKind::Docker,
            "abc123",
            Some(GrepPattern {
                needle: "match",
                regex: false,
                case_sensitive: false,
            }),
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].container_path, "path:with:colons");
        assert_eq!(hits[0].line_number, 42);
        assert_eq!(hits[0].line_content, "matched content");
    }

    #[test]
    fn parse_grep_output_skips_malformed_lines() {
        let stdout = "no-colons-at-all\n:only-one-colon\nok:10:content\n";
        let hits = parse_grep_output(stdout, ContainerRuntimeKind::Docker, "abc123");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line_number, 10);
    }

    #[test]
    fn parse_grep_output_skips_non_numeric_lineno() {
        let stdout = "file:abc:content\n";
        let hits = parse_grep_output(stdout, ContainerRuntimeKind::Podman, "abc123");
        assert!(hits.is_empty());
    }

    #[test]
    fn prune_invalid_stamp_dirs_are_cleaned() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("container-mirrors");
        let container = root.join("docker").join("cid");

        let invalid = container.join("not-a-number");
        std::fs::create_dir_all(&invalid).unwrap();
        std::fs::write(invalid.join("f.txt"), "x").unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let stale = container.join((now - 7200).to_string());
        std::fs::create_dir_all(&stale).unwrap();

        prune_mirrors_under(&root, 3600).unwrap();

        assert!(!invalid.exists(), "invalid-stamp dir must be removed");
        assert!(!stale.exists(), "expired-stamp dir must be removed");
    }

    #[test]
    fn prune_keeps_fresh_stamp_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("container-mirrors");
        let container = root.join("podman").join("cid");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let fresh = container.join(now.to_string());
        std::fs::create_dir_all(&fresh).unwrap();
        std::fs::write(fresh.join("f.txt"), "x").unwrap();

        prune_mirrors_under(&root, 3600).unwrap();

        assert!(fresh.exists(), "fresh-stamp dir must survive");
        assert!(fresh.join("f.txt").exists());
    }

    #[test]
    fn prune_missing_root_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        prune_mirrors_under(&dir.path().join("does-not-exist"), 3600).unwrap();
    }
}
