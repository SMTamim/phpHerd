use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailMessage {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub timestamp: String,
    pub read: bool,
    pub app_name: Option<String>,
}

#[tauri::command]
pub async fn get_emails() -> Result<Vec<EmailMessage>, String> {
    // TODO: Read from mail storage
    Ok(Vec::new())
}

#[tauri::command]
pub async fn delete_email(email_id: String) -> Result<(), String> {
    tracing::info!("Deleting email {}", email_id);
    // TODO: Delete email file
    Ok(())
}

#[tauri::command]
pub async fn clear_all_emails() -> Result<(), String> {
    tracing::info!("Clearing all emails");
    // TODO: Clear mail directory
    Ok(())
}
