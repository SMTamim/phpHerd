use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsStatus {
    pub running: bool,
    pub tld: String,
    pub resolver_type: String,
}

#[tauri::command]
pub async fn get_dns_status() -> Result<DnsStatus, String> {
    let resolver_type = if cfg!(target_os = "windows") {
        "dns-proxy"
    } else if cfg!(target_os = "macos") {
        "dnsmasq"
    } else {
        "dnsmasq"
    };

    Ok(DnsStatus {
        running: false,
        tld: "test".to_string(),
        resolver_type: resolver_type.to_string(),
    })
}
