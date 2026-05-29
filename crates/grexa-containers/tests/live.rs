// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Live-daemon integration tests for grexa-containers.
//!
//! Gated behind the `container-live` Cargo feature so CI without a
//! reachable Docker/Podman daemon stays green.
//!
//! Run with:
//!
//!   cargo test -p grexa-containers --features container-live -- live::
//!
//! Each test:
//!   1. Auto-detects a usable CLI-backed runtime.
//!   2. Skips itself (returns `Ok`) when no runtime is reachable, so
//!      the file remains useful as documentation even when the
//!      feature is on but the local box can't run containers.
//!   3. Cleans up its container on the way out (`podman rm -f`).

#![cfg(feature = "container-live")]

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use grexa_containers::{
    CliRuntime, ContainerInfo, ContainerRuntime, ContainerRuntimeKind, ContainerSearchOptions,
    LiveProbe, RuntimeOperations, SystemCommandRunner, detect_runtimes, search_container,
};

fn pick_runtime() -> Option<ContainerRuntime> {
    let probe = LiveProbe;
    detect_runtimes(&probe).into_iter().find(|runtime| {
        if runtime.cli_path.is_none() {
            eprintln!("live: {:?} detected without CLI; skipping", runtime.kind);
            return false;
        }
        let cli = CliRuntime::new(runtime.clone(), SystemCommandRunner);
        match cli.list_containers() {
            Ok(_) => true,
            Err(err) => {
                eprintln!("live: {:?} CLI is not usable: {err}; skipping", runtime.kind);
                false
            }
        }
    })
}

fn unique_name(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or_default();
    let pid = std::process::id();
    format!("{prefix}_{pid}_{ts}")
}

fn spawn_alpine_with_todo(
    runtime: &ContainerRuntime,
    container_name: &str,
) -> Option<ContainerInfo> {
    let cli = runtime.cli_path.clone()?;
    // `<cli> run -d --rm --name <name> alpine sh -c 'echo "TODO ship it" > /tmp/notes && sleep 60'`
    let mut cmd = std::process::Command::new(&cli);
    cmd.args([
        "run",
        "-d",
        "--rm",
        "--name",
        container_name,
        "alpine",
        "sh",
        "-c",
        "echo 'TODO ship it' > /tmp/notes && sleep 60",
    ]);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        eprintln!("live: failed to start container: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    for _ in 0..50 {
        let ready = std::process::Command::new(&cli)
            .args(["exec", &id, "test", "-f", "/tmp/notes"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if ready {
            return Some(ContainerInfo {
                runtime: runtime.kind,
                id,
                name: container_name.to_string(),
                image: "alpine".to_string(),
                status: "Up".to_string(),
                state: "running".to_string(),
            });
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("live: alpine container did not create /tmp/notes before timeout; skipping");
    cleanup(runtime, &id);
    None
}

fn cleanup(runtime: &ContainerRuntime, container_id: &str) {
    if let Some(cli) = runtime.cli_path.as_ref() {
        let _ = std::process::Command::new(cli)
            .args(["rm", "-f", container_id])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

#[test]
fn detect_runtimes_finds_at_least_one() {
    let Some(runtime) = pick_runtime() else {
        eprintln!("live: no runtime detected; skipping");
        return;
    };
    assert!(
        matches!(runtime.kind, ContainerRuntimeKind::Docker | ContainerRuntimeKind::Podman),
        "unexpected runtime kind: {:?}",
        runtime.kind
    );
}

#[test]
fn list_containers_against_live_daemon() {
    let Some(runtime) = pick_runtime() else {
        eprintln!("live: no runtime detected; skipping");
        return;
    };
    let cli = CliRuntime::new(runtime, SystemCommandRunner);
    // The list call must succeed even when there are zero containers.
    let containers = cli
        .list_containers()
        .expect("list_containers against a live runtime should succeed");
    // Don't assume anything about the user's existing containers; just
    // confirm the call returned a well-formed vector.
    for container in containers {
        assert!(!container.id.is_empty());
    }
}

#[test]
fn direct_grep_against_live_alpine() {
    let Some(runtime) = pick_runtime() else {
        eprintln!("live: no runtime detected; skipping");
        return;
    };
    let name = unique_name("grexa_test");
    let Some(container) = spawn_alpine_with_todo(&runtime, &name) else {
        eprintln!("live: could not start alpine container; skipping");
        return;
    };
    let kind = runtime.kind;
    let cli_runtime = CliRuntime::new(runtime.clone(), SystemCommandRunner);

    let summary =
        search_container(&cli_runtime, &container, &ContainerSearchOptions::new("/tmp", "TODO"));

    cleanup(&runtime, &container.id);

    let summary = summary.expect("search_container should succeed");
    assert!(!summary.used_mirror, "alpine has grep; mirror should not fire");
    assert!(summary.hits.iter().any(|h| h.line_content.contains("TODO")));
    assert!(
        summary
            .hits
            .iter()
            .any(|h| h.container_path == "/tmp/notes")
    );
    let _ = kind;
}

#[test]
fn archive_path_round_trips_via_live_daemon() {
    let Some(runtime) = pick_runtime() else {
        eprintln!("live: no runtime detected; skipping");
        return;
    };
    let name = unique_name("grexa_archive");
    let Some(container) = spawn_alpine_with_todo(&runtime, &name) else {
        eprintln!("live: could not start alpine container; skipping");
        return;
    };
    let cli_runtime = CliRuntime::new(runtime.clone(), SystemCommandRunner);

    let dest = std::env::temp_dir().join(format!("grexa-live-{name}"));
    let result = cli_runtime.archive_path(&container.id, "/tmp/notes", &dest);

    cleanup(&runtime, &container.id);

    let target = result.expect("archive_path against live runtime");
    let body = std::fs::read_to_string(&target).expect("archived file should be readable");
    assert!(body.contains("TODO"));
    let _: PathBuf = target;
    let _ = std::fs::remove_dir_all(&dest);
}
