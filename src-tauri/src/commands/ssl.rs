use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SslStatus {
    pub ca_installed: bool,
    pub ca_path: Option<String>,
}

#[tauri::command]
pub async fn get_ssl_status() -> Result<SslStatus, String> {
    // TODO: Check if CA is installed
    Ok(SslStatus {
        ca_installed: false,
        ca_path: None,
    })
}
