use crate::core::dns_manager::DnsManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsStatus {
    pub running: bool,
    pub tld: String,
    pub resolver_type: String,
    pub hosts_entries: Vec<String>,
}

#[tauri::command]
pub async fn get_dns_status(state: State<'_, AppState>) -> Result<DnsStatus, String> {
    let config = state.config.read().await;
    let tld = config.sites_config.tld.clone();

    let resolver_type = if cfg!(target_os = "windows") {
        "hosts-file"
    } else if cfg!(target_os = "macos") {
        "dnsmasq"
    } else {
        "dnsmasq"
    };

    let hosts_entries = DnsManager::get_hosts_entries(&tld);

    Ok(DnsStatus {
        running: !hosts_entries.is_empty(),
        tld,
        resolver_type: resolver_type.to_string(),
        hosts_entries,
    })
}

/// Sync the system hosts file with all known sites.
/// On Windows this requires running as admin.
#[tauri::command]
pub async fn sync_hosts_file(state: State<'_, AppState>) -> Result<usize, String> {
    let config = state.config.read().await;
    let tld = config.sites_config.tld.clone();

    // Collect all site names: linked sites + parked directory subdirs
    let mut site_names: Vec<String> = Vec::new();

    for site in &config.sites_config.linked_sites {
        site_names.push(site.name.clone());
    }

    for parked_path in &config.sites_config.parked_paths {
        let path = std::path::Path::new(parked_path);
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !site_names.contains(&name) {
                            site_names.push(name);
                        }
                    }
                }
            }
        }
    }

    let count = site_names.len();

    DnsManager::sync_hosts_file(&site_names, &tld)
        .map_err(|e| {
            if e.to_string().contains("Access is denied") || e.to_string().contains("Permission denied") {
                "Permission denied. Please run phpHerd as Administrator to update the hosts file.".to_string()
            } else {
                format!("Failed to update hosts file: {}", e)
            }
        })?;

    Ok(count)
}
