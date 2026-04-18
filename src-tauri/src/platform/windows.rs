use crate::core::platform::PlatformOps;
use anyhow::Result;
use std::path::Path;

pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
    fn install_ca_certificate(&self, ca_path: &Path) -> Result<()> {
        // Use certutil to add to Windows certificate store
        let output = std::process::Command::new("certutil")
            .args(["-addstore", "Root", &ca_path.to_string_lossy()])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to install CA: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn remove_ca_certificate(&self, name: &str) -> Result<()> {
        let output = std::process::Command::new("certutil")
            .args(["-delstore", "Root", name])
            .output()?;

        if !output.status.success() {
            tracing::warn!("Failed to remove CA: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    fn setup_dns_resolver(&self, _tld: &str, _listen_address: &str) -> Result<()> {
        // Windows: Use a DNS proxy service or modify NRPT rules
        // For now, we'll use the hosts file approach as a fallback
        tracing::info!("Setting up DNS resolver on Windows");
        // TODO: Implement Windows DNS proxy using trust-dns
        Ok(())
    }

    fn teardown_dns_resolver(&self, _tld: &str) -> Result<()> {
        tracing::info!("Tearing down DNS resolver on Windows");
        Ok(())
    }

    fn get_php_binary_url(&self, version: &str) -> String {
        format!(
            "https://windows.php.net/downloads/releases/php-{}-nts-Win32-vs16-x64.zip",
            version
        )
    }

    fn get_nginx_binary_url(&self) -> String {
        "https://nginx.org/download/nginx-1.27.3.zip".to_string()
    }

    fn get_service_binary_url(&self, service: &str, version: &str) -> String {
        match service {
            "mysql" => format!("https://dev.mysql.com/get/Downloads/MySQL-{}/mysql-{}-winx64.zip", version, version),
            "redis" => format!("https://github.com/tporadowski/redis/releases/download/v{}/Redis-x64-{}.zip", version, version),
            _ => String::new(),
        }
    }

    fn needs_admin_for_dns(&self) -> bool {
        true
    }

    fn request_admin(&self, reason: &str) -> Result<()> {
        tracing::info!("Admin privileges requested: {}", reason);
        // TODO: Use runas or ShellExecute with "runas" verb
        Ok(())
    }

    fn php_fpm_socket_path(&self, version: &str) -> String {
        // Windows doesn't use Unix sockets, use TCP
        let port = 9000 + version.replace('.', "").parse::<u16>().unwrap_or(0);
        format!("127.0.0.1:{}", port)
    }

    fn name(&self) -> &str {
        "windows"
    }
}
