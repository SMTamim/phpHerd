use crate::core::config::AppConfig;
use crate::core::php_manager::PhpManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposerStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposerOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Path to composer.phar in the bin directory
fn composer_phar_path() -> std::path::PathBuf {
    AppConfig::data_dir().join("bin").join("composer.phar")
}

#[tauri::command]
pub async fn get_composer_status(state: State<'_, AppState>) -> Result<ComposerStatus, String> {
    let phar = composer_phar_path();
    let installed = phar.exists();

    let version = if installed {
        let config = state.config.read().await;
        let php = &config.sites_config.default_php;
        let php_bin = PhpManager::php_binary_path(php);
        if php_bin.exists() {
            let output = std::process::Command::new(&php_bin)
                .args([phar.to_str().unwrap(), "--version", "--no-ansi"])
                .output()
                .ok();
            output.and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .next()
                    .and_then(|l| l.strip_prefix("Composer version "))
                    .and_then(|l| l.split_whitespace().next())
                    .map(String::from)
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(ComposerStatus {
        installed,
        version,
        path: phar.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn install_composer() -> Result<(), String> {
    let bin_dir = AppConfig::data_dir().join("bin");
    std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    let phar_path = bin_dir.join("composer.phar");

    tracing::info!("Downloading Composer...");

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("phpHerd/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://getcomposer.org/download/latest-stable/composer.phar")
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to download Composer: HTTP {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&phar_path, &bytes).map_err(|e| e.to_string())?;

    // Create/update composer.cmd shim
    update_composer_shim().ok();

    tracing::info!("Composer installed to {:?}", phar_path);
    Ok(())
}

/// Update the composer.cmd shim to use the current active PHP
fn update_composer_shim() -> Result<(), String> {
    let config = AppConfig::load_or_create().map_err(|e| e.to_string())?;
    let php_bin = PhpManager::php_binary_path(&config.sites_config.default_php);
    let phar = composer_phar_path();
    let bin_dir = AppConfig::data_dir().join("bin");

    if !php_bin.exists() || !phar.exists() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let shim = bin_dir.join("composer.cmd");
        let content = format!(
            "@echo off\r\n\"{}\" \"{}\" %*\r\n",
            php_bin.to_string_lossy(),
            phar.to_string_lossy()
        );
        std::fs::write(&shim, content).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let shim = bin_dir.join("composer");
        let content = format!(
            "#!/bin/sh\n\"{}\" \"{}\" \"$@\"\n",
            php_bin.to_string_lossy(),
            phar.to_string_lossy()
        );
        std::fs::write(&shim, &content).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).ok();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn run_composer(
    site_path: String,
    args: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ComposerOutput, String> {
    let phar = composer_phar_path();
    if !phar.exists() {
        return Err("Composer is not installed. Install it first from the Dashboard.".to_string());
    }

    let config = state.config.read().await;
    let php_version = &config.sites_config.default_php;
    let php_bin = PhpManager::php_binary_path(php_version);

    if !php_bin.exists() {
        return Err(format!("PHP {} is not installed", php_version));
    }

    let mut cmd_args = vec![phar.to_string_lossy().to_string()];
    cmd_args.extend(args.clone());
    cmd_args.push("--no-interaction".to_string());
    cmd_args.push("--ansi".to_string());

    tracing::info!("Running composer {} in {}", args.join(" "), site_path);

    let output = std::process::Command::new(&php_bin)
        .args(&cmd_args)
        .current_dir(&site_path)
        .env("COMPOSER_HOME", AppConfig::data_dir().join("composer"))
        .output()
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        tracing::warn!("Composer failed: {}", stderr);
    }

    Ok(ComposerOutput {
        success: output.status.success(),
        stdout,
        stderr,
    })
}
