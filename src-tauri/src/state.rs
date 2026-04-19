use crate::core::config::AppConfig;
use crate::core::process_manager::ProcessManager;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub process_manager: Arc<RwLock<ProcessManager>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let config = AppConfig::load_or_create()?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            process_manager: Arc::new(RwLock::new(ProcessManager::new())),
        })
    }
}
