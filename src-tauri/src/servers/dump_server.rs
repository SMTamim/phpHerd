use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpPayload {
    pub id: String,
    pub dump_type: String,
    pub content: serde_json::Value,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub site: Option<String>,
    pub timestamp: String,
}

pub struct DumpServer {
    pub port: u16,
    pub dumps: Arc<RwLock<Vec<DumpPayload>>>,
    max_dumps: usize,
}

impl DumpServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            dumps: Arc::new(RwLock::new(Vec::new())),
            max_dumps: 1000,
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting dump server on port {}", self.port);
        // TODO: Start HTTP server that accepts POST /dump
        // Parse the payload and store in the ring buffer
        // Emit Tauri events to frontend
        Ok(())
    }

    pub async fn add_dump(&self, dump: DumpPayload) {
        let mut dumps = self.dumps.write().await;
        if dumps.len() >= self.max_dumps {
            dumps.remove(0);
        }
        dumps.push(dump);
    }

    pub async fn get_dumps(&self) -> Vec<DumpPayload> {
        self.dumps.read().await.clone()
    }

    pub async fn clear(&self) {
        self.dumps.write().await.clear();
    }
}
