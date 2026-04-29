use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::{certs, private_key};

use crate::error::{Error, InternalError};
use std::fs::File;
use std::io::BufReader;

pub fn configure_tls(cert: &str, key: &str) -> Result<rustls::ServerConfig, Error> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cert_file = File::open(cert)?;
    let cert_reader = &mut BufReader::new(cert_file);
    let cert_chain: Vec<CertificateDer> = certs(cert_reader).collect::<Result<_, _>>()?;
    if cert_chain.is_empty() {
        return Err(Error::Internal(InternalError::Tls {
            path: cert.to_string(),
        }));
    }

    // Load private key
    let key_file = File::open(key)?;
    let key_reader = &mut BufReader::new(key_file);
    let key = private_key(key_reader)?.ok_or_else(|| {
        Error::Internal(InternalError::Tls {
            path: cert.to_string(),
        })
    })?;

    // Build TLS config
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|_| {
            Error::Internal(InternalError::Tls {
                path: cert.to_string(),
            })
        })
}
