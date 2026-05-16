use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub runtime: ContainerRuntimeKind,
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerSearchTarget {
    pub runtime: ContainerRuntimeKind,
    pub container_id: String,
    pub container_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerMirrorInfo {
    pub runtime: ContainerRuntimeKind,
    pub container_id: String,
    pub container_name: String,
    pub container_path: String,
    pub local_mirror_path: PathBuf,
    pub local_search_path: PathBuf,
    pub created_unix: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
