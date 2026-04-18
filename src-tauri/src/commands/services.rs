use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    pub id: String,
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub data_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error,
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
pub async fn get_services() -> Result<Vec<ServiceInfo>, String> {
    // TODO: Read from service registry
    Ok(Vec::new())
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
pub async fn create_service(request: CreateServiceRequest) -> Result<ServiceInfo, String> {
    let id = uuid::Uuid::new_v4().to_string();
    tracing::info!("Creating service {} v{}", request.service_type, request.version);

    Ok(ServiceInfo {
        id,
        service_type: request.service_type,
        version: request.version,
        port: request.port.unwrap_or(3306),
        status: ServiceStatus::Stopped,
        data_dir: String::new(),
    })
}

#[tauri::command]
pub async fn start_service(service_id: String) -> Result<(), String> {
    tracing::info!("Starting service {}", service_id);
    // TODO: Start the service process
    Ok(())
}

#[tauri::command]
pub async fn stop_service(service_id: String) -> Result<(), String> {
    tracing::info!("Stopping service {}", service_id);
    // TODO: Stop the service process
    Ok(())
}

#[tauri::command]
pub async fn delete_service(service_id: String) -> Result<(), String> {
    tracing::info!("Deleting service {}", service_id);
    // TODO: Stop and remove service
    Ok(())
}
