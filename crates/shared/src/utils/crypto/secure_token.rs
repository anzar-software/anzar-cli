use crate::error::{CoreError, InternalError, Result};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::TryRngCore;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct SecureToken {
    size: usize,
}

impl Default for SecureToken {
    fn default() -> Self {
        Self::with_size32()
    }
}

impl SecureToken {
    pub fn with_size32() -> Self {
        Self { size: 32 }
    }
    pub fn with_size64() -> Self {
        Self { size: 64 }
    }
}

impl SecureToken {
    pub fn generate(&self) -> Result<String> {
        let mut bytes = vec![0u8; self.size];

        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| CoreError::Internal(InternalError::Hashing))?;

        Ok(BASE64_URL_SAFE_NO_PAD.encode(&bytes))
    }
    pub fn hash(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    pub fn verify(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (a, b)| acc | (a ^ b))
            == 0
    }
}
