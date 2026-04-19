use crate::core::config::AppConfig;
use anyhow::Result;
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

    pub fn npm_binary_path(version: &str) -> PathBuf {
        let dir = Self::node_version_dir(version);
        if cfg!(target_os = "windows") {
            dir.join("npm.cmd")
        } else {
            dir.join("bin").join("npm")
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

    /// Get the Node.js version string
    pub fn get_version_string(version: &str) -> Option<String> {
        let binary = Self::node_binary_path(version);
        if !binary.exists() {
            return None;
        }
        let output = std::process::Command::new(&binary)
            .arg("--version")
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(stdout.trim().to_string())
    }

    /// Resolve download URL for a Node.js version
    fn resolve_download_url(version: &str) -> String {
        // version like "22" or "20" -> latest LTS-style
        // We use the direct download from nodejs.org
        let full_version = match version {
            "18" => "18.20.8",
            "20" => "20.19.0",
            "22" => "22.15.0",
            "23" => "23.11.1",
            "24" => "24.0.1",
            _ => version,
        };

        #[cfg(target_os = "windows")]
        {
            format!(
                "https://nodejs.org/dist/v{}/node-v{}-win-x64.zip",
                full_version, full_version
            )
        }

        #[cfg(target_os = "macos")]
        {
            let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" };
            format!(
                "https://nodejs.org/dist/v{}/node-v{}-darwin-{}.tar.gz",
                full_version, full_version, arch
            )
        }

        #[cfg(target_os = "linux")]
        {
            let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" };
            format!(
                "https://nodejs.org/dist/v{}/node-v{}-linux-{}.tar.gz",
                full_version, full_version, arch
            )
        }
    }

    /// Download and install a Node.js version
    pub async fn install_version(
        version: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        use futures_util::StreamExt;
        use std::io::Write;
        use tauri::Emitter;

        let target_dir = Self::node_version_dir(version);
        let url = Self::resolve_download_url(version);

        tracing::info!("Downloading Node.js {} from {}", version, url);

        let _ = app_handle.emit("node-install-progress", serde_json::json!({
            "version": version,
            "stage": "downloading",
            "progress": 0,
            "message": format!("Downloading Node.js {}...", version),
        }));

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download Node.js {}: HTTP {}", version, response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let temp_dir = std::env::temp_dir();
        let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
        let temp_file_path = temp_dir.join(format!("node-{}.{}", version, ext));
        let mut temp_file = std::fs::File::create(&temp_file_path)?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            temp_file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                let _ = app_handle.emit("node-install-progress", serde_json::json!({
                    "version": version,
                    "stage": "downloading",
                    "progress": progress,
                    "message": format!("Downloading... {}%", progress),
                }));
            }
        }
        drop(temp_file);

        tracing::info!("Downloaded {} bytes", downloaded);

        let _ = app_handle.emit("node-install-progress", serde_json::json!({
            "version": version,
            "stage": "extracting",
            "progress": 100,
            "message": "Extracting...",
        }));

        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        std::fs::create_dir_all(&target_dir)?;

        #[cfg(target_os = "windows")]
        {
            let file = std::fs::File::open(&temp_file_path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            // Node zips have "node-v22.15.0-win-x64/" top-level dir — strip it
            let common_prefix = {
                let first = archive.by_index(0)?.name().to_string();
                let prefix = first.split('/').next().unwrap_or("").to_string();
                if !prefix.is_empty() { Some(format!("{}/", prefix)) } else { None }
            };

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let raw_name = entry.name().to_string();

                let relative = if let Some(ref prefix) = common_prefix {
                    if let Some(stripped) = raw_name.strip_prefix(prefix) {
                        stripped.to_string()
                    } else if raw_name.trim_end_matches('/') == prefix.trim_end_matches('/') {
                        continue;
                    } else {
                        raw_name.clone()
                    }
                } else {
                    raw_name.clone()
                };

                if relative.is_empty() {
                    continue;
                }

                let out_path = target_dir.join(&relative);
                if entry.is_dir() {
                    std::fs::create_dir_all(&out_path)?;
                } else {
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = std::fs::File::create(&out_path)?;
                    std::io::copy(&mut entry, &mut outfile)?;
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let file = std::fs::File::open(&temp_file_path)?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&target_dir)?;
        }

        let _ = std::fs::remove_file(&temp_file_path);

        let _ = app_handle.emit("node-install-progress", serde_json::json!({
            "version": version,
            "stage": "complete",
            "progress": 100,
            "message": format!("Node.js {} installed!", version),
        }));

        tracing::info!("Node.js {} installed to {:?}", version, target_dir);
        Ok(())
    }

    /// Update the global shim to point to a specific Node version
    pub fn update_global_shim(version: &str) -> Result<()> {
        let bin_dir = AppConfig::data_dir().join("bin");
        std::fs::create_dir_all(&bin_dir)?;

        let node_binary = Self::node_binary_path(version);
        if !node_binary.exists() {
            anyhow::bail!("Node.js {} binary not found", version);
        }

        #[cfg(target_os = "windows")]
        {
            // node.cmd shim
            let shim = bin_dir.join("node.cmd");
            std::fs::write(&shim, format!("@echo off\r\n\"{}\" %*\r\n", node_binary.to_string_lossy()))?;

            // npm.cmd shim
            let npm = Self::npm_binary_path(version);
            if npm.exists() {
                let npm_shim = bin_dir.join("npm.cmd");
                std::fs::write(&npm_shim, format!("@echo off\r\n\"{}\" %*\r\n", npm.to_string_lossy()))?;
            }

            // npx.cmd shim
            let npx = Self::node_version_dir(version).join("npx.cmd");
            if npx.exists() {
                let npx_shim = bin_dir.join("npx.cmd");
                std::fs::write(&npx_shim, format!("@echo off\r\n\"{}\" %*\r\n", npx.to_string_lossy()))?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::symlink;
            let shim = bin_dir.join("node");
            let _ = std::fs::remove_file(&shim);
            symlink(&node_binary, &shim)?;
        }

        tracing::info!("Updated global Node shim to version {}", version);
        Ok(())
    }

    /// Uninstall a Node.js version
    pub fn uninstall_version(version: &str) -> Result<()> {
        let dir = Self::node_version_dir(version);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
            tracing::info!("Uninstalled Node.js {}", version);
        }
        Ok(())
    }
}
