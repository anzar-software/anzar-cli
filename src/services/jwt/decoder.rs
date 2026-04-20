use jsonwebtoken::DecodingKey;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Validation, decode};
use serde::de::DeserializeOwned;

use crate::config::AnzarConfiguration;
use crate::error::{AuthError, Error, Result, TokenErrorType};

pub struct JwtDecoder {
    token: String,
    decoding_key: DecodingKey,
    validation: Validation,
}
impl JwtDecoder {
    pub fn new(token: impl Into<String>, configuration: &AnzarConfiguration) -> Self {
        let mut validation =
            jsonwebtoken::Validation::new(configuration.auth.jwt.algorithm.clone().into());
        validation.set_audience(&[configuration.auth.jwt.clone().audience]);
        validation.set_issuer(&[configuration.auth.jwt.clone().issuer]);

        Self {
            token: token.into(),
            decoding_key: jsonwebtoken::DecodingKey::from_secret(
                configuration.security.secret_key.as_bytes(),
            ),
            validation,
        }
    }
}

impl JwtDecoder {
    pub fn decode<C: DeserializeOwned>(&self) -> Result<C> {
        decode::<C>(&self.token, &self.decoding_key, &self.validation)
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
