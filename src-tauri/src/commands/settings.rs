use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub tld: String,
    pub default_php: String,
    pub parked_paths: Vec<String>,
    pub editor: String,
    pub auto_start: bool,
    pub smtp_port: u16,
    pub dump_port: u16,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let config = state.config.read().await;
    Ok(AppSettings {
        tld: config.sites_config.tld.clone(),
        default_php: config.sites_config.default_php.clone(),
        parked_paths: config.sites_config.parked_paths.clone(),
        editor: config.settings.editor.clone(),
        auto_start: config.settings.auto_start,
        smtp_port: config.settings.smtp_port,
        dump_port: config.settings.dump_port,
    })
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    config.sites_config.tld = settings.tld;
    config.sites_config.default_php = settings.default_php;
    config.settings.editor = settings.editor;
    config.settings.auto_start = settings.auto_start;
    config.settings.smtp_port = settings.smtp_port;
    config.settings.dump_port = settings.dump_port;
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}
