use crate::core::node_manager::NodeManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeVersion {
    pub version: String,
    pub full_version: Option<String>,
    pub is_active: bool,
    pub is_installed: bool,
    pub path: String,
}

#[tauri::command]
pub async fn get_node_versions(state: State<'_, AppState>) -> Result<Vec<NodeVersion>, String> {
    let config = state.config.read().await;
    let active = config.settings.active_node.clone().unwrap_or_default();

    let available = ["18", "20", "22", "23", "24"];
    let versions = available
        .iter()
        .map(|ver| {
            let is_installed = NodeManager::is_installed(ver);
            let full_version = if is_installed {
                NodeManager::get_version_string(ver)
            } else {
                None
            };
            NodeVersion {
                version: ver.to_string(),
                full_version,
                is_active: active == *ver,
                is_installed,
                path: NodeManager::node_version_dir(ver).to_string_lossy().to_string(),
            }
        })
        .collect();

    Ok(versions)
}

#[tauri::command]
pub async fn get_current_node_version(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.read().await;
    Ok(config.settings.active_node.clone())
}

#[tauri::command]
pub async fn install_node_version(
    version: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    NodeManager::install_version(&version, &app_handle)
        .await
        .map_err(|e| format!("Failed to install Node.js {}: {}", version, e))
}

#[tauri::command]
pub async fn switch_node_version(
    version: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !NodeManager::is_installed(&version) {
        return Err(format!("Node.js {} is not installed", version));
    }

    NodeManager::update_global_shim(&version)
        .map_err(|e| format!("Failed to update shim: {}", e))?;

    let mut config = state.config.write().await;
    config.settings.active_node = Some(version);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}
