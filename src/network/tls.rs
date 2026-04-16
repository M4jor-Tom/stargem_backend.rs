use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer};
use rustls::{KeyLogFile, ServerConfig};
use std::fs;
use std::io::Read;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Failed to read certificate file: {0}")]
    CertificateRead(#[from] std::io::Error),
    #[error("Failed to parse certificate: {0}")]
    CertificateParse(String),
    #[error("Failed to build TLS config: {0}")]
    ConfigBuild(String),
}

pub struct TlsConfig {
    pub server_config: Arc<ServerConfig>,
}

impl TlsConfig {
    pub fn from_pem_files(cert_path: &str, key_path: &str) -> Result<Self, TlsError> {
        let mut cert_file = fs::File::open(cert_path)?;
        let mut cert_data = Vec::new();
        cert_file.read_to_end(&mut cert_data)?;

        let mut key_file = fs::File::open(key_path)?;
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data)?;

        Self::from_pem(&cert_data, &key_data)
    }

    pub fn from_pem(cert_data: &[u8], key_data: &[u8]) -> Result<Self, TlsError> {
        let certs = pem_parse_certs(cert_data)?;
        let private_key = pem_parse_key(key_data)?;

        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, private_key)
            .map_err(|e: rustls::Error| TlsError::ConfigBuild(e.to_string()))?;

        config.key_log = Arc::new(KeyLogFile::new());
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(Self {
            server_config: Arc::new(config),
        })
    }
}

fn pem_parse_certs(data: &[u8]) -> Result<Vec<CertificateDer<'static>>, TlsError> {
    let pem_str = String::from_utf8_lossy(data);
    let mut certs = Vec::new();

    for section in pem_str.split("-----BEGIN") {
        if section.trim().is_empty() {
            continue;
        }
        let full_section = format!("-----BEGIN{}", section);
        let encoded = full_section
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<String>();

        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
            .map_err(|e| TlsError::CertificateParse(e.to_string()))?;

        certs.push(CertificateDer::from(decoded));
    }

    if certs.is_empty() {
        return Err(TlsError::CertificateParse("No certificates found".into()));
    }

    Ok(certs)
}

fn pem_parse_key(data: &[u8]) -> Result<PrivateKeyDer<'static>, TlsError> {
    let pem_str = String::from_utf8_lossy(data);

    for section in pem_str.split("-----BEGIN") {
        if section.trim().is_empty() {
            continue;
        }
        let full_section = format!("-----BEGIN{}", section);

        let encoded = full_section
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<String>();

        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &encoded)
            .map_err(|e| TlsError::CertificateParse(e.to_string()))?;

        if full_section.contains("RSA PRIVATE KEY") {
            return Ok(PrivateKeyDer::from(PrivatePkcs1KeyDer::from(decoded)));
        } else if full_section.contains("PRIVATE KEY") {
            return Ok(PrivateKeyDer::from(PrivatePkcs8KeyDer::from(decoded)));
        }
    }

    Err(TlsError::CertificateParse("No private key found".into()))
}
