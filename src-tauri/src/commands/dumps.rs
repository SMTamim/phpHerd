use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DumpEntry {
    pub id: String,
    pub timestamp: String,
    pub dump_type: DumpType,
    pub content: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub site: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DumpType {
    Dump,
    Query,
    Job,
    View,
    HttpRequest,
    Log,
}

#[tauri::command]
pub async fn get_dumps() -> Result<Vec<DumpEntry>, String> {
    // TODO: Return dumps from the ring buffer
    Ok(Vec::new())
}

#[tauri::command]
pub async fn clear_dumps() -> Result<(), String> {
    // TODO: Clear the dump ring buffer
    Ok(())
}
