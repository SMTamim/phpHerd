use crate::core::config::AppConfig;
use anyhow::Result;
use std::path::PathBuf;

pub struct DnsManager;

impl DnsManager {
    pub fn dnsmasq_dir() -> PathBuf {
        AppConfig::data_dir().join("dnsmasq")
    }

    pub fn dnsmasq_config_path() -> PathBuf {
        AppConfig::config_dir().join("dnsmasq.conf")
    }

    /// Generate dnsmasq config to resolve *.tld to 127.0.0.1
    pub fn generate_config(tld: &str) -> Result<String> {
        let config = format!(
            "# phpHerd DNS configuration\n\
             address=/{tld}/127.0.0.1\n\
             listen-address=127.0.0.1\n\
             port=53\n",
            tld = tld,
        );
        Ok(config)
    }

    /// Write dnsmasq config to disk
    pub fn write_config(tld: &str) -> Result<()> {
        let config = Self::generate_config(tld)?;
        let path = Self::dnsmasq_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, config)?;
        Ok(())
    }

    // -- Windows hosts file management --

    const HOSTS_MARKER_BEGIN: &'static str = "# --- phpHerd BEGIN ---";
    const HOSTS_MARKER_END: &'static str = "# --- phpHerd END ---";

    /// Get the path to the system hosts file
    fn hosts_file_path() -> PathBuf {
        if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
        } else {
            PathBuf::from("/etc/hosts")
        }
    }

    /// Sync the hosts file with all .test domains.
    /// On Windows, uses UAC elevation via PowerShell if direct write fails.
    pub fn sync_hosts_file(site_names: &[String], tld: &str) -> Result<()> {
        let hosts_path = Self::hosts_file_path();
        let content = std::fs::read_to_string(&hosts_path).unwrap_or_default();

        // Remove existing phpHerd block
        let cleaned = Self::remove_pherd_block(&content);

        // Build new block
        let mut block = String::new();
        block.push_str(Self::HOSTS_MARKER_BEGIN);
        block.push('\n');
        for name in site_names {
            block.push_str(&format!("127.0.0.1    {}.{}\n", name, tld));
        }
        block.push_str(Self::HOSTS_MARKER_END);
        block.push('\n');

        // Assemble final content
        let mut new_content = cleaned.trim_end().to_string();
        new_content.push_str("\n\n");
        new_content.push_str(&block);

        // Try direct write first
        match std::fs::write(&hosts_path, &new_content) {
            Ok(()) => {
                tracing::info!(
                    "Updated hosts file with {} entries for .{} domains",
                    site_names.len(),
                    tld
                );
                return Ok(());
            }
            Err(e) => {
                tracing::info!("Direct hosts write failed ({}), requesting elevation...", e);
            }
        }

        // Elevate on Windows via PowerShell
        #[cfg(target_os = "windows")]
        {
            Self::write_hosts_elevated(&new_content)?;
            tracing::info!(
                "Updated hosts file (elevated) with {} entries for .{} domains",
                site_names.len(),
                tld
            );
            return Ok(());
        }

        #[cfg(not(target_os = "windows"))]
        {
            anyhow::bail!("Permission denied writing hosts file. Run with sudo.");
        }
    }

    /// Write hosts file content using UAC elevation on Windows.
    /// Writes to a temp file, then uses PowerShell Start-Process -Verb RunAs
    /// to copy it over the real hosts file.
    #[cfg(target_os = "windows")]
    fn write_hosts_elevated(content: &str) -> Result<()> {
        use std::os::windows::process::CommandExt;

        // Write desired hosts content to a temp file
        let temp_hosts = std::env::temp_dir().join("pherd_hosts.tmp");
        std::fs::write(&temp_hosts, content)?;

        // Write a small PS1 script that does the copy
        let temp_script = std::env::temp_dir().join("pherd_update_hosts.ps1");
        let script = format!(
            "Copy-Item -LiteralPath '{}' -Destination '{}' -Force\r\n",
            temp_hosts.to_string_lossy(),
            Self::hosts_file_path().to_string_lossy(),
        );
        std::fs::write(&temp_script, &script)?;

        // Run the script elevated via Start-Process -Verb RunAs
        // CREATE_NO_WINDOW = 0x08000000
        let status = std::process::Command::new("powershell")
            .creation_flags(0x08000000)
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','{}'",
                    temp_script.to_string_lossy()
                ),
            ])
            .status()?;

        // Cleanup
        let _ = std::fs::remove_file(&temp_hosts);
        let _ = std::fs::remove_file(&temp_script);

        if !status.success() {
            anyhow::bail!("UAC elevation was cancelled or failed");
        }

        // Verify
        let written = std::fs::read_to_string(Self::hosts_file_path())?;
        if !written.contains(Self::HOSTS_MARKER_BEGIN) {
            anyhow::bail!("Hosts file update failed — markers not found after write");
        }

        Ok(())
    }

    /// Remove the phpHerd managed block from hosts content
    fn remove_pherd_block(content: &str) -> String {
        let mut result = String::new();
        let mut in_block = false;

        for line in content.lines() {
            if line.trim() == Self::HOSTS_MARKER_BEGIN {
                in_block = true;
                continue;
            }
            if line.trim() == Self::HOSTS_MARKER_END {
                in_block = false;
                continue;
            }
            if !in_block {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Get the list of site names currently in the hosts file
    pub fn get_hosts_entries(tld: &str) -> Vec<String> {
        let hosts_path = Self::hosts_file_path();
        let content = std::fs::read_to_string(&hosts_path).unwrap_or_default();
        let suffix = format!(".{}", tld);

        let mut entries = Vec::new();
        let mut in_block = false;

        for line in content.lines() {
            if line.trim() == Self::HOSTS_MARKER_BEGIN {
                in_block = true;
                continue;
            }
            if line.trim() == Self::HOSTS_MARKER_END {
                in_block = false;
                continue;
            }
            if in_block {
                // Parse "127.0.0.1    name.test"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let domain = parts[1];
                    if let Some(name) = domain.strip_suffix(&suffix) {
                        entries.push(name.to_string());
                    }
                }
            }
        }

        entries
    }
}
