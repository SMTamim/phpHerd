use crate::core::config::AppConfig;
use std::path::PathBuf;

pub struct NodeManager;

impl NodeManager {
    pub fn node_dir() -> PathBuf {
        AppConfig::data_dir().join("node")
    }

    pub fn node_version_dir(version: &str) -> PathBuf {
        Self::node_dir().join(version)
    }

    pub fn node_binary_path(version: &str) -> PathBuf {
        let dir = Self::node_version_dir(version);
        if cfg!(target_os = "windows") {
            dir.join("node.exe")
        } else {
            dir.join("bin").join("node")
        }
    }

    pub fn is_installed(version: &str) -> bool {
        Self::node_binary_path(version).exists()
    }

    pub fn installed_versions() -> Vec<String> {
        let node_dir = Self::node_dir();
        if !node_dir.exists() {
            return Vec::new();
        }

        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&node_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if Self::node_binary_path(&name).exists() {
                        versions.push(name);
                    }
                }
            }
        }
        versions.sort();
        versions
    }
}
