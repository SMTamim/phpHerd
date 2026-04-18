use crate::core::config::SiteEntry;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteInfo {
    pub name: String,
    pub path: String,
    pub url: String,
    pub php_version: Option<String>,
    pub node_version: Option<String>,
    pub secured: bool,
    pub is_parked: bool,
}

#[tauri::command]
pub async fn get_sites(state: State<'_, AppState>) -> Result<Vec<SiteInfo>, String> {
    let config = state.config.read().await;
    let tld = &config.sites_config.tld;
    let mut sites = Vec::new();

    // Gather linked sites
    for site in &config.sites_config.linked_sites {
        sites.push(SiteInfo {
            name: site.name.clone(),
            url: format!("http://{}.{}", site.name, tld),
            path: site.path.clone(),
            php_version: site.php_version.clone(),
            node_version: site.node_version.clone(),
            secured: site.secured,
            is_parked: false,
        });
    }

    // Scan parked directories
    for parked_path in &config.sites_config.parked_paths {
        let path = std::path::Path::new(parked_path);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !sites.iter().any(|s| s.name == name) {
                            sites.push(SiteInfo {
                                url: format!("http://{}.{}", name, tld),
                                path: entry.path().to_string_lossy().to_string(),
                                php_version: None,
                                node_version: None,
                                secured: false,
                                is_parked: true,
                                name,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(sites)
}

#[tauri::command]
pub async fn get_parked_paths(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.read().await;
    Ok(config.sites_config.parked_paths.clone())
}

#[tauri::command]
pub async fn park_directory(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    if !config.sites_config.parked_paths.contains(&path) {
        config.sites_config.parked_paths.push(path);
        config.save().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn unpark_directory(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.sites_config.parked_paths.retain(|p| p != &path);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn link_site(
    name: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    // Remove existing entry with same name
    config.sites_config.linked_sites.retain(|s| s.name != name);
    config.sites_config.linked_sites.push(SiteEntry {
        name,
        path,
        php_version: None,
        node_version: None,
        secured: false,
    });
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn unlink_site(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.sites_config.linked_sites.retain(|s| s.name != name);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn isolate_site_php(
    site_name: String,
    php_version: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    if let Some(site) = config
        .sites_config
        .linked_sites
        .iter_mut()
        .find(|s| s.name == site_name)
    {
        site.php_version = Some(php_version);
        config.save().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn secure_site(site_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    if let Some(site) = config
        .sites_config
        .linked_sites
        .iter_mut()
        .find(|s| s.name == site_name)
    {
        site.secured = true;
        config.save().map_err(|e| e.to_string())?;
    }
    // TODO: Generate SSL cert and update Nginx config
    Ok(())
}

#[tauri::command]
pub async fn unsecure_site(site_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.write().await;
    if let Some(site) = config
        .sites_config
        .linked_sites
        .iter_mut()
        .find(|s| s.name == site_name)
    {
        site.secured = false;
        config.save().map_err(|e| e.to_string())?;
    }
    Ok(())
}
