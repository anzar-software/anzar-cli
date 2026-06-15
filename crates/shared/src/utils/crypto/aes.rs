use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use secrecy::{ExposeSecret, SecretString};

use crate::error::{CoreError, InternalError, Result};

#[derive(Clone, Default)]
pub struct Aes {
    pub secret_key: String,
}

impl Aes {
    pub fn new(secret_key: &SecretString) -> Self {
        Self {
            secret_key: secret_key.expose_secret().to_string(),
        }
    }
}

impl Aes {
    pub fn encrypt(self, msg: &str) -> Result<String> {
        let nonce = Aes256Gcm::generate_nonce(OsRng); // 96-bit / 12 bytes

        let key_bytes = hex::decode(&self.secret_key)?;
        let key: &[u8; 32] = key_bytes.as_slice().try_into().map_err(|_| {
            CoreError::Internal(InternalError::Crypto("Invalid key length - ".to_string()))
        })?;

        let cipher = Aes256Gcm::new(key.into());

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, msg.as_bytes())
            .map_err(|_| CoreError::Internal(crate::error::InternalError::Hashing))?;

        // Prepend nonce to ciphertext, then base64-encode for valid UTF-8
        let mut signature = Vec::with_capacity(12 + ciphertext.len());
        signature.extend_from_slice(&nonce[..]); // fix 1: index instead of .as_slice()
        signature.extend_from_slice(&ciphertext);

        Ok(BASE64_URL_SAFE_NO_PAD.encode(&signature))
    }

    pub fn decrypt(self, ciphertext: &str) -> Result<String> {
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(ciphertext.as_bytes())
            .map_err(|_| CoreError::Internal(crate::error::InternalError::Hashing))?;

        let (nonce_bytes, ciphertext_bytes) = bytes.split_at(12);
        let nonce = Nonce::from(
            <[u8; 12]>::try_from(nonce_bytes)
                .map_err(|_| CoreError::Internal(crate::error::InternalError::Hashing))?,
        );

        let key_bytes = hex::decode(&self.secret_key)?;
        let key: &[u8; 32] = key_bytes.as_slice().try_into().map_err(|_| {
            CoreError::Internal(InternalError::Crypto("Invalid key length - ".to_string()))
        })?;

        let cipher = Aes256Gcm::new(key.into());

        let data = cipher
            .decrypt(&nonce, ciphertext_bytes)
            .map_err(|_| CoreError::Internal(crate::error::InternalError::Hashing))?;
        String::from_utf8(data)
            .map_err(|_| CoreError::Internal(crate::error::InternalError::Hashing))
    }
}
