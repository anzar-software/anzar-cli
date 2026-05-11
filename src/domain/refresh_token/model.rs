use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::super::serde::{
    deserialize_object_id, deserialize_object_id_as_string, serialize_object_id_as_string,
};

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
}

impl RefreshToken {
    pub fn new(jti: &str) -> Self {
        RefreshToken {
            jti: jti.to_string(),
            issued_at: chrono::Utc::now(),
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
pub struct IssuedTokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "refreshToken")]
    pub refresh_token: String,

    #[serde(rename = "refreshTokenJti")]
    pub refresh_token_jti: String,
}
impl IssuedTokens {
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
