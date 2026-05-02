mod hmac;
mod jwt;
mod password_hasher;
mod secure_token;

use std::sync::Arc;

use hmac::HmacSigner;
use secrecy::SecretString;

pub use jwt::JwtSigner;
pub use password_hasher::{Argon2Password, BcryptPassword, Hashable};
pub use secure_token::SecureToken;

use crate::error::{Error, InternalError, Result};

#[derive(Clone)]
pub struct Crypto {
    pub password_hasher: Arc<dyn Hashable>,
    pub token: SecureToken,
    pub hmac: HmacSigner,
    jwt: Option<JwtSigner>,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            password_hasher: Arc::new(Argon2Password),
            token: SecureToken::default(),
            hmac: HmacSigner::default(),
            jwt: None,
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
}

impl Crypto {
    pub fn with_hmac_secret(mut self, key: &str) -> Self {
        self.hmac = HmacSigner::new(&SecretString::from(key));
        self
    }
}

impl Crypto {
    pub fn with_token_size32(mut self) -> Self {
        self.token = SecureToken::with_size32();
        self
    }
    pub fn with_token_size64(mut self) -> Self {
        self.token = SecureToken::with_size64();
        self
    }
}

impl Crypto {
    pub fn with_jwt(mut self, config: crate::config::AnzarConfiguration) -> Self {
        self.jwt = Some(JwtSigner::new(config));
        self
    }
}

impl Crypto {
    pub fn validate(&self) -> Result<()> {
        if self.jwt.is_none() {
            return Err(Error::Internal(InternalError::MissingConfiguration(
                "JwtSigner".into(),
            )));
        }
        if self.hmac.secret_key.is_empty() {
            return Err(Error::Internal(InternalError::MissingConfiguration(
                "HmacSigner".into(),
            )));
        }
        Ok(())
    }

    pub fn jwt(&self) -> Result<&JwtSigner> {
        self.jwt
            .as_ref()
            .ok_or(Error::Internal(InternalError::MissingConfiguration(
                "JwtSigner".to_string(),
            )))
    }
}
