use crate::utils::{Token, TokenHasher, mongodb_serde::*};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow)]
pub struct RefreshToken {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    #[sqlx(rename = "userId")]
    #[serde(
        rename = "userId",
        default,
        serialize_with = "serialize_object_id_as_string",
        deserialize_with = "deserialize_object_id"
    )]
    pub user_id: String,

    #[sqlx(rename = "issuedAt")]
    #[serde(rename = "issuedAt")]
    pub issued_at: DateTime<Utc>,
    #[sqlx(rename = "expiresAt")]
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
    #[sqlx(rename = "usedAt")]
    #[serde(rename = "usedAt")]
    pub used_at: Option<DateTime<Utc>>,

    pub jti: String,
    pub token: String,
    pub valid: bool,
}

impl RefreshToken {
    pub fn new(tokens: &Tokens) -> Self {
        RefreshToken {
            issued_at: chrono::Utc::now(),
            jti: tokens.refresh_token_jti.to_string(),
            token: Token::hash(&tokens.refresh_token),
            valid: true,
            ..Default::default()
        }
    }
}
impl RefreshToken {
    pub fn with_user_id(mut self, id: &str) -> Self {
        self.user_id = id.into();
        self
    }
    pub fn with_expire_at(mut self, expires_in: i64) -> Self {
        self.expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "refreshToken")]
    pub refresh_token: String,

    #[serde(rename = "refreshTokenJti")]
    pub refresh_token_jti: String,
}
impl Tokens {
    pub fn with_access_token(mut self, access_token: &str) -> Self {
        self.access_token = access_token.into();
        self
    }

    pub fn with_refresh_token(mut self, refresh_token: &str) -> Self {
        self.refresh_token = refresh_token.into();
        self
    }

    pub fn with_jti(mut self, jti: uuid::Uuid) -> Self {
        self.refresh_token_jti = jti.to_string();
        self
    }
}
