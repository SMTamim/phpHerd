use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeVersion {
    pub version: String,
    pub is_active: bool,
    pub is_installed: bool,
    pub path: String,
}

#[tauri::command]
pub async fn get_node_versions() -> Result<Vec<NodeVersion>, String> {
    // TODO: Scan installed Node versions
    Ok(Vec::new())
}

#[tauri::command]
pub async fn get_current_node_version() -> Result<Option<String>, String> {
    // TODO: Get current Node version
    Ok(None)
}

#[tauri::command]
pub async fn install_node_version(version: String) -> Result<(), String> {
    tracing::info!("Installing Node.js v{}", version);
    // TODO: Download and install Node.js version
    Ok(())
}

#[tauri::command]
pub async fn switch_node_version(version: String) -> Result<(), String> {
    tracing::info!("Switching to Node.js v{}", version);
    // TODO: Update symlinks
    Ok(())
}
