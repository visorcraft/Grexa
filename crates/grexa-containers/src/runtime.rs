// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Container runtime adapters.
//!
//! Two backends — `docker` and `podman` — share the same wire shape: their
//! CLIs accept `--format json` on `ps` and stream stdout from `exec`. The
//! [`RuntimeOperations`] trait represents the four operations Grexa drives
//! against any container runtime; tests use [`MockCommandRunner`] to inject
//! canned responses without touching a real daemon.
//!
//! Why CLI rather than the HTTP API? Three reasons:
//!
//! 1. The CLI handles socket discovery, rootless/rootful auth, and
//!    Docker-vs-Podman version skew already.
//! 2. The audit (`docs/grex-docker-search-service-audit.md`) accepts CLI
//!    fallback as a legitimate Tier-2 path.
//! 3. A sync HTTP-over-Unix-socket client would pull in `hyperlocal` or
//!    require ~250 lines of hand-rolled chunked-encoding parser; this layer
//!    only needs `list`, `exec`, and `archive`.
//!
//! A future spike can replace [`SystemCommandRunner`] with a `hyperlocal`
//! transport without disturbing the public surface — every callable
//! function on [`RuntimeOperations`] already returns a typed value rather
//! than a `Command` invocation.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ContainerInfo, ContainerRuntime, ContainerRuntimeKind};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime CLI {cli:?} exited with status {status}: {stderr}")]
    Cli {
        cli: PathBuf,
        status: i32,
        stderr: String,
    },
    #[error("runtime CLI for {kind:?} is not installed")]
    CliMissing { kind: ContainerRuntimeKind },
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported feature: {0}")]
    Unsupported(String),
}

/// One JSON record from `docker ps --format json` / `podman ps --format json`.
/// Field names are wire-level (PascalCase or shouted, depending on backend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerPsRow {
    // Docker emits the upper-case `ID` column; Podman emits PascalCase `Id`.
    #[serde(alias = "ID", alias = "Id")]
    id: String,
    #[serde(default)]
    names: NamesField,
    #[serde(default)]
    image: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    state: String,
}

/// Docker's `ps --format json` emits the `Names` column as `"Names": "a,b"`,
/// while Podman emits an array. `NamesField` adapts both shapes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
enum NamesField {
    Single(String),
    Many(Vec<String>),
    #[default]
    Empty,
}

impl NamesField {
    fn first(&self) -> String {
        match self {
            NamesField::Single(name) => name.trim_start_matches('/').to_string(),
            NamesField::Many(names) => names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default(),
            NamesField::Empty => String::new(),
        }
    }
}

/// Command invocation captured for mocking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInvocation {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub stdin: Vec<u8>,
}

/// Result of a captured `Command::output()`.
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CommandResult {
    pub fn success(stdout: &str) -> Self {
        Self {
            status: 0,
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    pub fn failure(status: i32, stderr: &str) -> Self {
        Self {
            status,
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }
}

/// Trait for executing CLI commands. Production uses
/// [`SystemCommandRunner`]; tests use [`MockCommandRunner`].
pub trait CommandRunner: Send + Sync {
    /// Run `invocation`, killing the child if it runs longer than `timeout`.
    /// Pass `Duration::MAX` to wait indefinitely (used by tests).
    fn run(&self, invocation: CommandInvocation, timeout: Duration) -> io::Result<CommandResult>;
}

/// Real-process runner with timeout enforcement.
pub struct SystemCommandRunner;

impl SystemCommandRunner {
    /// Default timeout for container runtime operations. Mirrors the
    /// historical constant logged by the search path.
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
    /// Maximum bytes captured from a child process's stdout or stderr.
    /// Exceeding this kills the process and returns an error, preventing a
    /// flood of output (e.g. an unbounded `grep`) from exhausting memory.
    const MAX_OUTPUT_BYTES: usize = 10 * 1024 * 1024;
}

impl CommandRunner for SystemCommandRunner {
    fn run(&self, invocation: CommandInvocation, timeout: Duration) -> io::Result<CommandResult> {
        use std::io::Write;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;

        let mut cmd = Command::new(&invocation.program);
        cmd.args(&invocation.args)
            .stdin(if invocation.stdin.is_empty() {
                Stdio::null()
            } else {
                Stdio::piped()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        if !invocation.stdin.is_empty()
            && let Some(mut handle) = child.stdin.take()
        {
            handle.write_all(&invocation.stdin)?;
        }

        let pid = child.id();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let cap_exceeded = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::channel();

        // Spawn the child watcher and the two bounded pipe readers.
        let tx_status = tx.clone();
        thread::spawn(move || {
            let _ = tx_status.send(Either::Left(child.wait()));
        });

        let stdout_cap = Arc::clone(&cap_exceeded);
        let tx_stdout = tx.clone();
        thread::spawn(move || {
            let _ = tx_stdout.send(Either::Right((
                "stdout",
                read_limited(stdout, Self::MAX_OUTPUT_BYTES, &stdout_cap),
            )));
        });

        let stderr_cap = Arc::clone(&cap_exceeded);
        thread::spawn(move || {
            let _ = tx.send(Either::Right((
                "stderr",
                read_limited(stderr, Self::MAX_OUTPUT_BYTES, &stderr_cap),
            )));
        });

        let start = Instant::now();
        let mut status: Option<std::process::ExitStatus> = None;
        let mut stdout_buf: Option<io::Result<Vec<u8>>> = None;
        let mut stderr_buf: Option<io::Result<Vec<u8>>> = None;

        while status.is_none() || stdout_buf.is_none() || stderr_buf.is_none() {
            let remaining = if timeout == Duration::MAX {
                Duration::from_millis(50)
            } else {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    break;
                }
                std::cmp::min(Duration::from_millis(50), timeout - elapsed)
            };

            match rx.recv_timeout(remaining) {
                Ok(Either::Left(Ok(s))) => status = Some(s),
                Ok(Either::Left(Err(err))) => {
                    return Err(err);
                }
                Ok(Either::Right(("stdout", buf))) => stdout_buf = Some(buf),
                Ok(Either::Right(("stderr", buf))) => stderr_buf = Some(buf),
                Ok(Either::Right((_, _))) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if cap_exceeded.load(Ordering::SeqCst) {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        // If any required piece is missing (timeout, cap exceeded, or channel
        // died), kill the child and return a corresponding error.
        if status.is_none() || stdout_buf.is_none() || stderr_buf.is_none() {
            // SAFETY: `libc::kill` with a non-negative pid and SIGKILL is
            // always valid to call; it returns an error for stale pids,
            // which we ignore.
            let _ = unsafe { libc::kill(pid as i32, libc::SIGKILL) };
            // Drain remaining channel messages so the helper threads can exit.
            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {}

            if cap_exceeded.load(Ordering::SeqCst) {
                return Err(io::Error::other("runtime command output exceeded size limit"));
            }
            return Err(io::Error::new(io::ErrorKind::TimedOut, "runtime command timed out"));
        }

        Ok(CommandResult {
            status: status.unwrap().code().unwrap_or(-1),
            stdout: stdout_buf.unwrap()?,
            stderr: stderr_buf.unwrap()?,
        })
    }
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

fn read_limited<R: Read + Send>(
    mut reader: R,
    max: usize,
    cap_flag: &AtomicBool,
) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(1024.min(max));
    let mut chunk = [0u8; 8192];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => return Ok(buf),
            Ok(n) => {
                if buf.len() + n > max {
                    cap_flag.store(true, Ordering::SeqCst);
                    return Err(io::Error::other("runtime command output exceeded size limit"));
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            Err(err) => return Err(err),
        }
    }
}

/// Mock runner used in tests: programs map onto a queue of canned results.
#[derive(Default, Clone)]
pub struct MockCommandRunner {
    inner: Arc<Mutex<MockState>>,
}

#[derive(Default)]
struct MockState {
    canned: Vec<CommandResult>,
    invocations: Vec<CommandInvocation>,
}

impl MockCommandRunner {
    pub fn push(&self, result: CommandResult) {
        self.inner.lock().unwrap().canned.push(result);
    }

    pub fn invocations(&self) -> Vec<CommandInvocation> {
        self.inner.lock().unwrap().invocations.clone()
    }
}

impl CommandRunner for MockCommandRunner {
    fn run(&self, invocation: CommandInvocation, _timeout: Duration) -> io::Result<CommandResult> {
        let mut state = self.inner.lock().unwrap();
        state.invocations.push(invocation.clone());
        if state.canned.is_empty() {
            return Err(io::Error::other(format!(
                "MockCommandRunner: no canned result for {invocation:?}"
            )));
        }
        Ok(state.canned.remove(0))
    }
}

type GrepAvailabilityCache = HashMap<(ContainerRuntimeKind, String), bool>;

static GREP_AVAILABILITY_CACHE: OnceLock<Mutex<GrepAvailabilityCache>> = OnceLock::new();

fn grep_availability_cache() -> &'static Mutex<GrepAvailabilityCache> {
    GREP_AVAILABILITY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Clear the per-container grep-availability cache. Exposed for tests and
/// long-lived GUI sessions that want to force a fresh probe.
pub fn clear_grep_availability_cache() {
    if let Ok(mut cache) = grep_availability_cache().lock() {
        cache.clear();
    }
}

/// Operations Grexa runs against a single container runtime.
pub trait RuntimeOperations {
    fn kind(&self) -> ContainerRuntimeKind;
    fn list_containers(&self) -> Result<Vec<ContainerInfo>, RuntimeError>;
    /// Like `list_containers`, but aborts after `timeout`.
    fn list_containers_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<ContainerInfo>, RuntimeError>;
    /// Run `argv` inside the named container and return the captured stdout.
    /// `argv` is an argv array, not a shell line — backends pass it directly
    /// to `exec` so quoting bugs around spaces / colons / globs / newlines
    /// can't appear.
    fn exec_capture(
        &self,
        container_id: &str,
        argv: &[&str],
    ) -> Result<CommandResult, RuntimeError>;
    /// Like `exec_capture`, but aborts after `timeout`.
    fn exec_capture_timeout(
        &self,
        container_id: &str,
        argv: &[&str],
        timeout: Duration,
    ) -> Result<CommandResult, RuntimeError>;
    /// Probe whether `grep` is callable inside the container.
    fn has_grep(&self, container_id: &str) -> Result<bool, RuntimeError>;
    /// Tar-archive a path inside the container into `dest_dir`. Returns the
    /// path of the materialized file (or directory root). Used by the
    /// mirror-fallback path when direct exec isn't possible.
    fn archive_path(
        &self,
        container_id: &str,
        path: &str,
        dest_dir: &Path,
    ) -> Result<PathBuf, RuntimeError>;
    /// Copy a local file or directory into the container at `container_path`.
    /// The inverse of `archive_path`.
    fn copy_into_container(
        &self,
        container_id: &str,
        local_path: &Path,
        container_path: &str,
    ) -> Result<(), RuntimeError>;
}

/// `docker` / `podman` CLI adapter. Identical wire shape for both runtimes.
pub struct CliRuntime<R: CommandRunner> {
    runtime: ContainerRuntime,
    runner: R,
}

impl<R: CommandRunner> CliRuntime<R> {
    pub fn new(runtime: ContainerRuntime, runner: R) -> Self {
        Self { runtime, runner }
    }

    pub fn runtime(&self) -> &ContainerRuntime {
        &self.runtime
    }

    fn cli_path(&self) -> Result<PathBuf, RuntimeError> {
        self.runtime
            .cli_path
            .clone()
            .ok_or(RuntimeError::CliMissing {
                kind: self.runtime.kind,
            })
    }

    fn invoke(
        &self,
        args: Vec<OsString>,
        timeout: Duration,
    ) -> Result<CommandResult, RuntimeError> {
        let program = self.cli_path()?;
        let result = self.runner.run(
            CommandInvocation {
                program: program.clone(),
                args: args.clone(),
                stdin: Vec::new(),
            },
            timeout,
        )?;
        if result.status != 0 {
            return Err(RuntimeError::Cli {
                cli: program,
                status: result.status,
                stderr: String::from_utf8_lossy(&result.stderr).into_owned(),
            });
        }
        Ok(result)
    }
}

impl<R: CommandRunner> RuntimeOperations for CliRuntime<R> {
    fn kind(&self) -> ContainerRuntimeKind {
        self.runtime.kind
    }

    fn list_containers(&self) -> Result<Vec<ContainerInfo>, RuntimeError> {
        self.list_containers_timeout(Duration::MAX)
    }

    fn list_containers_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<ContainerInfo>, RuntimeError> {
        let args = vec![
            OsString::from("ps"),
            OsString::from("--all"),
            OsString::from("--format=json"),
        ];
        let result = self.invoke(args, timeout)?;
        let stdout = String::from_utf8_lossy(&result.stdout);

        // Docker emits one JSON object per line. Podman emits a JSON array.
        // Try line-delimited first, fall back to array.
        let mut rows = Vec::new();
        let trimmed = stdout.trim();
        if trimmed.starts_with('[') {
            rows = serde_json::from_str::<Vec<DockerPsRow>>(trimmed)?;
        } else {
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                rows.push(serde_json::from_str::<DockerPsRow>(line)?);
            }
        }

        Ok(rows
            .into_iter()
            .map(|row| ContainerInfo {
                runtime: self.runtime.kind,
                id: row.id,
                name: row.names.first(),
                image: row.image,
                status: row.status,
                state: row.state,
            })
            .collect())
    }

    fn exec_capture(
        &self,
        container_id: &str,
        argv: &[&str],
    ) -> Result<CommandResult, RuntimeError> {
        self.exec_capture_timeout(container_id, argv, Duration::MAX)
    }

    fn exec_capture_timeout(
        &self,
        container_id: &str,
        argv: &[&str],
        timeout: Duration,
    ) -> Result<CommandResult, RuntimeError> {
        // `--` terminates option parsing so an untrusted container id (e.g.
        // `--user=root`, podman `--privileged`) can never be interpreted as a
        // flag to `exec`.
        let mut args = vec![
            OsString::from("exec"),
            OsString::from("--"),
            OsString::from(container_id),
        ];
        for arg in argv {
            args.push(OsString::from(*arg));
        }
        // exec is allowed to return non-zero — e.g. grep with no matches
        // exits 1. We surface the raw result.
        let program = self.cli_path()?;
        let result = self.runner.run(
            CommandInvocation {
                program,
                args,
                stdin: Vec::new(),
            },
            timeout,
        )?;
        Ok(result)
    }

    fn has_grep(&self, container_id: &str) -> Result<bool, RuntimeError> {
        let key = (self.kind(), container_id.to_string());
        if let Ok(cache) = grep_availability_cache().lock()
            && let Some(cached) = cache.get(&key)
        {
            return Ok(*cached);
        }

        // `which grep` is universally available across Linux containers.
        // Distroless containers may lack `which`; fall back to a probe via
        // exec returning a non-127 status.
        let result = self.exec_capture_timeout(
            container_id,
            &["which", "grep"],
            SystemCommandRunner::DEFAULT_TIMEOUT,
        )?;
        let has = result.status == 0 && !result.stdout.is_empty();

        if let Ok(mut cache) = grep_availability_cache().lock() {
            cache.insert(key, has);
        }
        Ok(has)
    }

    fn archive_path(
        &self,
        container_id: &str,
        path: &str,
        dest_dir: &Path,
    ) -> Result<PathBuf, RuntimeError> {
        std::fs::create_dir_all(dest_dir)?;
        let target = dest_dir.join(
            Path::new(path)
                .file_name()
                .map(|n| n.to_os_string())
                .unwrap_or_else(|| OsString::from("archive")),
        );
        let args = vec![
            OsString::from("cp"),
            OsString::from("--"),
            OsString::from(format!("{container_id}:{path}")),
            target.clone().into_os_string(),
        ];
        self.invoke(args, SystemCommandRunner::DEFAULT_TIMEOUT)?;
        Ok(target)
    }

    fn copy_into_container(
        &self,
        container_id: &str,
        local_path: &Path,
        container_path: &str,
    ) -> Result<(), RuntimeError> {
        let args = vec![
            OsString::from("cp"),
            OsString::from("--"),
            local_path.as_os_str().to_os_string(),
            OsString::from(format!("{container_id}:{container_path}")),
        ];
        self.invoke(args, SystemCommandRunner::DEFAULT_TIMEOUT)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_runtime() -> ContainerRuntime {
        ContainerRuntime {
            kind: ContainerRuntimeKind::Podman,
            socket_path: None,
            cli_path: Some(PathBuf::from("/usr/bin/podman")),
            rootless: true,
        }
    }

    #[test]
    fn list_containers_parses_podman_array() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success(
            r#"[
                {"Id":"abc","Names":["web"],"Image":"alpine","Status":"Up 5m","State":"running"},
                {"Id":"def","Names":["db"],"Image":"postgres","Status":"Exited","State":"exited"}
            ]"#,
        ));
        let runtime = CliRuntime::new(fake_runtime(), runner.clone());
        let containers = runtime.list_containers().unwrap();
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].name, "web");
        assert_eq!(containers[1].name, "db");

        let inv = runner.invocations();
        assert_eq!(inv[0].program, PathBuf::from("/usr/bin/podman"));
        assert_eq!(
            inv[0].args,
            vec![
                OsString::from("ps"),
                OsString::from("--all"),
                OsString::from("--format=json"),
            ]
        );
    }

    #[test]
    fn list_containers_parses_docker_line_delimited() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success(
            "{\"ID\":\"abc\",\"Names\":\"web\",\"Image\":\"alpine\",\"Status\":\"Up\",\"State\":\"running\"}\n",
        ));
        let runtime = CliRuntime::new(
            ContainerRuntime {
                kind: ContainerRuntimeKind::Docker,
                ..fake_runtime()
            },
            runner,
        );
        let containers = runtime.list_containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].name, "web");
        assert_eq!(containers[0].runtime, ContainerRuntimeKind::Docker);
    }

    #[test]
    fn list_containers_surfaces_cli_failure() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::failure(125, "permission denied"));
        let runtime = CliRuntime::new(fake_runtime(), runner);
        let err = runtime.list_containers().unwrap_err();
        match err {
            RuntimeError::Cli { status, stderr, .. } => {
                assert_eq!(status, 125);
                assert!(stderr.contains("permission denied"));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn exec_capture_passes_argv_array() {
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        let runtime = CliRuntime::new(fake_runtime(), runner.clone());
        runtime.exec_capture("abc", &["which", "grep"]).unwrap();
        let inv = runner.invocations();
        // `--` must terminate option parsing before the (untrusted) container
        // id so a value like `--user=root` can't be smuggled in as a flag.
        assert_eq!(
            inv[0].args,
            vec![
                OsString::from("exec"),
                OsString::from("--"),
                OsString::from("abc"),
                OsString::from("which"),
                OsString::from("grep"),
            ]
        );
    }

    #[test]
    fn has_grep_returns_true_when_which_succeeds() {
        clear_grep_availability_cache();
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        let runtime = CliRuntime::new(fake_runtime(), runner);
        assert!(runtime.has_grep("has-grep-true").unwrap());
    }

    #[test]
    fn has_grep_returns_false_when_which_returns_nothing() {
        clear_grep_availability_cache();
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success(""));
        let runtime = CliRuntime::new(fake_runtime(), runner);
        assert!(!runtime.has_grep("has-grep-false").unwrap());
    }

    #[test]
    fn has_grep_caches_result_across_calls() {
        clear_grep_availability_cache();
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success("/usr/bin/grep\n"));
        let runtime = CliRuntime::new(fake_runtime(), runner.clone());

        let id = "cached-grep";
        assert!(runtime.has_grep(id).unwrap());
        assert!(runtime.has_grep(id).unwrap());
        // Only one `which grep` exec should have been issued.
        assert_eq!(
            runner
                .invocations()
                .iter()
                .filter(|inv| inv.args.contains(&OsString::from("which")))
                .count(),
            1
        );
    }

    #[test]
    fn archive_path_invokes_cp() {
        let dir = tempfile::tempdir().unwrap();
        let runner = MockCommandRunner::default();
        runner.push(CommandResult::success(""));
        let runtime = CliRuntime::new(fake_runtime(), runner.clone());
        let mirror = runtime
            .archive_path("abc", "/etc/hostname", dir.path())
            .unwrap();
        assert_eq!(mirror, dir.path().join("hostname"));
        let inv = runner.invocations();
        assert_eq!(
            inv[0].args,
            vec![
                OsString::from("cp"),
                OsString::from("--"),
                OsString::from("abc:/etc/hostname"),
                dir.path().join("hostname").into_os_string(),
            ]
        );
    }

    #[test]
    fn system_runner_enforces_timeout() {
        let invocation = CommandInvocation {
            program: PathBuf::from("sleep"),
            args: vec![OsString::from("10")],
            stdin: Vec::new(),
        };
        let err = SystemCommandRunner
            .run(invocation, Duration::from_millis(50))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn system_runner_enforces_output_size_cap() {
        let invocation = CommandInvocation {
            program: PathBuf::from("cat"),
            args: vec![OsString::from("/dev/zero")],
            stdin: Vec::new(),
        };
        let err = SystemCommandRunner
            .run(invocation, Duration::from_secs(5))
            .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert!(err.to_string().contains("size limit"), "expected size-limit error, got {err}");
    }
}
