mod aes;
mod hmac;
mod jwt;
mod openssl;
mod password_hasher;
mod secure_token;

use std::sync::{Arc, RwLock};

use hmac::HmacSigner;
use secrecy::SecretString;

pub use aes::Aes;
pub use jwt::JwtSigner;
pub use openssl::Openssl;
pub use password_hasher::{Argon2Password, BcryptPassword, Hashable};
pub use secure_token::SecureToken;

use crate::{
    config::{AnzarConfiguration, AuthStrategy, HashingAlgorithm, JwtConfig},
    domain::model::SigningKeys,
    error::{AuthError, CoreError, InternalError, Result},
};

#[derive(Clone)]
pub struct Crypto {
    pub password_hasher: Arc<dyn Hashable>,
    pub token: SecureToken,
    pub hmac: HmacSigner,
    pub aes: Aes,
    pub openssl: Openssl,
    jwt: Arc<RwLock<Option<JwtSigner>>>,
}

impl Default for Crypto {
    fn default() -> Self {
        Self {
            password_hasher: Arc::new(Argon2Password::default()),
            token: SecureToken::default(),
            hmac: HmacSigner::default(),
            aes: Aes::default(),
            openssl: Openssl::default(),
            jwt: Arc::new(RwLock::new(None)),
        }
    }
}

impl Crypto {
    pub fn with_argon(m_cost: u32, t_cost: u32, p_cost: u32) -> Self {
        Self {
            password_hasher: Arc::new(Argon2Password::new(m_cost, t_cost, p_cost)),
            ..Self::default()
        }
    }
    pub fn with_bcrypt(cost: u32) -> Self {
        Self {
            password_hasher: Arc::new(BcryptPassword::new(cost)),
            ..Self::default()
        }
    }
}

impl Crypto {
    pub fn with_initializations(mut self, configuration: &AnzarConfiguration) -> Self {
        let key = &configuration.security.secret_key;

        self.hmac = HmacSigner::new(&SecretString::from(key.clone()));
        self.aes = Aes::new(&SecretString::from(key.clone()));

        if let Ok(conf) = configuration.auth.jwt() {
            self.openssl = Openssl::new(&conf.algorithm);
        }
        self
    }
}

impl Crypto {
    pub fn with_jwt(mut self, keys: Vec<SigningKeys>, jwt_config: &JwtConfig) -> Self {
        self.jwt = Arc::new(RwLock::new(Some(JwtSigner::new(keys, jwt_config))));
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
    pub fn from_configuration(configuration: &AnzarConfiguration) -> Self {
        match configuration.auth.password.algorithm {
            HashingAlgorithm::Argon2 {
                memory_kib,
                iterations,
                parallelism,
            } => Crypto::with_argon(memory_kib, iterations, parallelism),
            HashingAlgorithm::Bcrypt { cost } => Crypto::with_bcrypt(cost),
        }
        .with_initializations(configuration)
        .with_token_size64()
    }
}

impl Crypto {
    pub fn validate(self, strategy: &AuthStrategy) -> Result<Self> {
        if matches!(strategy, AuthStrategy::Jwt(..)) && self.jwt().is_err() {
            return Err(CoreError::Internal(InternalError::MissingConfiguration(
                "JWT strategy requires a JWT signer, but none was configured".into(),
            )));
        }

        match self.hmac.secret_key.len() {
            0 => {
                return Err(CoreError::Internal(InternalError::MissingConfiguration(
                    "HMAC secret key is missing".into(),
                )));
            }
            n if n < 32 => {
                return Err(CoreError::Internal(InternalError::MissingConfiguration(
                    format!("HMAC secret key is too short ({n} bytes), minimum is 32 bytes"),
                )));
            }
            _ => {}
        }

        Ok(self)
    }

    pub fn jwt(&self) -> Result<JwtSigner> {
        let jwt = self.jwt.read().unwrap();

        jwt.clone()
            .ok_or(CoreError::Unauthenticated(AuthError::JwtNotConfigured))
    }

    pub fn rotate_jwt(&self, new_signer: JwtSigner) {
        let mut jwt = self.jwt.write().unwrap();
        *jwt = Some(new_signer);
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::config::RateLimit;
//
//     use super::*;
//
//     fn base_crypto() -> Crypto {
//         pub const DEFAULT_M_COST: u32 = 19 * 1024; // ~19 MiB
//         pub const DEFAULT_T_COST: u32 = 2;
//         pub const DEFAULT_P_COST: u32 = 1;
//
//         Crypto::with_argon(DEFAULT_M_COST, DEFAULT_T_COST, DEFAULT_P_COST)
//             .with_initializations(&mock_configuration())
//             .with_token_size64()
//     }
//     fn bcrypt_crypto() -> Crypto {
//         let cost = 12;
//         Crypto::with_bcrypt(cost)
//             .with_initializations(&mock_configuration())
//             .with_token_size64()
//     }
//     fn mock_configuration() -> AnzarConfiguration {
//         AnzarConfiguration {
//             app: crate::config::App {
//                 environment: "dev".into(),
//                 url: "localhost:3000".to_string(),
//             },
//             database: crate::config::Database {
//                 driver: crate::config::database::DatabaseDriver::PostgreSQL,
//                 connection_string: "postgres://hakou:password@localhost:5432/dev".into(),
//                 cache: crate::config::Cache {
//                     driver: crate::config::cache::CacheDriver::InMemory,
//                     url: "".into(),
//                 },
//             },
//             server: crate::config::Server::default(),
//             auth: crate::config::Authentication {
//                 strategy: AuthStrategy::Jwt(crate::config::JwtConfig {
//                     issuer: "localhost:3000".into(),
//                     audience: "users".into(),
//                     ..Default::default()
//                 }),
//                 ..Default::default()
//             },
//             security: crate::config::Security {
//                 secret_key: "a".repeat(32),
//                 rate_limit: RateLimit::default(),
//                 headers: vec![],
//             },
//         }
//     }
//
//     #[test]
//     fn test_valid_session_strategy() {
//         let crypto = base_crypto();
//         let strategy = &AuthStrategy::Session(crate::config::SessionConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_ok());
//     }
//
//     #[test]
//     fn test_valid_jwt_strategy() {
//         let binding = mock_configuration();
//         let jwt_config = binding.auth.jwt().unwrap();
//         let crypto = base_crypto().with_jwt("", "", jwt_config);
//         let strategy = &AuthStrategy::Jwt(crate::config::JwtConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_ok());
//     }
//     #[test]
//     fn test_valid_session_strategy_with_bcrypt() {
//         let crypto = bcrypt_crypto();
//         let strategy = &AuthStrategy::Session(crate::config::SessionConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_ok());
//     }
//
//     #[test]
//     fn test_valid_jwt_strategy_with_bcrypt() {
//         let binding = mock_configuration();
//         let jwt_config = binding.auth.jwt().unwrap();
//         let crypto = base_crypto().with_jwt("", "", jwt_config);
//         let strategy = &AuthStrategy::Jwt(crate::config::JwtConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_ok());
//     }
//
//     #[test]
//     fn test_jwt_strategy_missing_signer() {
//         let crypto = base_crypto(); // no .with_jwt()
//         let strategy = &AuthStrategy::Jwt(crate::config::JwtConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_err());
//     }
//
//     #[test]
//     fn test_hmac_secret_empty() {
//         pub const DEFAULT_M_COST: u32 = 19 * 1024; // ~19 MiB
//         pub const DEFAULT_T_COST: u32 = 2;
//         pub const DEFAULT_P_COST: u32 = 1;
//         let crypto = Crypto::with_argon(DEFAULT_M_COST, DEFAULT_T_COST, DEFAULT_P_COST)
//             .with_initializations(&mock_configuration())
//             .with_token_size64();
//         let strategy = &AuthStrategy::Session(crate::config::SessionConfig {
//             ..Default::default()
//         });
//
//         assert!(crypto.validate(strategy).is_err());
//     }
//
//     // #[test]
//     // fn test_hmac_secret_too_short() {
//     //     let crypto = Crypto::with_argon()
//     //         .with_hmac_secret("tooshort")
//     //         .with_token_size64();
//     //     let strategy = &AuthStrategy::Session(crate::config::SessionConfig {
//     //         ..Default::default()
//     //     });
//     //
//     //     let err = crypto.validate(strategy).unwrap_err();
//     //     assert!(err.to_string().contains("too short"));
//     // }
// }
