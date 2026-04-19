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
        site.php_version = Some(php_version.clone());

        // Write .php-version file in the project root so CLI shims pick it up
        let version_file = std::path::Path::new(&site.path).join(".php-version");
        std::fs::write(&version_file, &php_version).ok();
        tracing::info!("Wrote .php-version ({}) to {}", php_version, site.path);

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

#[tauri::command]
pub async fn install_phpmyadmin(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use futures_util::StreamExt;
    use std::io::Write;
    use tauri::Emitter;

    let config = state.config.read().await;
    let tld = config.sites_config.tld.clone();

    // Check if already linked
    if config.sites_config.linked_sites.iter().any(|s| s.name == "pma") {
        return Err("phpMyAdmin is already installed (pma.test)".to_string());
    }
    drop(config);

    // Install into ~/Herd/pma
    let install_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Herd")
        .join("pma");

    let _ = app_handle.emit("phpmyadmin-install-progress", serde_json::json!({
        "stage": "downloading",
        "progress": 0,
        "message": "Downloading phpMyAdmin...",
    }));

    let url = "https://www.phpmyadmin.net/downloads/phpMyAdmin-latest-all-languages.zip";
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("phpHerd/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Failed to download phpMyAdmin: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let temp_path = std::env::temp_dir().join("phpmyadmin.zip");
    let mut temp_file = std::fs::File::create(&temp_path).map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        temp_file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
            let _ = app_handle.emit("phpmyadmin-install-progress", serde_json::json!({
                "stage": "downloading",
                "progress": progress,
                "message": format!("Downloading... {}%", progress),
            }));
        }
    }
    drop(temp_file);

    let _ = app_handle.emit("phpmyadmin-install-progress", serde_json::json!({
        "stage": "extracting",
        "progress": 100,
        "message": "Extracting...",
    }));

    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir).ok();
    }

    // Extract — strip top-level "phpMyAdmin-x.x.x-all-languages/" prefix
    let file = std::fs::File::open(&temp_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let common_prefix = {
        let first = archive.by_index(0).map_err(|e| e.to_string())?.name().to_string();
        let prefix = first.split('/').next().unwrap_or("").to_string();
        if !prefix.is_empty() { Some(format!("{}/", prefix)) } else { None }
    };

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
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

        if relative.is_empty() { continue; }

        let out_path = install_dir.join(&relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    let _ = std::fs::remove_file(&temp_path);

    // Write config.inc.php for local use
    let config_content = r#"<?php
$cfg['blowfish_secret'] = 'phpHerd-auto-generated-secret-key-32ch';
$i = 0;
$i++;
$cfg['Servers'][$i]['auth_type'] = 'cookie';
$cfg['Servers'][$i]['host'] = '127.0.0.1';
$cfg['Servers'][$i]['port'] = '3306';
$cfg['Servers'][$i]['compress'] = false;
$cfg['Servers'][$i]['AllowNoPassword'] = true;
$cfg['UploadDir'] = '';
$cfg['SaveDir'] = '';
"#;
    std::fs::write(install_dir.join("config.inc.php"), config_content)
        .map_err(|e| e.to_string())?;

    // Link as pma.test
    let mut config = state.config.write().await;
    config.sites_config.linked_sites.retain(|s| s.name != "pma");
    config.sites_config.linked_sites.push(SiteEntry {
        name: "pma".to_string(),
        path: install_dir.to_string_lossy().to_string(),
        php_version: None,
        node_version: None,
        secured: false,
    });
    config.save().map_err(|e| e.to_string())?;

    let _ = app_handle.emit("phpmyadmin-install-progress", serde_json::json!({
        "stage": "complete",
        "progress": 100,
        "message": "phpMyAdmin installed at pma.test!",
    }));

    tracing::info!("phpMyAdmin installed to {:?}, linked as pma.{}", install_dir, tld);
    Ok(())
}
