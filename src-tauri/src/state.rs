use crate::core::config::AppConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let config = AppConfig::load_or_create()?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }
}
