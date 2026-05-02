use jsonwebtoken::errors::ErrorKind;
use serde::de::DeserializeOwned;

use crate::config::AnzarConfiguration;
use crate::error::{AuthError, Error, Result, TokenErrorType};
use crate::extractors::Claims;

#[derive(Clone)]
pub struct JwtSigner {
    configuration: AnzarConfiguration,
}

impl JwtSigner {
    pub fn new(configuration: AnzarConfiguration) -> Self {
        Self { configuration }
    }
}

impl JwtSigner {
    pub fn encode(&self, claims: Claims) -> Result<String> {
        let header =
            jsonwebtoken::Header::new(self.configuration.auth.jwt.algorithm.clone().into());
        let encoding_secret = jsonwebtoken::EncodingKey::from_secret(
            self.configuration.security.secret_key.as_bytes(),
        );

        let token = jsonwebtoken::encode(&header, &claims, &encoding_secret)?;
        Ok(token)
    }

    pub fn decode<C: DeserializeOwned>(&self, token: &str) -> Result<C> {
        let jwt_config = &self.configuration.auth.jwt;

        let mut validation = jsonwebtoken::Validation::new(jwt_config.algorithm.clone().into());
        validation.set_audience(&[jwt_config.clone().audience]);
        validation.set_issuer(&[jwt_config.clone().issuer]);

        let decoding_key = jsonwebtoken::DecodingKey::from_secret(
            self.configuration.security.secret_key.as_bytes(),
        );

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
