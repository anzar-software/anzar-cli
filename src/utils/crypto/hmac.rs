use crate::error::{Error, InternalError, Result};

use super::secure_token::SecureToken;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use hmac::Mac;
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;

#[derive(Clone, Default)]
pub struct HmacSigner {
    pub secret_key: String,
}

impl HmacSigner {
    pub fn new(secret_key: &SecretString) -> Self {
        Self {
            secret_key: secret_key.expose_secret().to_string(),
        }
    }
}

impl HmacSigner {
    pub fn issue(&self, id: &str) -> Result<String> {
        let nonce = SecureToken::with_size32().generate()?;
        let message = format!("{}|{}", id, nonce);

        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(self.secret_key.as_bytes())
            .map_err(|_| Error::Internal(InternalError::Hashing))?;
        mac.update(message.as_bytes());

        let signature = BASE64_URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        Ok(format!("{}|{}", message, signature))
    }

    pub fn validate(&self, cookie_value: &str) -> Option<bool> {
        let parts: Vec<&str> = cookie_value.split('|').collect();
        if parts.len() != 3 {
            return None;
        }

        let (user_id, nonce, signature) = (parts[0], parts[1], parts[2]);

        let message = format!("{}|{}", user_id, nonce);
        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes()).ok()?;
        mac.update(message.as_bytes());
        let expected = BASE64_URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

        Some(self.verify(&expected, signature))
    }

    fn verify(&self, a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (a, b)| acc | (a ^ b))
            == 0
    }
    // use subtle::ConstantTimeEq;
    //
    // fn verify(&self, a: &str, b: &str) -> bool {
    //     let a_bytes = a.as_bytes();
    //     let b_bytes = b.as_bytes();
    //
    //     // Constant-time comparison
    //     a_bytes.ct_eq(b_bytes).into()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_cookie() {
        let dc = HmacSigner::new(&SecretString::from("supersecretkey"));
        let cookie = dc.issue("alice").unwrap_or_default();

        assert!(dc.validate(&cookie).unwrap_or_default());
        assert!(!dc.validate("alice,wrongnonce,badsig").unwrap_or_default());
    }
}
