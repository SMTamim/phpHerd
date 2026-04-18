use anyhow::Result;
use std::path::Path;

/// Platform abstraction trait for OS-specific operations
pub trait PlatformOps: Send + Sync {
    /// Install a CA certificate into the system trust store
    fn install_ca_certificate(&self, ca_path: &Path) -> Result<()>;

    /// Remove a CA certificate from the system trust store
    fn remove_ca_certificate(&self, name: &str) -> Result<()>;

    /// Set up DNS resolver for the given TLD (e.g., "test")
    fn setup_dns_resolver(&self, tld: &str, listen_address: &str) -> Result<()>;

    /// Tear down DNS resolver for the given TLD
    fn teardown_dns_resolver(&self, tld: &str) -> Result<()>;

    /// Get the download URL for a PHP binary of the given version
    fn get_php_binary_url(&self, version: &str) -> String;

    /// Get the download URL for the Nginx binary
    fn get_nginx_binary_url(&self) -> String;

    /// Get the download URL for a service binary
    fn get_service_binary_url(&self, service: &str, version: &str) -> String;

    /// Check if admin/root privileges are needed for an operation
    fn needs_admin_for_dns(&self) -> bool;

    /// Request elevated privileges
    fn request_admin(&self, reason: &str) -> Result<()>;

    /// Get the PHP-FPM socket path for a given PHP version
    fn php_fpm_socket_path(&self, version: &str) -> String;

    /// Get the platform name
    fn name(&self) -> &str;
}

/// Create the appropriate platform implementation for the current OS
pub fn create_platform() -> Box<dyn PlatformOps> {
    #[cfg(target_os = "windows")]
    {
        Box::new(crate::platform::windows::WindowsPlatform)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(crate::platform::macos::MacOsPlatform)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(crate::platform::linux::LinuxPlatform)
    }
}
