use crate::core::platform::PlatformOps;
use anyhow::Result;
use std::path::Path;

pub struct MacOsPlatform;

impl PlatformOps for MacOsPlatform {
    fn install_ca_certificate(&self, ca_path: &Path) -> Result<()> {
        let output = std::process::Command::new("sudo")
            .args([
                "security",
                "add-trusted-cert",
                "-d",
                "-r",
                "trustRoot",
                "-k",
                "/Library/Keychains/System.keychain",
                &ca_path.to_string_lossy(),
            ])
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
        let output = std::process::Command::new("sudo")
            .args([
                "security",
                "delete-certificate",
                "-c",
                name,
                "/Library/Keychains/System.keychain",
            ])
            .output()?;

        if !output.status.success() {
            tracing::warn!("Failed to remove CA: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    fn setup_dns_resolver(&self, tld: &str, listen_address: &str) -> Result<()> {
        // Create /etc/resolver/<tld> file
        let resolver_dir = "/etc/resolver";
        let resolver_file = format!("{}/{}", resolver_dir, tld);

        let content = format!("nameserver {}\n", listen_address);

        std::process::Command::new("sudo")
            .args(["mkdir", "-p", resolver_dir])
            .output()?;

        std::process::Command::new("sudo")
            .args(["bash", "-c", &format!("echo '{}' > {}", content.trim(), resolver_file)])
            .output()?;

        Ok(())
    }

    fn teardown_dns_resolver(&self, tld: &str) -> Result<()> {
        let resolver_file = format!("/etc/resolver/{}", tld);
        std::process::Command::new("sudo")
            .args(["rm", "-f", &resolver_file])
            .output()?;
        Ok(())
    }

    fn get_php_binary_url(&self, version: &str) -> String {
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x86_64"
        };
        format!(
            "https://github.com/pherd/php-binaries/releases/download/v{}/php-{}-darwin-{}.tar.gz",
            version, version, arch
        )
    }

    fn get_nginx_binary_url(&self) -> String {
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x86_64"
        };
        format!(
            "https://github.com/pherd/nginx-binaries/releases/download/latest/nginx-darwin-{}.tar.gz",
            arch
        )
    }

    fn get_service_binary_url(&self, service: &str, version: &str) -> String {
        match service {
            "mysql" => format!("https://dev.mysql.com/get/Downloads/MySQL-{}/mysql-{}-macos-arm64.tar.gz", version, version),
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
        "macos"
    }
}
