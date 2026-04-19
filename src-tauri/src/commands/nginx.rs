use crate::core::config::AppConfig;
use crate::core::nginx_manager::NginxManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct NginxStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub installed: bool,
}

#[tauri::command]
pub async fn get_nginx_status(state: State<'_, AppState>) -> Result<NginxStatus, String> {
    let installed = NginxManager::nginx_binary().exists();
    let pm = state.process_manager.read().await;
    let status = pm.status("nginx").await;
    let running = status == crate::core::process_manager::ProcessStatus::Running;

    let version = if installed {
        NginxManager::get_version_string()
    } else {
        None
    };

    let pid = if running {
        let procs = pm.list().await;
        procs.iter().find(|p| p.name == "nginx").map(|p| p.pid)
    } else {
        None
    };

    Ok(NginxStatus {
        running,
        pid,
        version,
        installed,
    })
}

#[tauri::command]
pub async fn install_nginx(app_handle: tauri::AppHandle) -> Result<(), String> {
    NginxManager::install(&app_handle)
        .await
        .map_err(|e| format!("Failed to install Nginx: {}", e))
}

#[tauri::command]
pub async fn start_nginx(state: State<'_, AppState>) -> Result<(), String> {
    let binary = NginxManager::nginx_binary();
    if !binary.exists() {
        return Err("Nginx is not installed. Install it first.".to_string());
    }

    // Generate config before starting
    let config = state.config.read().await;
    let tld = &config.sites_config.tld;
    NginxManager::write_main_config(tld).map_err(|e| e.to_string())?;

    let config_path = NginxManager::config_dir().join("nginx.conf");

    let pm = state.process_manager.write().await;
    pm.start(
        "nginx",
        binary.to_str().unwrap(),
        &["-c", config_path.to_str().unwrap(), "-g", "daemon off;"],
        Some(NginxManager::nginx_dir()),
        None,
    )
    .await
    .map_err(|e| format!("Failed to start Nginx: {}", e))?;

    tracing::info!("Nginx started");
    Ok(())
}

#[tauri::command]
pub async fn stop_nginx(state: State<'_, AppState>) -> Result<(), String> {
    let pm = state.process_manager.write().await;
    pm.stop("nginx")
        .await
        .map_err(|e| format!("Failed to stop Nginx: {}", e))?;
    tracing::info!("Nginx stopped");
    Ok(())
}

#[tauri::command]
pub async fn restart_nginx(state: State<'_, AppState>) -> Result<(), String> {
    // Stop then start
    let pm = state.process_manager.write().await;
    pm.stop("nginx").await.ok();
    drop(pm);

    // Re-generate config and start
    start_nginx(state).await
}
