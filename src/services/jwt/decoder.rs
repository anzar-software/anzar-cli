use jsonwebtoken::DecodingKey;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Validation, decode};
use serde::de::DeserializeOwned;

use crate::config::AnzarConfiguration;
use crate::error::{Error, Reason, Result, TokenErrorType};

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
                ErrorKind::InvalidSignature => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::InvalidSignature,
                },
                ErrorKind::ExpiredSignature => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::Expired,
                },
                ErrorKind::InvalidAudience => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::InvalidAudience,
                },
                ErrorKind::InvalidIssuer => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::InvalidIssuer,
                },
                ErrorKind::InvalidAlgorithm | ErrorKind::MissingAlgorithm => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::InvalidAlgorithm,
                },
                _ => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: Reason::Unknown,
                },
            })
    }
}
