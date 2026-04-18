use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

#[tauri::command]
pub async fn get_log_files(site_path: Option<String>) -> Result<Vec<LogFile>, String> {
    let mut log_files = Vec::new();

    if let Some(path) = site_path {
        let log_dir = std::path::Path::new(&path).join("storage").join("logs");
        if log_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&log_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "log") {
                        if let Ok(metadata) = entry.metadata() {
                            log_files.push(LogFile {
                                name: entry.file_name().to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                size: metadata.len(),
                                modified: format!("{:?}", metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(log_files)
}

#[tauri::command]
pub async fn get_log_entries(
    file_path: String,
    search: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();

    for line in content.lines().rev().take(500) {
        if let Some(ref search_term) = search {
            if !line.to_lowercase().contains(&search_term.to_lowercase()) {
                continue;
            }
        }
        entries.push(LogEntry {
            timestamp: String::new(),
            level: String::new(),
            message: line.to_string(),
            context: None,
        });
    }

    Ok(entries)
}
