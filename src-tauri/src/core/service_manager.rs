use crate::core::config::AppConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub id: String,
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub auto_start: bool,
}

pub struct ServiceManager;

impl ServiceManager {
    pub fn services_dir() -> PathBuf {
        AppConfig::data_dir().join("services")
    }

    pub fn service_dir(service_type: &str, version: &str) -> PathBuf {
        Self::services_dir().join(format!("{}-{}", service_type, version))
    }

    pub fn service_data_dir(service_type: &str, version: &str) -> PathBuf {
        Self::service_dir(service_type, version).join("data")
    }

    pub fn service_config_dir(service_type: &str, version: &str) -> PathBuf {
        Self::service_dir(service_type, version).join("config")
    }

    pub fn service_bin_dir(service_type: &str, version: &str) -> PathBuf {
        Self::service_dir(service_type, version).join("bin")
    }

    /// Get the default port for a service type
    pub fn default_port(service_type: &str) -> u16 {
        match service_type {
            "mysql" => 3306,
            "mariadb" => 3307,
            "postgresql" => 5432,
            "redis" => 6379,
            "mongodb" => 27017,
            "meilisearch" => 7700,
            "typesense" => 8108,
            "minio" => 9000,
            _ => 0,
        }
    }

    /// Create service directory structure
    pub fn create_service_dirs(service_type: &str, version: &str) -> Result<()> {
        let dirs = [
            Self::service_bin_dir(service_type, version),
            Self::service_data_dir(service_type, version),
            Self::service_config_dir(service_type, version),
        ];

        for dir in &dirs {
            std::fs::create_dir_all(dir)?;
        }

        Ok(())
    }

    /// List all installed services
    pub fn list_services() -> Vec<ServiceConfig> {
        let services_dir = Self::services_dir();
        let registry_path = services_dir.join("registry.json");

        if registry_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&registry_path) {
                if let Ok(services) = serde_json::from_str(&content) {
                    return services;
                }
            }
        }

        Vec::new()
    }

    /// Save service registry
    pub fn save_registry(services: &[ServiceConfig]) -> Result<()> {
        let registry_path = Self::services_dir().join("registry.json");
        std::fs::create_dir_all(Self::services_dir())?;
        let content = serde_json::to_string_pretty(services)?;
        std::fs::write(registry_path, content)?;
        Ok(())
    }
}
