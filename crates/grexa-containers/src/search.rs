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

/// Single grep hit produced by either backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerSearchHit {
    pub runtime: ContainerRuntimeKind,
    pub container_id: String,
    /// Path *inside the container*, even when produced by the mirror
    /// fallback.
    pub container_path: String,
    pub line_number: usize,
    pub line_content: String,
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
}

impl ContainerSearchOptions {
    pub fn new(path: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            container_path: path.into(),
            pattern: pattern.into(),
            case_sensitive: false,
            regex: false,
        }
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

    let (hits, used_mirror) = if has_grep {
        (direct_grep(runtime, container, options)?, false)
    } else {
        (mirror_search(runtime, container, options)?, true)
    };

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
    let mut argv: Vec<&str> = vec!["grep", "-rnH"];
    if options.regex {
        argv.push("-E");
    } else {
        argv.push("-F");
    }
    if !options.case_sensitive {
        argv.push("-i");
    }
    argv.push("--");
    argv.push(&options.pattern);
    argv.push(&options.container_path);

    let result = runtime.exec_capture(&container.id, &argv)?;
    // grep exits 1 when nothing matched — that's a successful empty search.
    if result.status != 0 && result.status != 1 {
        return Err(RuntimeError::Cli {
            cli: PathBuf::from(format!("{:?}", runtime.kind())),
            status: result.status,
            stderr: String::from_utf8_lossy(&result.stderr).into_owned(),
        });
    }

    Ok(parse_grep_output(
        &String::from_utf8_lossy(&result.stdout),
        runtime.kind(),
        &container.id,
    ))
}

/// Parse colon-delimited `grep -rnH` output. The first colon separates the
/// file path from the line number; the second separates line number from
/// content. Both halves may themselves contain colons (Windows-style paths
/// don't show up in containers, but Unix paths with `:` are rare yet legal).
/// We split greedily on the first two colons only.
pub fn parse_grep_output(
    stdout: &str,
    kind: ContainerRuntimeKind,
    container_id: &str,
) -> Vec<ContainerSearchHit> {
    let mut hits = Vec::new();
    for line in stdout.lines() {
        // Need exactly two colons to split into (path, lineno, content).
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
        let path = &line[..first];
        let lineno_str = &line[first + 1..second];
        let content = &line[second + 1..];
        let Ok(line_number) = lineno_str.parse::<usize>() else {
            continue;
        };
        hits.push(ContainerSearchHit {
            runtime: kind,
            container_id: container_id.to_string(),
            container_path: path.to_string(),
            line_number,
            line_content: content.to_string(),
        });
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
    let root = paths.cache_dir.join("container-mirrors");
    if !root.exists() {
        return Ok(());
    }
    let cutoff = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().saturating_sub(max_age_secs))
        .unwrap_or_default();

    for runtime_entry in std::fs::read_dir(&root)? {
        let runtime_entry = runtime_entry?;
        for container_entry in std::fs::read_dir(runtime_entry.path())? {
            let container_entry = container_entry?;
            for stamp_entry in std::fs::read_dir(container_entry.path())? {
                let stamp_entry = stamp_entry?;
                let stamp_name = stamp_entry.file_name();
                let stamp: u64 = stamp_name
                    .to_string_lossy()
                    .parse()
                    .unwrap_or(u64::MAX);
                if stamp < cutoff {
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
        assert_eq!(hits[1].line_content, "127.0.0.1 localhost");
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
        runner.push(CommandResult::success(
            "/etc/hostname:1:my-host\n/etc/hosts:5:another match\n",
        ));
        let runtime = cli_runtime(runner.clone());

        let opts = ContainerSearchOptions::new("/etc", "host");
        let summary = search_container(&runtime, &fake_container(), &opts).unwrap();
        assert!(!summary.used_mirror);
        assert_eq!(summary.hits.len(), 2);

        let inv = runner.invocations();
        // First invocation: has_grep probe via `which grep`.
        assert_eq!(inv[0].args[2], OsString::from("which"));
        // Second invocation: the actual grep call.
        let grep_args = &inv[1].args;
        assert!(grep_args.iter().any(|a| a == &OsString::from("grep")));
        assert!(grep_args.iter().any(|a| a == &OsString::from("-rnH")));
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
        assert_eq!(
            rewrite_path(Path::new("/tmp/cache/mirror/abc/123"), local, "/data"),
            "/data"
        );
        assert_eq!(
            rewrite_path(Path::new("/tmp/cache/mirror/abc/123/a/b"), local, "/data/"),
            "/data/a/b"
        );
    }
}
