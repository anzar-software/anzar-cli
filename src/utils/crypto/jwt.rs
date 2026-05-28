use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::de::DeserializeOwned;

use crate::application::model::Claims;
use crate::config::JwtConfig;
use crate::domain::model::SigningKeys;
use crate::error::{AuthError, Error, Result, TokenErrorType};

#[derive(Clone)]
pub struct JwtSigner {
    jwt_config: JwtConfig,
    private: String,
    signing_key: SigningKeys,
}

impl JwtSigner {
    pub fn new(private: &str, signing_key: &SigningKeys, jwt_config: &JwtConfig) -> Self {
        Self {
            jwt_config: jwt_config.clone(),
            private: private.into(),
            signing_key: signing_key.clone(),
        }
    }
}

impl JwtSigner {
    fn load_keys(&self) -> (Result<EncodingKey>, Result<DecodingKey>) {
        let prv_key = BASE64_URL_SAFE_NO_PAD.decode(&self.private).unwrap();
        let pub_key = BASE64_URL_SAFE_NO_PAD
            .decode(&self.signing_key.public_key)
            .unwrap();

        match self.signing_key.algorithm.as_str() {
            "EdDSA" => (
                EncodingKey::from_ed_pem(&prv_key).map_err(Into::into),
                DecodingKey::from_ed_pem(&pub_key).map_err(Into::into),
            ),
            "ES256" | "ES384" => (
                EncodingKey::from_ec_pem(&prv_key).map_err(Into::into),
                DecodingKey::from_ec_pem(&pub_key).map_err(Into::into),
            ),
            "RS256" | "RS384" | "RS512" | "PS256" | "PS384" | "PS512" => (
                EncodingKey::from_rsa_pem(&prv_key).map_err(Into::into),
                DecodingKey::from_rsa_pem(&pub_key).map_err(Into::into),
            ),
            &_ => (
                EncodingKey::from_rsa_pem(&prv_key).map_err(Into::into),
                DecodingKey::from_rsa_pem(&pub_key).map_err(Into::into),
            ),
        }
    }

    pub fn encode(&self, claims: Claims) -> Result<String> {
        let mut header = jsonwebtoken::Header::new(self.jwt_config.algorithm.clone().into());
        header.kid = Some(self.signing_key.kid.clone());

        // If your key is in PEM format, it is better performance wise to generate
        // the EncodingKey once in a lazy_static or something similar and reuse it.
        let encoding_secret = self.load_keys().0?;

        let token = jsonwebtoken::encode(&header, &claims, &encoding_secret)?;
        Ok(token)
    }

    pub fn decode<C: DeserializeOwned>(&self, token: &str) -> Result<C> {
        let mut validation =
            jsonwebtoken::Validation::new(self.jwt_config.algorithm.clone().into());
        validation.set_audience(&[self.jwt_config.clone().audience]);
        validation.set_issuer(&[self.jwt_config.clone().issuer]);

        let decoding_key = self.load_keys().1?;
        jsonwebtoken::decode::<C>(&token, &decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::InvalidSignature => {
                    Error::Unauthenticated(AuthError::TokenInvalidSignature {
                        token_type: TokenErrorType::Token,
                    })
                }
                ErrorKind::ExpiredSignature => Error::Unauthenticated(AuthError::TokenExpired {
                    token_type: TokenErrorType::Token,
                    expired_at: chrono::Utc::now(),
                }),
                ErrorKind::InvalidAudience => {
                    Error::Unauthenticated(AuthError::TokenInvalidAudience {
                        token_type: TokenErrorType::Token,
                    })
                }
                ErrorKind::InvalidIssuer => Error::Unauthenticated(AuthError::TokenInvalidIssuer {
                    token_type: TokenErrorType::Token,
                }),
                ErrorKind::InvalidAlgorithm | ErrorKind::MissingAlgorithm => {
                    Error::Unauthenticated(AuthError::TokenInvalidAlgorithm {
                        token_type: TokenErrorType::Token,
                    })
                }
                _ => Error::Unauthenticated(AuthError::TokenInvalid {
                    token_type: TokenErrorType::Token,
                }),
            })
    }
}
