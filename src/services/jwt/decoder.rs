use jsonwebtoken::DecodingKey;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Validation, decode};
use serde::de::DeserializeOwned;

use crate::error::{Error, InvalidTokenReason, Result, TokenErrorType};

pub struct JwtDecoder {
    token: String,
    decoding_key: DecodingKey,
}
impl JwtDecoder {
    pub fn new(token: impl Into<String>, secret: &[u8]) -> Self {
        Self {
            token: token.into(),
            decoding_key: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
}

impl JwtDecoder {
    pub fn decode<C: DeserializeOwned>(&self) -> Result<C> {
        decode::<C>(&self.token, &self.decoding_key, &Validation::default())
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::InvalidSignature => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: InvalidTokenReason::InvalidSignature,
                },
                ErrorKind::ExpiredSignature => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: InvalidTokenReason::Expired,
                },
                _ => Error::InvalidToken {
                    token_type: TokenErrorType::Token,
                    reason: InvalidTokenReason::Unknown,
                },
            })
    }
}
