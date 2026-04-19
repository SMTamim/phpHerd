use crate::core::config::AppConfig;
use anyhow::Result;
use std::path::PathBuf;

pub struct NginxManager;

impl NginxManager {
    pub fn nginx_dir() -> PathBuf {
        AppConfig::data_dir().join("nginx")
    }

    pub fn config_dir() -> PathBuf {
        AppConfig::data_dir().join("config").join("nginx")
    }

    pub fn sites_config_dir() -> PathBuf {
        Self::config_dir().join("sites")
    }

    pub fn nginx_binary() -> PathBuf {
        let dir = Self::nginx_dir();
        if cfg!(target_os = "windows") {
            dir.join("nginx.exe")
        } else {
            dir.join("sbin").join("nginx")
        }
    }

    /// Get the Nginx version string by running nginx -v
    pub fn get_version_string() -> Option<String> {
        let binary = Self::nginx_binary();
        if !binary.exists() {
            return None;
        }
        let output = std::process::Command::new(&binary)
            .arg("-v")
            .output()
            .ok()?;
        // nginx -v outputs to stderr: "nginx version: nginx/1.27.3"
        let stderr = String::from_utf8_lossy(&output.stderr);
        stderr
            .lines()
            .next()
            .and_then(|line| line.split('/').last())
            .map(|v| v.trim().to_string())
    }

    /// Download and install Nginx
    pub async fn install(app_handle: &tauri::AppHandle) -> Result<()> {
        use tauri::Emitter;

        let target_dir = Self::nginx_dir();

        #[cfg(target_os = "windows")]
        let url = "https://nginx.org/download/nginx-1.27.3.zip";
        #[cfg(not(target_os = "windows"))]
        let url = "https://nginx.org/download/nginx-1.27.3.tar.gz";

        tracing::info!("Downloading Nginx from {}", url);

        let _ = app_handle.emit("nginx-install-progress", serde_json::json!({
            "stage": "downloading",
            "progress": 0,
            "message": "Downloading Nginx...",
        }));

        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download Nginx: HTTP {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let temp_dir = std::env::temp_dir();
        let ext = if cfg!(target_os = "windows") { "zip" } else { "tar.gz" };
        let temp_file_path = temp_dir.join(format!("nginx.{}", ext));
        let mut temp_file = std::fs::File::create(&temp_file_path)?;

        use futures_util::StreamExt;
        use std::io::Write;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            temp_file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u32;
                let _ = app_handle.emit("nginx-install-progress", serde_json::json!({
                    "stage": "downloading",
                    "progress": progress,
                    "message": format!("Downloading... {}%", progress),
                }));
            }
        }
        drop(temp_file);

        let _ = app_handle.emit("nginx-install-progress", serde_json::json!({
            "stage": "extracting",
            "progress": 100,
            "message": "Extracting...",
        }));

        // Clean and extract
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        std::fs::create_dir_all(&target_dir)?;

        #[cfg(target_os = "windows")]
        {
            let file = std::fs::File::open(&temp_file_path)?;
            let mut archive = zip::ZipArchive::new(file)?;

            // Nginx zips have a top-level "nginx-1.27.3/" directory — strip it
            let common_prefix = {
                let first = archive.by_index(0)?.name().to_string();
                let prefix = first.split('/').next().unwrap_or("").to_string();
                if !prefix.is_empty() { Some(format!("{}/", prefix)) } else { None }
            };

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let raw_name = entry.name().to_string();

                let relative = if let Some(ref prefix) = common_prefix {
                    if let Some(stripped) = raw_name.strip_prefix(prefix) {
                        stripped.to_string()
                    } else if raw_name.trim_end_matches('/') == prefix.trim_end_matches('/') {
                        continue;
                    } else {
                        raw_name.clone()
                    }
                } else {
                    raw_name.clone()
                };

                if relative.is_empty() {
                    continue;
                }

                let out_path = target_dir.join(&relative);
                if entry.is_dir() {
                    std::fs::create_dir_all(&out_path)?;
                } else {
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = std::fs::File::create(&out_path)?;
                    std::io::copy(&mut entry, &mut outfile)?;
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let file = std::fs::File::open(&temp_file_path)?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&target_dir)?;
        }

        let _ = std::fs::remove_file(&temp_file_path);

        // Create necessary directories
        std::fs::create_dir_all(Self::config_dir())?;
        std::fs::create_dir_all(Self::sites_config_dir())?;
        std::fs::create_dir_all(AppConfig::data_dir().join("logs"))?;

        // Write initial config
        Self::write_main_config("test")?;

        let _ = app_handle.emit("nginx-install-progress", serde_json::json!({
            "stage": "complete",
            "progress": 100,
            "message": "Nginx installed successfully!",
        }));

        tracing::info!("Nginx installed to {:?}", target_dir);
        Ok(())
    }

    /// Write the main nginx.conf to disk
    pub fn write_main_config(tld: &str) -> Result<()> {
        let config = Self::generate_main_config(tld)?;
        let config_path = Self::config_dir().join("nginx.conf");
        std::fs::create_dir_all(Self::config_dir())?;
        std::fs::write(config_path, config)?;
        Ok(())
    }

    /// Generate the main nginx.conf
    pub fn generate_main_config(tld: &str) -> Result<String> {
        let sites_dir = Self::sites_config_dir();
        let logs_dir = AppConfig::data_dir().join("logs");
        let nginx_dir = Self::nginx_dir();

        std::fs::create_dir_all(&sites_dir)?;
        std::fs::create_dir_all(&logs_dir)?;

        let config = format!(
            r#"worker_processes auto;
error_log "{logs_dir}/nginx-error.log";
pid "{logs_dir}/nginx.pid";

events {{
    worker_connections 1024;
}}

http {{
    include       "{nginx_dir}/conf/mime.types";
    default_type  application/octet-stream;

    sendfile        on;
    keepalive_timeout  65;

    access_log "{logs_dir}/nginx-access.log";

    # Include all site configs
    include "{sites_dir}/*.conf";

    # Default server - catch all .{tld} requests
    server {{
        listen 80 default_server;
        server_name *.{tld};

        location / {{
            return 404 "phpHerd: site not found";
        }}
    }}
}}
"#,
            logs_dir = logs_dir.to_string_lossy().replace('\\', "/"),
            nginx_dir = nginx_dir.to_string_lossy().replace('\\', "/"),
            sites_dir = sites_dir.to_string_lossy().replace('\\', "/"),
            tld = tld,
        );

        Ok(config)
    }

    /// Generate a server block config for a site
    pub fn generate_site_config(
        name: &str,
        root_path: &str,
        tld: &str,
        php_fpm_socket: &str,
        secured: bool,
    ) -> Result<String> {
        let ssl_dir = AppConfig::data_dir().join("config").join("ssl").join("certs");

        let mut config = String::new();

        if secured {
            config.push_str(&format!(
                r#"server {{
    listen 443 ssl http2;
    server_name {name}.{tld};
    root "{root_path}/public";

    ssl_certificate "{ssl_dir}/{name}.{tld}.crt";
    ssl_certificate_key "{ssl_dir}/{name}.{tld}.key";

    index index.php index.html index.htm;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        fastcgi_pass {php_fpm_socket};
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    location ~ /\.ht {{
        deny all;
    }}
}}

server {{
    listen 80;
    server_name {name}.{tld};
    return 301 https://$host$request_uri;
}}
"#,
                name = name,
                tld = tld,
                root_path = root_path.replace('\\', "/"),
                ssl_dir = ssl_dir.to_string_lossy().replace('\\', "/"),
                php_fpm_socket = php_fpm_socket,
            ));
        } else {
            config.push_str(&format!(
                r#"server {{
    listen 80;
    server_name {name}.{tld};
    root "{root_path}/public";

    index index.php index.html index.htm;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location ~ \.php$ {{
        fastcgi_pass {php_fpm_socket};
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    location ~ /\.ht {{
        deny all;
    }}
}}
"#,
                name = name,
                tld = tld,
                root_path = root_path.replace('\\', "/"),
                php_fpm_socket = php_fpm_socket,
            ));
        }

        Ok(config)
    }

    /// Write site config to disk
    pub fn write_site_config(name: &str, config: &str) -> Result<()> {
        let config_path = Self::sites_config_dir().join(format!("{}.conf", name));
        std::fs::create_dir_all(Self::sites_config_dir())?;
        std::fs::write(config_path, config)?;
        Ok(())
    }

    /// Remove site config
    pub fn remove_site_config(name: &str) -> Result<()> {
        let config_path = Self::sites_config_dir().join(format!("{}.conf", name));
        if config_path.exists() {
            std::fs::remove_file(config_path)?;
        }
        Ok(())
    }
}
