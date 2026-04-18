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
}
