use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CapturedEmail {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub raw: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct SmtpServer {
    pub port: u16,
    pub emails: Arc<RwLock<Vec<CapturedEmail>>>,
}

impl SmtpServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            emails: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting SMTP server on port {}", self.port);
        // TODO: Implement SMTP server using mailin-embedded or tokio-based SMTP
        // For now this is a placeholder
        Ok(())
    }

    pub async fn get_emails(&self) -> Vec<CapturedEmail> {
        self.emails.read().await.clone()
    }

    pub async fn clear(&self) {
        self.emails.write().await.clear();
    }
}
