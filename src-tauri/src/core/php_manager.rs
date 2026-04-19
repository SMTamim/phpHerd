use crate::core::config::AppConfig;
use anyhow::Result;
use std::io::Write;
use std::path::PathBuf;

pub struct PhpManager;

impl PhpManager {
    /// Get the directory where PHP versions are stored
    pub fn php_dir() -> PathBuf {
        AppConfig::data_dir().join("php")
    }

    /// Get the path to a specific PHP version's directory
    pub fn version_dir(version: &str) -> PathBuf {
        Self::php_dir().join(version)
    }

    /// Get the path to a specific PHP version's binary
    pub fn php_binary_path(version: &str) -> PathBuf {
        let dir = Self::version_dir(version);
        if cfg!(target_os = "windows") {
            dir.join("php.exe")
        } else {
            dir.join("bin").join("php")
        }
    }

    /// Check if a PHP version is installed (binary exists)
    pub fn is_installed(version: &str) -> bool {
        Self::php_binary_path(version).exists()
    }

    /// Get list of installed PHP versions
    pub fn installed_versions() -> Vec<String> {
        let php_dir = Self::php_dir();
        if !php_dir.exists() {
            return Vec::new();
        }

        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&php_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if Self::php_binary_path(&name).exists() {
                        versions.push(name);
                    }
                }
            }
        }
        versions.sort();
        versions
    }

    /// Resolve the exact download URL and full version for a given minor version
    /// Returns (download_url, full_version_string)
    pub fn resolve_download_url(minor_version: &str) -> (String, String) {
        // Map minor versions to latest known patch releases and their Windows download URLs
        // Windows PHP downloads come from windows.php.net as zip archives (NTS = Non-Thread-Safe)
        let (full_ver, _arch_suffix) = match minor_version {
            "7.4" => ("7.4.33", "Win32-vc15-x64"),
            "8.0" => ("8.0.30", "Win32-vs16-x64"),
            "8.1" => ("8.1.32", "Win32-vs16-x64"),
            "8.2" => ("8.2.28", "Win32-vs16-x64"),
            "8.3" => ("8.3.20", "Win32-vs16-x64"),
            "8.4" => ("8.4.8", "Win32-vs16-x64"),
            _ => (minor_version, "Win32-vs16-x64"),
        };

        #[cfg(target_os = "windows")]
        {
            let url = format!(
                "https://windows.php.net/downloads/releases/php-{}-nts-{}.zip",
                full_ver, _arch_suffix
            );
            (url, full_ver.to_string())
        }

        #[cfg(target_os = "macos")]
        {
            let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" };
            let url = format!(
                "https://github.com/shivammathur/php-builder/releases/download/php-{}/php-{}-darwin-{}.tar.gz",
                full_ver, full_ver, arch
            );
            (url, full_ver.to_string())
        }

        #[cfg(target_os = "linux")]
        {
            let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" };
            let url = format!(
                "https://github.com/shivammathur/php-builder/releases/download/php-{}/php-{}-linux-{}.tar.gz",
                full_ver, full_ver, arch
            );
            (url, full_ver.to_string())
        }
    }

    /// Download and install a PHP version.
    /// Emits progress events to the given Tauri app handle.
    pub async fn install_version(
        minor_version: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        use tauri::Emitter;

        let target_dir = Self::version_dir(minor_version);
        let (url, full_ver) = Self::resolve_download_url(minor_version);

        tracing::info!("Downloading PHP {} ({}) from {}", minor_version, full_ver, url);

        // Emit: starting download
        let _ = app_handle.emit("php-install-progress", serde_json::json!({
            "version": minor_version,
            "stage": "downloading",
            "progress": 0,
            "message": format!("Downloading PHP {}...", full_ver),
        }));

        // Download the archive
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Download failed: {}. URL: {}", e, url))?;

        if !response.status().is_success() {
            // Try archive URL for older versions
            let archive_url = url.replace("/releases/", "/releases/archives/");
            tracing::info!("Primary URL failed ({}), trying archive: {}", response.status(), archive_url);

            let response = client.get(&archive_url).send().await
                .map_err(|e| anyhow::anyhow!("Archive download also failed: {}", e))?;

            if !response.status().is_success() {
                anyhow::bail!(
                    "Failed to download PHP {}: HTTP {} from both {} and {}",
                    minor_version, response.status(), url, archive_url
                );
            }

            return Self::download_and_extract(response, minor_version, &target_dir, app_handle).await;
        }

        Self::download_and_extract(response, minor_version, &target_dir, app_handle).await
    }

    async fn download_and_extract(
        response: reqwest::Response,
        minor_version: &str,
        target_dir: &std::path::Path,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        use tauri::Emitter;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        // Stream the download to a temp file
        let temp_dir = std::env::temp_dir();
        let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
        let temp_file_path = temp_dir.join(format!("php-{}.{}", minor_version, ext));
        let mut temp_file = std::fs::File::create(&temp_file_path)?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("Download stream error: {}", e))?;
            temp_file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                let _ = app_handle.emit("php-install-progress", serde_json::json!({
                    "version": minor_version,
                    "stage": "downloading",
                    "progress": progress,
                    "message": format!("Downloading... {}%", progress),
                }));
            }
        }
        drop(temp_file);

        tracing::info!("Downloaded {} bytes to {:?}", downloaded, temp_file_path);

        // Emit: extracting
        let _ = app_handle.emit("php-install-progress", serde_json::json!({
            "version": minor_version,
            "stage": "extracting",
            "progress": 100,
            "message": "Extracting...",
        }));

        // Clean target dir if it exists (partial previous install)
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)?;
        }
        std::fs::create_dir_all(target_dir)?;

        // Extract
        #[cfg(target_os = "windows")]
        {
            Self::extract_zip(&temp_file_path, target_dir)?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::extract_tar_gz(&temp_file_path, target_dir)?;
        }

        // Cleanup temp file
        let _ = std::fs::remove_file(&temp_file_path);

        // Generate a default php.ini
        Self::generate_default_ini(minor_version)?;

        // Emit: complete
        let _ = app_handle.emit("php-install-progress", serde_json::json!({
            "version": minor_version,
            "stage": "complete",
            "progress": 100,
            "message": format!("PHP {} installed successfully!", minor_version),
        }));

        tracing::info!("PHP {} installed to {:?}", minor_version, target_dir);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn extract_zip(zip_path: &std::path::Path, target_dir: &std::path::Path) -> Result<()> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // PHP Windows zips typically have files inside a top-level directory like "php-8.3.20-nts-Win32-vs16-x64/"
        // We need to detect that and strip it so files end up directly in target_dir

        // Check if all entries share a common top-level directory
        let common_prefix = {
            let first_name = archive.by_index(0)?.name().to_string();
            let prefix = first_name.split('/').next().unwrap_or("").to_string();
            // Verify all entries share this prefix
            let mut all_share = !prefix.is_empty();
            for i in 1..archive.len() {
                let entry = archive.by_index(i)?;
                if !entry.name().starts_with(&format!("{}/", prefix)) && entry.name() != format!("{}/", prefix) {
                    // Some entries might be the prefix dir itself
                    if entry.name().trim_end_matches('/') != prefix {
                        all_share = false;
                        break;
                    }
                }
            }
            if all_share { Some(format!("{}/", prefix)) } else { None }
        };

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let raw_name = entry.name().to_string();

            // Strip common prefix if present
            let relative_name = if let Some(ref prefix) = common_prefix {
                if let Some(stripped) = raw_name.strip_prefix(prefix) {
                    stripped.to_string()
                } else if raw_name.trim_end_matches('/') == prefix.trim_end_matches('/') {
                    // This is the top-level dir itself, skip it
                    continue;
                } else {
                    raw_name.clone()
                }
            } else {
                raw_name.clone()
            };

            if relative_name.is_empty() {
                continue;
            }

            let out_path = target_dir.join(&relative_name);

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

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn extract_tar_gz(tar_path: &std::path::Path, target_dir: &std::path::Path) -> Result<()> {
        let file = std::fs::File::open(tar_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(target_dir)?;
        Ok(())
    }

    /// Generate a sensible default php.ini for a freshly installed version
    fn generate_default_ini(version: &str) -> Result<()> {
        let version_dir = Self::version_dir(version);

        // Look for php.ini-development or php.ini-production to use as base
        let dev_ini = version_dir.join("php.ini-development");
        let prod_ini = version_dir.join("php.ini-production");

        let base_content = if dev_ini.exists() {
            std::fs::read_to_string(&dev_ini)?
        } else if prod_ini.exists() {
            std::fs::read_to_string(&prod_ini)?
        } else {
            // Generate a minimal php.ini
            String::new()
        };

        let ini_path = version_dir.join("php.ini");

        if !base_content.is_empty() {
            // Use the bundled ini as base, with a few overrides appended
            let mut content = base_content;
            content.push_str("\n\n; === phpHerd overrides ===\n");
            content.push_str("memory_limit = 512M\n");
            content.push_str("max_execution_time = 300\n");
            content.push_str("upload_max_filesize = 100M\n");
            content.push_str("post_max_size = 100M\n");
            content.push_str("date.timezone = UTC\n");

            // Enable extension_dir relative to the PHP directory
            let ext_dir = version_dir.join("ext");
            if ext_dir.exists() {
                content.push_str(&format!(
                    "extension_dir = \"{}\"\n",
                    ext_dir.to_string_lossy().replace('\\', "/")
                ));
            }

            // Enable common extensions on Windows
            #[cfg(target_os = "windows")]
            {
                let common_exts = [
                    "curl", "fileinfo", "gd", "intl", "mbstring", "exif",
                    "mysqli", "openssl", "pdo_mysql", "pdo_sqlite", "zip",
                ];
                content.push_str("\n; Common extensions\n");
                for ext in &common_exts {
                    let dll = ext_dir.join(format!("php_{}.dll", ext));
                    if dll.exists() {
                        content.push_str(&format!("extension={}\n", ext));
                    }
                }
            }

            std::fs::write(&ini_path, content)?;
        } else {
            let ext_dir = version_dir.join("ext");
            let content = format!(
                "; phpHerd generated php.ini\n\
                 [PHP]\n\
                 memory_limit = 512M\n\
                 max_execution_time = 300\n\
                 upload_max_filesize = 100M\n\
                 post_max_size = 100M\n\
                 date.timezone = UTC\n\
                 error_reporting = E_ALL\n\
                 display_errors = On\n\
                 extension_dir = \"{}\"\n",
                ext_dir.to_string_lossy().replace('\\', "/")
            );
            std::fs::write(&ini_path, content)?;
        }

        tracing::info!("Generated php.ini for PHP {} at {:?}", version, ini_path);
        Ok(())
    }

    /// Create smart shims that check .php-version in the current directory
    /// and fall back to the global default version.
    pub fn update_global_shim(version: &str) -> Result<()> {
        let bin_dir = AppConfig::data_dir().join("bin");
        std::fs::create_dir_all(&bin_dir)?;

        let php_binary = Self::php_binary_path(version);
        if !php_binary.exists() {
            anyhow::bail!("PHP {} binary not found at {:?}", version, php_binary);
        }

        let php_dir = Self::php_dir().to_string_lossy().replace('\\', "\\\\");
        let default_ver = version;

        #[cfg(target_os = "windows")]
        {
            // Smart php.cmd — checks .php-version, falls back to default
            let php_shim = format!(
                r#"@echo off
setlocal enabledelayedexpansion
set "PHERD_PHP_VER={default_ver}"
if exist ".php-version" (
    set /p PHERD_PHP_VER=<.php-version
)
set "PHERD_PHP={php_dir}\\!PHERD_PHP_VER!\\php.exe"
if not exist "!PHERD_PHP!" (
    set "PHERD_PHP={php_dir}\\{default_ver}\\php.exe"
)
"!PHERD_PHP!" %*
"#,
                default_ver = default_ver,
                php_dir = Self::php_dir().to_string_lossy(),
            );
            std::fs::write(bin_dir.join("php.cmd"), php_shim)?;

            // Smart composer.cmd — uses the same PHP resolution
            let composer_phar = bin_dir.join("composer.phar");
            let composer_shim = format!(
                r#"@echo off
setlocal enabledelayedexpansion
set "PHERD_PHP_VER={default_ver}"
if exist ".php-version" (
    set /p PHERD_PHP_VER=<.php-version
)
set "PHERD_PHP={php_dir}\\!PHERD_PHP_VER!\\php.exe"
if not exist "!PHERD_PHP!" (
    set "PHERD_PHP={php_dir}\\{default_ver}\\php.exe"
)
"!PHERD_PHP!" "{composer_phar}" %*
"#,
                default_ver = default_ver,
                php_dir = Self::php_dir().to_string_lossy(),
                composer_phar = composer_phar.to_string_lossy(),
            );
            std::fs::write(bin_dir.join("composer.cmd"), composer_shim)?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Smart php shell script
            let php_shim = format!(
                r#"#!/bin/sh
PHERD_PHP_VER="{default_ver}"
if [ -f ".php-version" ]; then
    PHERD_PHP_VER=$(cat .php-version)
fi
PHERD_PHP="{php_dir}/$PHERD_PHP_VER/bin/php"
if [ ! -f "$PHERD_PHP" ]; then
    PHERD_PHP="{php_dir}/{default_ver}/bin/php"
fi
exec "$PHERD_PHP" "$@"
"#,
                default_ver = default_ver,
                php_dir = Self::php_dir().to_string_lossy(),
            );
            let shim_path = bin_dir.join("php");
            std::fs::write(&shim_path, php_shim)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&shim_path, std::fs::Permissions::from_mode(0o755))?;
            }

            // Smart composer shell script
            let composer_phar = bin_dir.join("composer.phar");
            let composer_shim = format!(
                r#"#!/bin/sh
PHERD_PHP_VER="{default_ver}"
if [ -f ".php-version" ]; then
    PHERD_PHP_VER=$(cat .php-version)
fi
PHERD_PHP="{php_dir}/$PHERD_PHP_VER/bin/php"
if [ ! -f "$PHERD_PHP" ]; then
    PHERD_PHP="{php_dir}/{default_ver}/bin/php"
fi
exec "$PHERD_PHP" "{composer_phar}" "$@"
"#,
                default_ver = default_ver,
                php_dir = Self::php_dir().to_string_lossy(),
                composer_phar = composer_phar.to_string_lossy(),
            );
            let shim_path = bin_dir.join("composer");
            std::fs::write(&shim_path, composer_shim)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&shim_path, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        tracing::info!("Updated global PHP/Composer shims (default: {})", version);
        Ok(())
    }

    /// Get the PHP version string by running php -v
    pub fn get_version_string(version: &str) -> Option<String> {
        let binary = Self::php_binary_path(version);
        if !binary.exists() {
            return None;
        }

        let output = std::process::Command::new(&binary)
            .arg("-v")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "PHP 8.3.20 (cli) ..." -> "8.3.20"
        stdout
            .lines()
            .next()
            .and_then(|line| {
                line.strip_prefix("PHP ")
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(String::from)
            })
    }

    /// Uninstall a PHP version by removing its directory
    pub fn uninstall_version(version: &str) -> Result<()> {
        let dir = Self::version_dir(version);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
            tracing::info!("Uninstalled PHP {}", version);
        }
        Ok(())
    }
}
