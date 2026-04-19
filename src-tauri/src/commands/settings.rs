use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub tld: String,
    pub default_php: String,
    pub parked_paths: Vec<String>,
    pub editor: String,
    pub auto_start: bool,
    pub smtp_port: u16,
    pub dump_port: u16,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let config = state.config.read().await;
    Ok(AppSettings {
        tld: config.sites_config.tld.clone(),
        default_php: config.sites_config.default_php.clone(),
        parked_paths: config.sites_config.parked_paths.clone(),
        editor: config.settings.editor.clone(),
        auto_start: config.settings.auto_start,
        smtp_port: config.settings.smtp_port,
        dump_port: config.settings.dump_port,
    })
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.sites_config.tld = settings.tld;
    config.sites_config.default_php = settings.default_php;
    config.settings.editor = settings.editor;
    config.settings.auto_start = settings.auto_start;
    config.settings.smtp_port = settings.smtp_port;
    config.settings.dump_port = settings.dump_port;
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_bin_to_path() -> Result<bool, String> {
    let bin_dir = crate::core::config::AppConfig::data_dir()
        .join("bin")
        .to_string_lossy()
        .to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        // Read current user PATH
        let output = std::process::Command::new("powershell")
            .creation_flags(0x08000000)
            .args([
                "-NoProfile",
                "-Command",
                "[Environment]::GetEnvironmentVariable('PATH', 'User')",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let current = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if current.to_lowercase().contains(&bin_dir.to_lowercase()) {
            return Ok(false); // Already on PATH
        }

        // Add to user PATH
        let new_path = format!("{};{}", bin_dir, current);
        let status = std::process::Command::new("powershell")
            .creation_flags(0x08000000)
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "[Environment]::SetEnvironmentVariable('PATH', '{}', 'User')",
                    new_path.replace('\'', "''")
                ),
            ])
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to update PATH".to_string());
        }

        tracing::info!("Added {} to user PATH", bin_dir);
        Ok(true)
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, suggest adding to shell profile
        Err(format!(
            "Add this to your shell profile (~/.bashrc or ~/.zshrc):\nexport PATH=\"{}:$PATH\"",
            bin_dir
        ))
    }
}

#[tauri::command]
pub async fn check_bin_on_path() -> Result<bool, String> {
    let bin_dir = crate::core::config::AppConfig::data_dir()
        .join("bin")
        .to_string_lossy()
        .to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let output = std::process::Command::new("powershell")
            .creation_flags(0x08000000)
            .args([
                "-NoProfile",
                "-Command",
                "[Environment]::GetEnvironmentVariable('PATH', 'User')",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(current.to_lowercase().contains(&bin_dir.to_lowercase()))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let path = std::env::var("PATH").unwrap_or_default();
        Ok(path.contains(&bin_dir))
    }
}
