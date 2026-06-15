use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::de::DeserializeOwned;

use crate::application::model::Claims;
use crate::config::JwtConfig;
use crate::domain::model::SigningKeys;
use crate::error::{AuthError, CoreError, Result, TokenErrorType};

#[derive(Clone)]
pub struct JwtSigner {
    keys: Vec<SigningKeys>,
    jwt_config: JwtConfig,
}

impl JwtSigner {
    pub fn new(keys: Vec<SigningKeys>, jwt_config: &JwtConfig) -> Self {
        Self {
            keys,
            jwt_config: jwt_config.clone(),
        }
    }
}

impl JwtSigner {
    fn load_encodingkey(&self, signing_key: &SigningKeys) -> Result<EncodingKey> {
        let prv_key = BASE64_URL_SAFE_NO_PAD
            .decode(&signing_key.private_key)
            .unwrap();

        match signing_key.key.algorithm.as_str() {
            "EdDSA" => EncodingKey::from_ed_pem(&prv_key).map_err(Into::into),
            "ES256" | "ES384" => EncodingKey::from_ec_pem(&prv_key).map_err(Into::into),
            "RS256" | "RS384" | "RS512" | "PS256" | "PS384" | "PS512" => {
                EncodingKey::from_rsa_pem(&prv_key).map_err(Into::into)
            }
            &_ => EncodingKey::from_rsa_pem(&prv_key).map_err(Into::into),
        }
    }
    pub fn encode(&self, claims: Claims) -> Result<String> {
        let active_key = self.keys.iter().find(|k| k.key.status == "active").unwrap();

        let mut header = jsonwebtoken::Header::new(self.jwt_config.algorithm.clone().into());
        header.kid = Some(active_key.key.kid.clone());

        // If your key is in PEM format, it is better performance wise to generate
        // the EncodingKey once in a lazy_static or something similar and reuse it.
        let encoding_secret = self.load_encodingkey(active_key)?;

        let token = jsonwebtoken::encode(&header, &claims, &encoding_secret)?;
        Ok(token)
    }

    fn load_decodingkey(&self, kid: &str) -> Result<DecodingKey> {
        let signing_key = self.keys.iter().find(|k| k.key.kid == kid).unwrap();
        let pub_key = BASE64_URL_SAFE_NO_PAD
            .decode(&signing_key.key.public_key)
            .unwrap();

        match signing_key.key.algorithm.as_str() {
            "EdDSA" => DecodingKey::from_ed_pem(&pub_key).map_err(Into::into),
            "ES256" | "ES384" => DecodingKey::from_ec_pem(&pub_key).map_err(Into::into),
            "RS256" | "RS384" | "RS512" | "PS256" | "PS384" | "PS512" => {
                DecodingKey::from_rsa_pem(&pub_key).map_err(Into::into)
            }
            &_ => DecodingKey::from_rsa_pem(&pub_key).map_err(Into::into),
        }
    }
    pub fn decode<C: DeserializeOwned>(&self, token: &str) -> Result<C> {
        let mut validation =
            jsonwebtoken::Validation::new(self.jwt_config.algorithm.clone().into());
        validation.set_audience(&[&self.jwt_config.audience]);
        validation.set_issuer(&[&self.jwt_config.issuer]);

        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.unwrap_or("".to_string());

        let decoding_key = self.load_decodingkey(&kid)?;
        jsonwebtoken::decode::<C>(&token, &decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::InvalidSignature => {
                    CoreError::Unauthenticated(AuthError::TokenInvalidSignature {
                        token_type: TokenErrorType::Token,
                    })
                }
                ErrorKind::ExpiredSignature => {
                    CoreError::Unauthenticated(AuthError::TokenExpired {
                        token_type: TokenErrorType::Token,
                        expired_at: chrono::Utc::now(),
                    })
                }
                ErrorKind::InvalidAudience => {
                    CoreError::Unauthenticated(AuthError::TokenInvalidAudience {
                        token_type: TokenErrorType::Token,
                    })
                }
                ErrorKind::InvalidIssuer => {
                    CoreError::Unauthenticated(AuthError::TokenInvalidIssuer {
                        token_type: TokenErrorType::Token,
                    })
                }
                ErrorKind::InvalidAlgorithm | ErrorKind::MissingAlgorithm => {
                    CoreError::Unauthenticated(AuthError::TokenInvalidAlgorithm {
                        token_type: TokenErrorType::Token,
                    })
                }
                _ => CoreError::Unauthenticated(AuthError::TokenInvalid {
                    token_type: TokenErrorType::Token,
                }),
            })
    }
}
