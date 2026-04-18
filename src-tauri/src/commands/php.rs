use crate::core::php_manager::PhpManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhpVersion {
    pub version: String,
    pub full_version: Option<String>,
    pub path: String,
    pub is_active: bool,
    pub is_installed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
}

#[tauri::command]
pub async fn get_php_versions(state: State<'_, AppState>) -> Result<Vec<PhpVersion>, String> {
    let config = state.config.read().await;
    let available = ["7.4", "8.0", "8.1", "8.2", "8.3", "8.4"];

    let versions = available
        .iter()
        .map(|ver| {
            let is_installed = PhpManager::is_installed(ver);
            let full_version = if is_installed {
                PhpManager::get_version_string(ver)
            } else {
                None
            };
            PhpVersion {
                version: ver.to_string(),
                full_version,
                path: PhpManager::version_dir(ver).to_string_lossy().to_string(),
                is_active: config.sites_config.default_php == *ver,
                is_installed,
            }
        })
        .collect();

    Ok(versions)
}

#[tauri::command]
pub async fn get_current_php_version(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.sites_config.default_php.clone())
}

#[tauri::command]
pub async fn install_php_version(
    version: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    PhpManager::install_version(&version, &app_handle)
        .await
        .map_err(|e| format!("Failed to install PHP {}: {}", version, e))
}

#[tauri::command]
pub async fn uninstall_php_version(version: String) -> Result<(), String> {
    PhpManager::uninstall_version(&version)
        .map_err(|e| format!("Failed to uninstall PHP {}: {}", version, e))
}

#[tauri::command]
pub async fn switch_php_version(
    version: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Verify it's installed
    if !PhpManager::is_installed(&version) {
        return Err(format!("PHP {} is not installed", version));
    }

    // Update the global shim
    PhpManager::update_global_shim(&version)
        .map_err(|e| format!("Failed to update shim: {}", e))?;

    // Save config
    let mut config = state.config.write().await;
    config.sites_config.default_php = version;
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_php_extensions(
    version: String,
) -> Result<Vec<PhpExtension>, String> {
    let ext_dir = PhpManager::version_dir(&version).join("ext");

    if !ext_dir.exists() {
        return Ok(Vec::new());
    }

    // Scan the ext directory for actual extension files
    let mut extensions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ext_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            // On Windows: php_curl.dll -> curl
            // On Unix: curl.so -> curl
            let ext_name = if cfg!(target_os = "windows") {
                name.strip_prefix("php_")
                    .and_then(|n| n.strip_suffix(".dll"))
                    .map(String::from)
            } else {
                name.strip_suffix(".so").map(String::from)
            };

            if let Some(ext_name) = ext_name {
                // Check if enabled in php.ini
                let ini_path = PhpManager::version_dir(&version).join("php.ini");
                let enabled = if ini_path.exists() {
                    let content = std::fs::read_to_string(&ini_path).unwrap_or_default();
                    content.contains(&format!("extension={}", ext_name))
                        && !content.contains(&format!(";extension={}", ext_name))
                } else {
                    false
                };

                extensions.push(PhpExtension {
                    name: ext_name,
                    enabled,
                });
            }
        }
    }

    extensions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(extensions)
}

#[tauri::command]
pub async fn toggle_php_extension(
    version: String,
    extension: String,
    enabled: bool,
) -> Result<(), String> {
    let ini_path = PhpManager::version_dir(&version).join("php.ini");
    if !ini_path.exists() {
        return Err("php.ini not found".to_string());
    }

    let content = std::fs::read_to_string(&ini_path).map_err(|e| e.to_string())?;
    let ext_line = format!("extension={}", extension);
    let disabled_line = format!(";extension={}", extension);

    let new_content = if enabled {
        if content.contains(&disabled_line) {
            content.replace(&disabled_line, &ext_line)
        } else if !content.contains(&ext_line) {
            format!("{}\n{}\n", content, ext_line)
        } else {
            content
        }
    } else {
        if content.contains(&ext_line) && !content.contains(&disabled_line) {
            content.replace(&ext_line, &disabled_line)
        } else {
            content
        }
    };

    std::fs::write(&ini_path, new_content).map_err(|e| e.to_string())?;
    Ok(())
}
