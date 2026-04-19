use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub sites_config: SitesConfig,
    pub settings: GeneralSettings,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SitesConfig {
    pub parked_paths: Vec<String>,
    pub linked_sites: Vec<SiteEntry>,
    pub default_php: String,
    pub tld: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteEntry {
    pub name: String,
    pub path: String,
    pub php_version: Option<String>,
    pub node_version: Option<String>,
    pub secured: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralSettings {
    pub editor: String,
    pub auto_start: bool,
    pub smtp_port: u16,
    pub dump_port: u16,
    #[serde(default)]
    pub active_node: Option<String>,
}

impl Default for SitesConfig {
    fn default() -> Self {
        let default_parked = Self::default_parked_path();
        Self {
            parked_paths: vec![default_parked],
            linked_sites: Vec::new(),
            default_php: "8.3".to_string(),
            tld: "test".to_string(),
        }
    }
}

impl SitesConfig {
    fn default_parked_path() -> String {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Herd")
            .to_string_lossy()
            .to_string()
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            editor: "code".to_string(),
            auto_start: true,
            smtp_port: 2525,
            dump_port: 9912,
            active_node: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sites_config: SitesConfig::default(),
            settings: GeneralSettings::default(),
            config_path: None,
        }
    }
}

impl AppConfig {
    pub fn data_dir() -> PathBuf {
        let base = if cfg!(target_os = "windows") {
            dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        };

        if cfg!(target_os = "windows") {
            base.join("pherd")
        } else {
            base.join(".pherd")
        }
    }

    pub fn config_dir() -> PathBuf {
        Self::data_dir().join("config")
    }

    fn config_file_path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::config_file_path();

        // Ensure all directories exist
        let data_dir = Self::data_dir();
        let dirs_to_create = [
            data_dir.clone(),
            data_dir.join("config"),
            data_dir.join("config").join("nginx"),
            data_dir.join("config").join("nginx").join("sites"),
            data_dir.join("config").join("ssl"),
            data_dir.join("config").join("ssl").join("ca"),
            data_dir.join("config").join("ssl").join("certs"),
            data_dir.join("php"),
            data_dir.join("node"),
            data_dir.join("nginx"),
            data_dir.join("services"),
            data_dir.join("bin"),
            data_dir.join("logs"),
            data_dir.join("mail"),
            data_dir.join("dumps"),
        ];

        for dir in &dirs_to_create {
            std::fs::create_dir_all(dir)?;
        }

        // Also ensure default parked path exists
        let default_parked = SitesConfig::default_parked_path();
        std::fs::create_dir_all(&default_parked)?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let mut config: AppConfig = serde_json::from_str(&content)?;
            config.config_path = Some(config_path);
            Ok(config)
        } else {
            let mut config = AppConfig::default();
            config.config_path = Some(config_path);
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = self
            .config_path
            .as_ref()
            .cloned()
            .unwrap_or_else(Self::config_file_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
