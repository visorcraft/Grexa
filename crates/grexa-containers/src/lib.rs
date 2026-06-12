// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub mod runtime;
pub mod search;

pub use runtime::{
    CliRuntime, CommandInvocation, CommandResult, CommandRunner, MockCommandRunner, RuntimeError,
    RuntimeOperations, SystemCommandRunner,
};

pub use search::{
    ContainerReplaceOptions, ContainerReplaceSummary, ContainerSearchHit, ContainerSearchOptions,
    ContainerSearchSummary, GrepPattern, container_context_preview, parse_grep_output,
    parse_grep_output_with_pattern, prune_mirrors, replace_container, search_container,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerRuntimeKind {
    Docker,
    Podman,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerRuntime {
    pub kind: ContainerRuntimeKind,
    pub socket_path: Option<PathBuf>,
    pub cli_path: Option<PathBuf>,
    pub rootless: bool,
}

impl ContainerRuntime {
    pub fn is_available(&self) -> bool {
        self.socket_path.is_some() || self.cli_path.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub runtime: ContainerRuntimeKind,
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
}

/// Filesystem probe abstraction so detection can run against fixtures.
pub trait FsProbe: Send + Sync {
    fn env_var(&self, key: &str) -> Option<String>;
    fn path_exists(&self, path: &Path) -> bool;
    fn find_on_path(&self, name: &str) -> Option<PathBuf>;
}

/// Default probe that hits the live system.
pub struct LiveProbe;

impl FsProbe for LiveProbe {
    fn env_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn find_on_path(&self, name: &str) -> Option<PathBuf> {
        let path_env = env::var_os("PATH")?;
        for dir in env::split_paths(&path_env) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}

/// Detect every available container runtime. Returns one entry per detected
/// runtime; the ordering is Docker → rootless Podman → rootful Podman, which
/// matches the precedence the GUI uses to default the target selector.
pub fn detect_runtimes(probe: &dyn FsProbe) -> Vec<ContainerRuntime> {
    let mut runtimes = Vec::new();
    if let Some(docker) = detect_docker(probe) {
        runtimes.push(docker);
    }
    if let Some(rootless) = detect_podman_rootless(probe) {
        runtimes.push(rootless);
    }
    if let Some(rootful) = detect_podman_rootful(probe) {
        runtimes.push(rootful);
    }
    runtimes
}

fn detect_docker(probe: &dyn FsProbe) -> Option<ContainerRuntime> {
    let cli_path = probe.find_on_path("docker");
    let socket_path = docker_socket_from_env(probe).or_else(|| {
        let default = PathBuf::from("/var/run/docker.sock");
        probe.path_exists(&default).then_some(default)
    });

    if socket_path.is_none() && cli_path.is_none() {
        return None;
    }
    Some(ContainerRuntime {
        kind: ContainerRuntimeKind::Docker,
        socket_path,
        cli_path,
        rootless: false,
    })
}

fn docker_socket_from_env(probe: &dyn FsProbe) -> Option<PathBuf> {
    let host = probe.env_var("DOCKER_HOST")?;
    // Linux Docker sockets are typically expressed as unix:///var/run/docker.sock.
    let stripped = host
        .trim()
        .strip_prefix("unix://")
        .or_else(|| host.trim().strip_prefix("unix:"))?;
    let path = PathBuf::from(stripped);
    probe.path_exists(&path).then_some(path)
}

fn detect_podman_rootless(probe: &dyn FsProbe) -> Option<ContainerRuntime> {
    let runtime_dir = probe.env_var("XDG_RUNTIME_DIR")?;
    let socket_path = PathBuf::from(runtime_dir)
        .join("podman")
        .join("podman.sock");
    if !probe.path_exists(&socket_path) {
        return None;
    }
    Some(ContainerRuntime {
        kind: ContainerRuntimeKind::Podman,
        socket_path: Some(socket_path),
        cli_path: probe.find_on_path("podman"),
        rootless: true,
    })
}

fn detect_podman_rootful(probe: &dyn FsProbe) -> Option<ContainerRuntime> {
    let socket_path = PathBuf::from("/run/podman/podman.sock");
    let socket_present = probe.path_exists(&socket_path);
    let cli_path = probe.find_on_path("podman");

    if !socket_present && cli_path.is_none() {
        return None;
    }
    if !socket_present {
        // CLI-only access is enough for `podman ps`; flag the rootful flavor
        // so the runtime badge in the GUI reflects what the user actually has.
        return Some(ContainerRuntime {
            kind: ContainerRuntimeKind::Podman,
            socket_path: None,
            cli_path,
            rootless: false,
        });
    }
    Some(ContainerRuntime {
        kind: ContainerRuntimeKind::Podman,
        socket_path: Some(socket_path),
        cli_path,
        rootless: false,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    #[derive(Default)]
    struct FakeProbe {
        env: HashMap<String, String>,
        existing_paths: HashSet<PathBuf>,
        path_binaries: HashMap<String, PathBuf>,
    }

    impl FakeProbe {
        fn with_env(mut self, key: &str, value: &str) -> Self {
            self.env.insert(key.to_string(), value.to_string());
            self
        }

        fn with_path(mut self, path: &str) -> Self {
            self.existing_paths.insert(PathBuf::from(path));
            self
        }

        fn with_binary(mut self, name: &str, path: &str) -> Self {
            self.path_binaries
                .insert(name.to_string(), PathBuf::from(path));
            self
        }
    }

    impl FsProbe for FakeProbe {
        fn env_var(&self, key: &str) -> Option<String> {
            self.env.get(key).cloned()
        }

        fn path_exists(&self, path: &Path) -> bool {
            self.existing_paths.contains(path)
        }

        fn find_on_path(&self, name: &str) -> Option<PathBuf> {
            self.path_binaries.get(name).cloned()
        }
    }

    #[test]
    fn records_runtime_kind_on_container_info() {
        let container = ContainerInfo {
            runtime: ContainerRuntimeKind::Podman,
            id: "abc123".to_string(),
            name: "web".to_string(),
            image: "alpine".to_string(),
            status: "Up".to_string(),
            state: "running".to_string(),
        };
        assert_eq!(container.runtime, ContainerRuntimeKind::Podman);
    }

    #[test]
    fn detects_docker_socket_at_default_path() {
        let probe = FakeProbe::default().with_path("/var/run/docker.sock");
        let runtimes = detect_runtimes(&probe);
        assert!(runtimes.iter().any(|runtime| {
            runtime.kind == ContainerRuntimeKind::Docker
                && runtime.socket_path == Some(PathBuf::from("/var/run/docker.sock"))
        }));
    }

    #[test]
    fn docker_host_env_var_overrides_default_socket() {
        let probe = FakeProbe::default()
            .with_env("DOCKER_HOST", "unix:///run/user/1000/docker.sock")
            .with_path("/run/user/1000/docker.sock");
        let runtimes = detect_runtimes(&probe);
        let docker = runtimes
            .iter()
            .find(|runtime| runtime.kind == ContainerRuntimeKind::Docker)
            .expect("docker detected");
        assert_eq!(docker.socket_path, Some(PathBuf::from("/run/user/1000/docker.sock")));
    }

    #[test]
    fn docker_cli_alone_is_enough() {
        let probe = FakeProbe::default().with_binary("docker", "/usr/bin/docker");
        let runtimes = detect_runtimes(&probe);
        let docker = runtimes
            .iter()
            .find(|runtime| runtime.kind == ContainerRuntimeKind::Docker)
            .expect("docker detected");
        assert_eq!(docker.cli_path, Some(PathBuf::from("/usr/bin/docker")));
        assert!(docker.socket_path.is_none());
    }

    #[test]
    fn detects_rootless_podman_via_xdg_runtime_dir() {
        let probe = FakeProbe::default()
            .with_env("XDG_RUNTIME_DIR", "/run/user/1000")
            .with_path("/run/user/1000/podman/podman.sock")
            .with_binary("podman", "/usr/bin/podman");
        let runtimes = detect_runtimes(&probe);
        let rootless = runtimes
            .iter()
            .find(|runtime| runtime.kind == ContainerRuntimeKind::Podman && runtime.rootless)
            .expect("rootless podman detected");
        assert_eq!(rootless.socket_path, Some(PathBuf::from("/run/user/1000/podman/podman.sock")));
        assert!(rootless.cli_path.is_some());
    }

    #[test]
    fn detects_rootful_podman_via_run_socket() {
        let probe = FakeProbe::default().with_path("/run/podman/podman.sock");
        let runtimes = detect_runtimes(&probe);
        let rootful = runtimes
            .iter()
            .find(|runtime| runtime.kind == ContainerRuntimeKind::Podman && !runtime.rootless)
            .expect("rootful podman detected");
        assert_eq!(rootful.socket_path, Some(PathBuf::from("/run/podman/podman.sock")));
    }

    #[test]
    fn cli_only_podman_is_still_reported_as_rootful() {
        let probe = FakeProbe::default().with_binary("podman", "/usr/bin/podman");
        let runtimes = detect_runtimes(&probe);
        let rootful = runtimes
            .iter()
            .find(|runtime| runtime.kind == ContainerRuntimeKind::Podman && !runtime.rootless)
            .expect("cli-only podman detected");
        assert!(rootful.socket_path.is_none());
        assert_eq!(rootful.cli_path, Some(PathBuf::from("/usr/bin/podman")));
    }

    #[test]
    fn empty_environment_returns_no_runtimes() {
        let probe = FakeProbe::default();
        assert!(detect_runtimes(&probe).is_empty());
    }
}
