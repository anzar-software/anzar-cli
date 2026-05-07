use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::super::serde::{deserialize_object_id, deserialize_object_id_as_string};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow)]
pub struct EmailVerificationToken {
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
        // serialize_with = "serialize_object_id_as_string",
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

    pub token: String,
}

impl Default for EmailVerificationToken {
    fn default() -> Self {
        Self {
            id: None,
            user_id: String::default(),
            token: String::default(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(86400),
            used_at: None,
        }
    }
}
impl EmailVerificationToken {
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.into();
        self
    }
    pub fn with_token_hash(mut self, hash: &str) -> Self {
        self.token = hash.into();
        self
    }
    pub fn with_expiray(mut self, expires_at: chrono::Duration) -> Self {
        self.expires_at = Utc::now() + expires_at;
        self
    }
}

impl EmailVerificationToken {
    pub fn id(&self) -> Result<&str, crate::error::Error> {
        self.id.as_deref().ok_or_else(|| {
            tracing::error!(
                error_code = "ValidationError::Malformed",
                "Unexpected null/missing data"
            );
            crate::error::Error::Validation(crate::error::ValidationError::Malformed {
                field: crate::error::CredentialField::ObjectId,
            })
        })
    }
}
