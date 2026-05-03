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

use crate::{
    config::{AnzarConfiguration, AuthStrategy, HashingAlgorithm},
    error::{Error, InternalError, Result},
};

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
    pub fn validate(self, strategy: &AuthStrategy) -> Result<Self> {
        if matches!(strategy, AuthStrategy::Jwt(..)) && self.jwt.is_none() {
            return Err(Error::Internal(InternalError::MissingConfiguration(
                "JWT strategy requires a JWT signer, but none was configured".into(),
            )));
        }

        match self.hmac.secret_key.len() {
            0 => {
                return Err(Error::Internal(InternalError::MissingConfiguration(
                    "HMAC secret key is missing".into(),
                )));
            }
            n if n < 32 => {
                return Err(Error::Internal(InternalError::MissingConfiguration(
                    format!("HMAC secret key is too short ({n} bytes), minimum is 32 bytes"),
                )));
            }
            _ => {}
        }

        Ok(self)
    }

    pub fn jwt(&self) -> Result<&JwtSigner> {
        self.jwt
            .as_ref()
            .ok_or(Error::Internal(InternalError::MissingConfiguration(
                "JwtSigner".to_string(),
            )))
    }
}

impl Crypto {
    pub fn from_configuration(configuration: &AnzarConfiguration) -> Result<Self> {
        let base = match configuration.auth.password.algorithm {
            HashingAlgorithm::Argon2 { .. } => Crypto::with_argon(),
            HashingAlgorithm::Bcrypt { .. } => Crypto::with_bcrypt(),
        }
        .with_hmac_secret(&configuration.security.secret_key)
        .with_token_size64();

        match configuration.auth.strategy {
            AuthStrategy::Jwt(..) => base.with_jwt(configuration.clone()),
            _ => base,
        }
        .validate(&configuration.auth.strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_crypto() -> Crypto {
        Crypto::with_argon()
            .with_hmac_secret(&"a".repeat(32))
            .with_token_size64()
    }

    #[test]
    fn test_valid_session_strategy() {
        let crypto = base_crypto();
        assert!(
            crypto
                .validate(&AuthStrategy::Session(crate::config::SessionConfig {
                    ..Default::default()
                }))
                .is_ok()
        );
    }
}
