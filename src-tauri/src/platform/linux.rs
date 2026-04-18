use crate::core::platform::PlatformOps;
use anyhow::Result;
use std::path::Path;

pub struct LinuxPlatform;

impl PlatformOps for LinuxPlatform {
    fn install_ca_certificate(&self, ca_path: &Path) -> Result<()> {
        let dest = format!(
            "/usr/local/share/ca-certificates/phpHerdCA.crt"
        );

        std::process::Command::new("sudo")
            .args(["cp", &ca_path.to_string_lossy(), &dest])
            .output()?;

        std::process::Command::new("sudo")
            .args(["update-ca-certificates"])
            .output()?;

        Ok(())
    }

    fn remove_ca_certificate(&self, _name: &str) -> Result<()> {
        std::process::Command::new("sudo")
            .args(["rm", "-f", "/usr/local/share/ca-certificates/phpHerdCA.crt"])
            .output()?;

        std::process::Command::new("sudo")
            .args(["update-ca-certificates", "--fresh"])
            .output()?;

        Ok(())
    }

    fn setup_dns_resolver(&self, tld: &str, listen_address: &str) -> Result<()> {
        // Configure systemd-resolved or NetworkManager
        tracing::info!(
            "Setting up DNS resolver for .{} -> {}",
            tld,
            listen_address
        );
        // TODO: Configure systemd-resolved for .test domain
        Ok(())
    }

    fn teardown_dns_resolver(&self, tld: &str) -> Result<()> {
        tracing::info!("Tearing down DNS resolver for .{}", tld);
        Ok(())
    }

    fn get_php_binary_url(&self, version: &str) -> String {
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        format!(
            "https://github.com/pherd/php-binaries/releases/download/v{}/php-{}-linux-{}.tar.gz",
            version, version, arch
        )
    }

    fn get_nginx_binary_url(&self) -> String {
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        format!(
            "https://github.com/pherd/nginx-binaries/releases/download/latest/nginx-linux-{}.tar.gz",
            arch
        )
    }

    fn get_service_binary_url(&self, service: &str, version: &str) -> String {
        match service {
            "redis" => format!("https://download.redis.io/releases/redis-{}.tar.gz", version),
            _ => String::new(),
        }
    }

    fn needs_admin_for_dns(&self) -> bool {
        true
    }

    fn request_admin(&self, reason: &str) -> Result<()> {
        tracing::info!("Admin privileges requested: {}", reason);
        Ok(())
    }

    fn php_fpm_socket_path(&self, version: &str) -> String {
        let data_dir = crate::core::config::AppConfig::data_dir();
        format!(
            "unix:{}/php/{}/var/run/php-fpm.sock",
            data_dir.to_string_lossy(),
            version
        )
    }

    fn name(&self) -> &str {
        "linux"
    }
}
