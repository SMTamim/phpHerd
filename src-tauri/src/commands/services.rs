use crate::core::service_manager::{ServiceConfig, ServiceManager};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub status: String,
    pub data_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub service_type: String,
    pub version: String,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableService {
    pub service_type: String,
    pub display_name: String,
    pub versions: Vec<String>,
    pub default_port: u16,
}

#[tauri::command]
pub async fn get_services(state: State<'_, AppState>) -> Result<Vec<ServiceInfo>, String> {
    let registry = ServiceManager::list_services();
    let pm = state.process_manager.read().await;

    let mut services = Vec::new();
    for svc in &registry {
        let proc_status = pm.status(&svc.id).await;
        let status = if proc_status == crate::core::process_manager::ProcessStatus::Running {
            "Running".to_string()
        } else {
            "Stopped".to_string()
        };

        services.push(ServiceInfo {
            id: svc.id.clone(),
            service_type: svc.service_type.clone(),
            version: svc.version.clone(),
            port: svc.port,
            status,
            data_dir: ServiceManager::service_data_dir(&svc.service_type, &svc.version)
                .to_string_lossy()
                .to_string(),
        });
    }

    Ok(services)
}

#[tauri::command]
pub async fn get_available_services() -> Result<Vec<AvailableService>, String> {
    Ok(vec![
        AvailableService {
            service_type: "mysql".to_string(),
            display_name: "MySQL".to_string(),
            versions: vec!["8.0".to_string(), "8.4".to_string()],
            default_port: 3306,
        },
        AvailableService {
            service_type: "mariadb".to_string(),
            display_name: "MariaDB".to_string(),
            versions: vec!["10.11".to_string(), "11.4".to_string()],
            default_port: 3307,
        },
        AvailableService {
            service_type: "postgresql".to_string(),
            display_name: "PostgreSQL".to_string(),
            versions: vec!["15".to_string(), "16".to_string(), "17".to_string()],
            default_port: 5432,
        },
        AvailableService {
            service_type: "redis".to_string(),
            display_name: "Redis".to_string(),
            versions: vec!["7.2".to_string(), "7.4".to_string()],
            default_port: 6379,
        },
        AvailableService {
            service_type: "mongodb".to_string(),
            display_name: "MongoDB".to_string(),
            versions: vec!["7.0".to_string(), "8.0".to_string()],
            default_port: 27017,
        },
        AvailableService {
            service_type: "meilisearch".to_string(),
            display_name: "Meilisearch".to_string(),
            versions: vec!["1.9".to_string(), "1.10".to_string()],
            default_port: 7700,
        },
        AvailableService {
            service_type: "typesense".to_string(),
            display_name: "Typesense".to_string(),
            versions: vec!["0.25".to_string(), "27.1".to_string()],
            default_port: 8108,
        },
        AvailableService {
            service_type: "minio".to_string(),
            display_name: "MinIO".to_string(),
            versions: vec!["latest".to_string()],
            default_port: 9000,
        },
    ])
}

#[tauri::command]
pub async fn create_service(
    request: CreateServiceRequest,
    app_handle: tauri::AppHandle,
) -> Result<ServiceInfo, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let port = request
        .port
        .unwrap_or_else(|| ServiceManager::default_port(&request.service_type));

    // Create directory structure
    ServiceManager::create_service_dirs(&request.service_type, &request.version)
        .map_err(|e| e.to_string())?;

    // Download binary if not already installed
    if !ServiceManager::is_binary_installed(&request.service_type, &request.version) {
        ServiceManager::download_binary(&request.service_type, &request.version, &app_handle)
            .await
            .map_err(|e| format!("Failed to download {}: {}", request.service_type, e))?;
    }

    // Add to registry
    let mut registry = ServiceManager::list_services();
    let config = ServiceConfig {
        id: id.clone(),
        service_type: request.service_type.clone(),
        version: request.version.clone(),
        port,
        auto_start: false,
    };
    registry.push(config);
    ServiceManager::save_registry(&registry).map_err(|e| e.to_string())?;

    let data_dir = ServiceManager::service_data_dir(&request.service_type, &request.version)
        .to_string_lossy()
        .to_string();

    tracing::info!(
        "Created service {} v{} on port {}",
        request.service_type,
        request.version,
        port
    );

    Ok(ServiceInfo {
        id,
        service_type: request.service_type,
        version: request.version,
        port,
        status: "Stopped".to_string(),
        data_dir,
    })
}

#[tauri::command]
pub async fn download_service_binary(
    service_type: String,
    version: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    ServiceManager::download_binary(&service_type, &version, &app_handle)
        .await
        .map_err(|e| format!("Failed to download {} v{}: {}", service_type, version, e))
}

#[tauri::command]
pub async fn start_service(
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let registry = ServiceManager::list_services();
    let svc = registry
        .iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service {} not found", service_id))?;

    // Check if a binary exists for this service
    let bin_dir = ServiceManager::service_bin_dir(&svc.service_type, &svc.version);
    let binary = find_service_binary(&svc.service_type, &bin_dir);

    if let Some(binary_path) = binary {
        let data_dir = ServiceManager::service_data_dir(&svc.service_type, &svc.version);
        let args = build_service_args(&svc.service_type, &svc.version, svc.port, &data_dir);
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let pm = state.process_manager.write().await;
        pm.start(
            &service_id,
            binary_path.to_str().unwrap(),
            &args_refs,
            Some(bin_dir),
            None,
        )
        .await
        .map_err(|e| format!("Failed to start {}: {}", svc.service_type, e))?;

        tracing::info!("Started service {} ({})", svc.service_type, service_id);
    } else {
        // No binary — mark as "started" in a placeholder sense
        // In a full implementation we'd download the binary first
        tracing::warn!(
            "No binary found for {} v{} at {:?}. Service registered but cannot start yet.",
            svc.service_type,
            svc.version,
            bin_dir
        );
        return Err(format!(
            "No binary found for {} v{}. Place the binary in {:?} or download it first.",
            svc.service_type, svc.version, bin_dir
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_service(
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pm = state.process_manager.write().await;
    pm.stop(&service_id)
        .await
        .map_err(|e| format!("Failed to stop service: {}", e))?;
    tracing::info!("Stopped service {}", service_id);
    Ok(())
}

#[tauri::command]
pub async fn delete_service(
    service_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Stop if running
    let pm = state.process_manager.write().await;
    pm.stop(&service_id).await.ok();
    drop(pm);

    // Remove from registry
    let mut registry = ServiceManager::list_services();
    if let Some(svc) = registry.iter().find(|s| s.id == service_id).cloned() {
        // Remove service directory
        let dir = ServiceManager::service_dir(&svc.service_type, &svc.version);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).ok();
        }
    }

    registry.retain(|s| s.id != service_id);
    ServiceManager::save_registry(&registry).map_err(|e| e.to_string())?;

    tracing::info!("Deleted service {}", service_id);
    Ok(())
}

/// Find the binary for a service type in the bin directory
fn find_service_binary(
    service_type: &str,
    bin_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let names: Vec<&str> = match service_type {
        "mysql" => vec!["mysqld.exe", "mysqld"],
        "mariadb" => vec!["mariadbd.exe", "mariadbd", "mysqld.exe", "mysqld"],
        "postgresql" => vec!["postgres.exe", "postgres"],
        "redis" => vec!["redis-server.exe", "redis-server"],
        "mongodb" => vec!["mongod.exe", "mongod"],
        "meilisearch" => vec!["meilisearch.exe", "meilisearch"],
        "typesense" => vec!["typesense-server.exe", "typesense-server"],
        "minio" => vec!["minio.exe", "minio"],
        _ => vec![],
    };

    for name in names {
        let path = bin_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Build command-line arguments for starting a service
fn build_service_args(
    service_type: &str,
    _version: &str,
    port: u16,
    data_dir: &std::path::Path,
) -> Vec<String> {
    let data = data_dir.to_string_lossy().to_string();
    match service_type {
        "mysql" | "mariadb" => vec![
            format!("--port={}", port),
            format!("--datadir={}", data),
            "--console".to_string(),
        ],
        "postgresql" => vec![
            "-D".to_string(),
            data.clone(),
            "-p".to_string(),
            port.to_string(),
        ],
        "redis" => vec![
            "--port".to_string(),
            port.to_string(),
            "--dir".to_string(),
            data,
        ],
        "mongodb" => vec![
            "--port".to_string(),
            port.to_string(),
            "--dbpath".to_string(),
            data,
        ],
        "meilisearch" => vec![
            "--http-addr".to_string(),
            format!("127.0.0.1:{}", port),
            "--db-path".to_string(),
            data,
        ],
        "minio" => vec![
            "server".to_string(),
            data,
            "--address".to_string(),
            format!(":{}", port),
        ],
        _ => vec![],
    }
}
