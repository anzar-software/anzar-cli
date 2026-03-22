use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::{certs, private_key};

use crate::error::{Error, Reason};
use std::fs::File;
use std::io::BufReader;

pub fn configure_tls(cert: String, key: String) -> Result<rustls::ServerConfig, Error> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Load certificate chain
    let cert_file = File::open(&cert).map_err(|_| Error::TlsConfig {
        path: cert.clone(),
        reason: Reason::NotFound,
    })?;
    let cert_reader = &mut BufReader::new(cert_file);
    let cert_chain: Vec<CertificateDer> = certs(cert_reader).collect::<Result<_, _>>()?;
    if cert_chain.is_empty() {
        return Err(Error::TlsConfig {
            path: cert.clone(),
            reason: Reason::Empty,
        });
    }

    // Load private key
    let key_file = File::open(&key).map_err(|_| Error::TlsConfig {
        path: key.clone(),
        reason: Reason::NotFound,
    })?;
    let key_reader = &mut BufReader::new(key_file);
    let key = private_key(key_reader)?.ok_or_else(|| Error::TlsConfig {
        path: key.clone(),
        reason: Reason::NotFound,
    })?;

    // Build TLS config
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|e| Error::InternalServerError(format!("Failed to build ServerConfig: {}", e)))
}
