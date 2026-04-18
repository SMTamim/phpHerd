use crate::core::config::AppConfig;
use anyhow::Result;
use std::path::PathBuf;

pub struct SslManager;

impl SslManager {
    pub fn ca_dir() -> PathBuf {
        AppConfig::data_dir().join("config").join("ssl").join("ca")
    }

    pub fn certs_dir() -> PathBuf {
        AppConfig::data_dir()
            .join("config")
            .join("ssl")
            .join("certs")
    }

    pub fn ca_cert_path() -> PathBuf {
        Self::ca_dir().join("phpHerdCA.pem")
    }

    pub fn ca_key_path() -> PathBuf {
        Self::ca_dir().join("phpHerdCA.key")
    }

    /// Generate the root CA certificate if it doesn't exist
    pub fn generate_ca() -> Result<()> {
        let cert_path = Self::ca_cert_path();
        let key_path = Self::ca_key_path();

        if cert_path.exists() && key_path.exists() {
            tracing::info!("CA already exists");
            return Ok(());
        }

        std::fs::create_dir_all(Self::ca_dir())?;

        let mut params = rcgen::CertificateParams::new(Vec::new())?;
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "phpHerd CA");
        params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "phpHerd");

        let key_pair = rcgen::KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        std::fs::write(&cert_path, cert.pem())?;
        std::fs::write(&key_path, key_pair.serialize_pem())?;

        tracing::info!("Generated CA certificate at {:?}", cert_path);
        Ok(())
    }

    /// Generate a certificate for a site signed by our CA
    pub fn generate_site_cert(site_name: &str, tld: &str) -> Result<(PathBuf, PathBuf)> {
        let domain = format!("{}.{}", site_name, tld);
        let cert_path = Self::certs_dir().join(format!("{}.crt", domain));
        let key_path = Self::certs_dir().join(format!("{}.key", domain));

        std::fs::create_dir_all(Self::certs_dir())?;

        // Load CA key and re-generate CA cert params for signing
        let ca_key_pem = std::fs::read_to_string(Self::ca_key_path())?;
        let ca_key = rcgen::KeyPair::from_pem(&ca_key_pem)?;

        let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new())?;
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "phpHerd CA");
        let ca_cert = ca_params.self_signed(&ca_key)?;

        // Generate site certificate
        let subject_alt_names = vec![domain.clone()];
        let mut params = rcgen::CertificateParams::new(subject_alt_names)?;
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, &domain);

        let site_key = rcgen::KeyPair::generate()?;
        let site_cert = params.signed_by(&site_key, &ca_cert, &ca_key)?;

        std::fs::write(&cert_path, site_cert.pem())?;
        std::fs::write(&key_path, site_key.serialize_pem())?;

        tracing::info!("Generated certificate for {} at {:?}", domain, cert_path);
        Ok((cert_path, key_path))
    }

    /// Check if CA certificate exists
    pub fn ca_exists() -> bool {
        Self::ca_cert_path().exists() && Self::ca_key_path().exists()
    }

    /// Check if a site has a certificate
    pub fn site_cert_exists(site_name: &str, tld: &str) -> bool {
        let domain = format!("{}.{}", site_name, tld);
        Self::certs_dir().join(format!("{}.crt", domain)).exists()
    }
}
