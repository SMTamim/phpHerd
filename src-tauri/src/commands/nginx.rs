use crate::core::nginx_manager::NginxManager;
use crate::core::php_manager::PhpManager;
use crate::core::site_manager::SiteManager;
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

    let config = state.config.read().await;
    let tld = &config.sites_config.tld;
    let default_php = &config.sites_config.default_php;

    // 1. Start PHP-CGI if a PHP version is installed
    let php_cgi_port: u16 = 9000;
    if PhpManager::is_installed(default_php) {
        let php_cgi = PhpManager::version_dir(default_php).join("php-cgi.exe");
        if php_cgi.exists() {
            let pm = state.process_manager.write().await;
            // Stop old one first
            pm.stop("php-cgi").await.ok();
            pm.start(
                "php-cgi",
                php_cgi.to_str().unwrap(),
                &["-b", &format!("127.0.0.1:{}", php_cgi_port)],
                None,
                None,
            )
            .await
            .map_err(|e| format!("Failed to start PHP-CGI: {}", e))?;
            tracing::info!("PHP-CGI started on port {}", php_cgi_port);
            drop(pm);
        }
    }

    // 2. Generate site configs for all linked + parked sites
    let php_fpm_addr = format!("127.0.0.1:{}", php_cgi_port);

    // Linked sites
    for site in &config.sites_config.linked_sites {
        let doc_root = SiteManager::detect_document_root(&site.path);
        let site_config = NginxManager::generate_site_config(
            &site.name,
            &doc_root,
            tld,
            &php_fpm_addr,
            site.secured,
        )
        .map_err(|e| e.to_string())?;
        NginxManager::write_site_config(&site.name, &site_config)
            .map_err(|e| e.to_string())?;
    }

    // Parked directories
    for parked_path in &config.sites_config.parked_paths {
        let path = std::path::Path::new(parked_path);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Skip if a linked site with same name exists
                        if config.sites_config.linked_sites.iter().any(|s| s.name == name) {
                            continue;
                        }
                        let doc_root = SiteManager::detect_document_root(
                            &entry.path().to_string_lossy(),
                        );
                        let site_config = NginxManager::generate_site_config(
                            &name, &doc_root, tld, &php_fpm_addr, false,
                        )
                        .map_err(|e| e.to_string())?;
                        NginxManager::write_site_config(&name, &site_config)
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    // 3. Write main nginx.conf and start Nginx
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
    pm.stop("php-cgi").await.ok();
    pm.stop("nginx")
        .await
        .map_err(|e| format!("Failed to stop Nginx: {}", e))?;
    tracing::info!("Nginx and PHP-CGI stopped");
    Ok(())
}

#[tauri::command]
pub async fn restart_nginx(state: State<'_, AppState>) -> Result<(), String> {
    let pm = state.process_manager.write().await;
    pm.stop("php-cgi").await.ok();
    pm.stop("nginx").await.ok();
    drop(pm);

    start_nginx(state).await
}
