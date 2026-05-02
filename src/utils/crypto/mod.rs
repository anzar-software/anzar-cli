mod hmac;
mod password_hasher;
mod secure_token;

use std::sync::Arc;

use hmac::HmacSigner;
use secrecy::SecretString;

pub use password_hasher::{Argon2Password, BcryptPassword, Hashable};
pub use secure_token::SecureToken;

#[derive(Clone)]
pub struct Crypto {
    pub password_hasher: Arc<dyn Hashable>,
    pub token: SecureToken,
    pub hmac: HmacSigner,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            password_hasher: Arc::new(Argon2Password),
            token: SecureToken::default(),
            hmac: HmacSigner::default(),
        }
    }
}

impl Crypto {
    pub fn with_argon() -> Self {
        Self::default()
    }
    pub fn with_bcrypt() -> Self {
        Self {
            password_hasher: Arc::new(BcryptPassword),
            ..Self::default()
        }
    }

    pub fn with_hmac_secret(mut self, key: &str) -> Self {
        self.hmac = HmacSigner::new(&SecretString::from(key));
        self
    }

    pub fn with_token_size32(mut self) -> Self {
        self.token = SecureToken::with_size32();
        self
    }
    pub fn with_token_size64(mut self) -> Self {
        self.token = SecureToken::with_size64();
        self
    }
}
