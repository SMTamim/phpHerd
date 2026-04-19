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

    /// Check if a service binary is installed
    pub fn is_binary_installed(service_type: &str, version: &str) -> bool {
        let bin_dir = Self::service_bin_dir(service_type, version);
        let names = Self::binary_names(service_type);
        names.iter().any(|name| bin_dir.join(name).exists())
    }

    /// Get possible binary names for a service type
    fn binary_names(service_type: &str) -> Vec<&'static str> {
        match service_type {
            "mysql" => vec!["mysqld.exe", "mysqld"],
            "mariadb" => vec!["mariadbd.exe", "mariadbd", "mysqld.exe", "mysqld"],
            "postgresql" => vec!["postgres.exe", "postgres", "bin/postgres.exe", "bin/postgres"],
            "redis" => vec!["redis-server.exe", "redis-server"],
            "mongodb" => vec!["mongod.exe", "mongod"],
            "meilisearch" => vec!["meilisearch.exe", "meilisearch"],
            "typesense" => vec!["typesense-server.exe", "typesense-server"],
            "minio" => vec!["minio.exe", "minio"],
            _ => vec![],
        }
    }

    /// Get download URL for a service binary
    fn download_url(service_type: &str, version: &str) -> Option<DownloadInfo> {
        #[cfg(target_os = "windows")]
        {
            match service_type {
                "mysql" => {
                    let full_ver = match version {
                        "8.0" => "8.0.42",
                        "8.4" => "8.4.5",
                        _ => return None,
                    };
                    Some(DownloadInfo {
                        url: format!(
                            "https://dev.mysql.com/get/Downloads/MySQL-{}/mysql-{}-winx64.zip",
                            version, full_ver
                        ),
                        archive_type: ArchiveType::Zip,
                        strip_prefix: true,
                    })
                }
                "mariadb" => {
                    let full_ver = match version {
                        "10.11" => "10.11.11",
                        "11.4" => "11.4.5",
                        _ => return None,
                    };
                    Some(DownloadInfo {
                        url: format!(
                            "https://archive.mariadb.org/mariadb-{}/winx64-packages/mariadb-{}-winx64.zip",
                            full_ver, full_ver
                        ),
                        archive_type: ArchiveType::Zip,
                        strip_prefix: true,
                    })
                }
                "postgresql" => {
                    let full_ver = match version {
                        "15" => "15.13-1",
                        "16" => "16.9-1",
                        "17" => "17.4-1",
                        _ => return None,
                    };
                    Some(DownloadInfo {
                        url: format!(
                            "https://get.enterprisedb.com/postgresql/postgresql-{}-windows-x64-binaries.zip",
                            full_ver
                        ),
                        archive_type: ArchiveType::Zip,
                        strip_prefix: true,
                    })
                }
                "redis" => {
                    let full_ver = match version {
                        "7.2" => "7.2.7",
                        "7.4" => "7.4.2",
                        _ => return None,
                    };
                    Some(DownloadInfo {
                        url: format!(
                            "https://github.com/redis-windows/redis-windows/releases/download/{}/Redis-{}-Windows-x64.zip",
                            full_ver, full_ver
                        ),
                        archive_type: ArchiveType::Zip,
                        strip_prefix: true,
                    })
                }
                "meilisearch" => {
                    let full_ver = match version {
                        "1.9" => "v1.9.0",
                        "1.10" => "v1.10.0",
                        _ => return None,
                    };
                    Some(DownloadInfo {
                        url: format!(
                            "https://github.com/meilisearch/meilisearch/releases/download/{}/meilisearch-windows-amd64.exe",
                            full_ver
                        ),
                        archive_type: ArchiveType::SingleBinary("meilisearch.exe".to_string()),
                        strip_prefix: false,
                    })
                }
                "minio" => {
                    Some(DownloadInfo {
                        url: "https://dl.min.io/server/minio/release/windows-amd64/minio.exe".to_string(),
                        archive_type: ArchiveType::SingleBinary("minio.exe".to_string()),
                        strip_prefix: false,
                    })
                }
                _ => None,
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS services are typically installed via Homebrew
            // For now, return None — could add direct binary URLs later
            let _ = (service_type, version);
            None
        }

        #[cfg(target_os = "linux")]
        {
            let _ = (service_type, version);
            None
        }
    }

    /// Download and install a service binary
    pub async fn download_binary(
        service_type: &str,
        version: &str,
        app_handle: &tauri::AppHandle,
    ) -> Result<()> {
        use futures_util::StreamExt;
        use std::io::Write;
        use tauri::Emitter;

        let info = Self::download_url(service_type, version)
            .ok_or_else(|| anyhow::anyhow!("No download available for {} v{}", service_type, version))?;

        let bin_dir = Self::service_bin_dir(service_type, version);
        std::fs::create_dir_all(&bin_dir)?;

        tracing::info!("Downloading {} v{} from {}", service_type, version, info.url);

        let _ = app_handle.emit("service-download-progress", serde_json::json!({
            "service_type": service_type,
            "version": version,
            "stage": "downloading",
            "progress": 0,
            "message": format!("Downloading {}...", service_type),
        }));

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;
        let response = client.get(&info.url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download {} v{}: HTTP {}",
                service_type, version, response.status()
            );
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        match info.archive_type {
            ArchiveType::SingleBinary(ref filename) => {
                // Download directly to bin dir
                let target_path = bin_dir.join(filename);
                let mut file = std::fs::File::create(&target_path)?;
                let mut stream = response.bytes_stream();

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    file.write_all(&chunk)?;
                    downloaded += chunk.len() as u64;

                    if total_size > 0 {
                        let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                        let _ = app_handle.emit("service-download-progress", serde_json::json!({
                            "service_type": service_type,
                            "version": version,
                            "stage": "downloading",
                            "progress": progress,
                            "message": format!("Downloading... {}%", progress),
                        }));
                    }
                }

                tracing::info!("Downloaded single binary to {:?}", target_path);
            }
            ArchiveType::Zip => {
                // Download to temp, then extract
                let temp_path = std::env::temp_dir().join(format!("{}-{}.zip", service_type, version));
                let mut temp_file = std::fs::File::create(&temp_path)?;
                let mut stream = response.bytes_stream();

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    temp_file.write_all(&chunk)?;
                    downloaded += chunk.len() as u64;

                    if total_size > 0 {
                        let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                        let _ = app_handle.emit("service-download-progress", serde_json::json!({
                            "service_type": service_type,
                            "version": version,
                            "stage": "downloading",
                            "progress": progress,
                            "message": format!("Downloading... {}%", progress),
                        }));
                    }
                }
                drop(temp_file);

                let _ = app_handle.emit("service-download-progress", serde_json::json!({
                    "service_type": service_type,
                    "version": version,
                    "stage": "extracting",
                    "progress": 100,
                    "message": "Extracting...",
                }));

                // Extract — find binaries and copy to bin_dir
                Self::extract_service_zip(&temp_path, service_type, version, info.strip_prefix)?;

                let _ = std::fs::remove_file(&temp_path);
            }
        }

        // Run any post-install initialization
        Self::post_install_init(service_type, version)?;

        let _ = app_handle.emit("service-download-progress", serde_json::json!({
            "service_type": service_type,
            "version": version,
            "stage": "complete",
            "progress": 100,
            "message": format!("{} v{} installed!", service_type, version),
        }));

        tracing::info!("{} v{} binary installed", service_type, version);
        Ok(())
    }

    /// Extract a service zip archive, placing relevant files into the service directory
    fn extract_service_zip(
        zip_path: &std::path::Path,
        service_type: &str,
        version: &str,
        strip_prefix: bool,
    ) -> Result<()> {
        let service_dir = Self::service_dir(service_type, version);
        let bin_dir = Self::service_bin_dir(service_type, version);

        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Detect common top-level prefix to strip
        let common_prefix = if strip_prefix {
            let first = archive.by_index(0)?.name().to_string();
            let prefix = first.split('/').next().unwrap_or("").to_string();
            if !prefix.is_empty() {
                Some(format!("{}/", prefix))
            } else {
                None
            }
        } else {
            None
        };

        // For services, we want to extract specific directories:
        // - bin/ files go to our bin_dir
        // - share/, lib/, etc. go to service_dir
        // For some services (Redis), everything is flat in the zip

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
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

            if relative.is_empty() {
                continue;
            }

            // Determine where to put the file
            let is_binary = Self::is_service_binary_file(&relative, service_type);
            let out_path = if is_binary {
                // Place binaries flat in bin_dir
                let filename = std::path::Path::new(&relative)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                bin_dir.join(filename)
            } else {
                // Keep other files in the service directory structure
                service_dir.join(&relative)
            };

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&out_path)?;
                std::io::copy(&mut entry, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Check if a file path within an archive is a service binary we want
    fn is_service_binary_file(relative_path: &str, service_type: &str) -> bool {
        let lower = relative_path.to_lowercase();
        let filename = std::path::Path::new(&lower)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match service_type {
            "mysql" => matches!(
                filename.as_str(),
                "mysqld.exe" | "mysqld" | "mysql.exe" | "mysql"
                | "mysqladmin.exe" | "mysqladmin" | "mysqldump.exe" | "mysqldump"
                | "mysql_upgrade.exe" | "mysql_upgrade"
            ),
            "mariadb" => matches!(
                filename.as_str(),
                "mariadbd.exe" | "mariadbd" | "mysqld.exe" | "mysqld"
                | "mariadb.exe" | "mariadb" | "mysql.exe" | "mysql"
                | "mariadb-install-db.exe" | "mariadb-install-db"
                | "mysqladmin.exe" | "mysqladmin"
            ),
            "postgresql" => {
                // PostgreSQL has bin/ directory with many tools
                let in_bin = relative_path.contains("bin/") || relative_path.contains("bin\\");
                in_bin && (filename.ends_with(".exe") || !filename.contains('.'))
            }
            "redis" => filename.starts_with("redis-"),
            "mongodb" => matches!(
                filename.as_str(),
                "mongod.exe" | "mongod" | "mongos.exe" | "mongos"
                | "mongo.exe" | "mongo" | "mongosh.exe" | "mongosh"
            ),
            _ => false,
        }
    }

    /// Run post-install initialization for a service
    fn post_install_init(service_type: &str, version: &str) -> Result<()> {
        let bin_dir = Self::service_bin_dir(service_type, version);
        let data_dir = Self::service_data_dir(service_type, version);

        match service_type {
            "mysql" => {
                // Initialize MySQL data directory if empty
                if data_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
                    let mysqld = bin_dir.join(if cfg!(windows) { "mysqld.exe" } else { "mysqld" });
                    if mysqld.exists() {
                        tracing::info!("Initializing MySQL data directory...");
                        let basedir = Self::service_dir(service_type, version);
                        let status = std::process::Command::new(&mysqld)
                            .arg("--initialize-insecure")
                            .arg(format!("--basedir={}", basedir.to_string_lossy()))
                            .arg(format!("--datadir={}", data_dir.to_string_lossy()))
                            .status();

                        match status {
                            Ok(s) if s.success() => {
                                tracing::info!("MySQL data directory initialized");
                            }
                            Ok(s) => {
                                tracing::warn!("MySQL init exited with {}", s);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to run mysqld --initialize: {}", e);
                            }
                        }
                    }
                }
            }
            "postgresql" => {
                // Run initdb if data dir is empty
                if data_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
                    let initdb = bin_dir.join(if cfg!(windows) { "initdb.exe" } else { "initdb" });
                    if initdb.exists() {
                        tracing::info!("Initializing PostgreSQL data directory...");
                        let status = std::process::Command::new(&initdb)
                            .arg("-D")
                            .arg(data_dir.to_str().unwrap())
                            .arg("-U")
                            .arg("postgres")
                            .arg("--encoding=UTF8")
                            .status();

                        match status {
                            Ok(s) if s.success() => {
                                tracing::info!("PostgreSQL data directory initialized");
                            }
                            Ok(s) => {
                                tracing::warn!("initdb exited with {}", s);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to run initdb: {}", e);
                            }
                        }
                    }
                }
            }
            _ => {}
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

struct DownloadInfo {
    url: String,
    archive_type: ArchiveType,
    strip_prefix: bool,
}

enum ArchiveType {
    Zip,
    SingleBinary(String),
}
