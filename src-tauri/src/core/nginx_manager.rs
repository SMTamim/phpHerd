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

    /// Generate the main nginx.conf
    pub fn generate_main_config(tld: &str) -> Result<String> {
        let sites_dir = Self::sites_config_dir();
        let logs_dir = AppConfig::data_dir().join("logs");

        let config = format!(
            r#"worker_processes auto;
error_log {logs_dir}/nginx-error.log;
pid {logs_dir}/nginx.pid;

events {{
    worker_connections 1024;
}}

http {{
    include       mime.types;
    default_type  application/octet-stream;

    sendfile        on;
    keepalive_timeout  65;

    access_log {logs_dir}/nginx-access.log;

    # Include all site configs
    include {sites_dir}/*.conf;

    # Default server - catch all .{tld} requests
    server {{
        listen 80 default_server;
        server_name *.{tld};
        root /var/www/html;

        location / {{
            try_files $uri $uri/ /index.php?$query_string;
        }}

        location ~ \.php$ {{
            fastcgi_pass 127.0.0.1:9000;
            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
            include fastcgi_params;
        }}
    }}
}}
"#,
            logs_dir = logs_dir.to_string_lossy().replace('\\', "/"),
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
    root {root_path}/public;

    ssl_certificate {ssl_dir}/{name}.{tld}.crt;
    ssl_certificate_key {ssl_dir}/{name}.{tld}.key;

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
    root {root_path}/public;

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
