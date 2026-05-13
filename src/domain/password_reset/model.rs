use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::domain::query::IntoBsonDocument;

use super::super::serde::{
    deserialize_datetime, deserialize_object_id, deserialize_object_id_as_string,
    deserialize_option_datetime,
};

#[derive(Default, Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"link": String::default(), "expires_at": "2026-02-19T22:42:23.467Z"}))]
pub struct ExpiringLink {
    pub link: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow)]
pub struct PasswordResetToken {
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
    #[serde(rename = "issuedAt", deserialize_with = "deserialize_datetime")]
    pub issued_at: DateTime<Utc>,
    #[sqlx(rename = "expiresAt")]
    #[serde(rename = "expiresAt", deserialize_with = "deserialize_datetime")]
    pub expires_at: DateTime<Utc>,
    #[sqlx(rename = "usedAt")]
    #[serde(rename = "usedAt", deserialize_with = "deserialize_option_datetime")]
    pub used_at: Option<DateTime<Utc>>,

    pub token: String,
}

impl Default for PasswordResetToken {
    fn default() -> Self {
        Self::new()
    }
}
impl PasswordResetToken {
    pub fn new() -> Self {
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
impl PasswordResetToken {
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.into();
        self
    }
    pub fn with_token_hash(mut self, hash: &str) -> Self {
        self.token = hash.into();
        self
    }
    pub fn with_expiray(mut self, expires_at: &chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = *expires_at;
        self
    }
}

impl PasswordResetToken {
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
impl IntoBsonDocument for PasswordResetToken {
    fn into_bson_document(self) -> Result<mongodb::bson::Document, mongodb::bson::ser::Error> {
        let mut doc = mongodb::bson::to_document(&self)?;

        for key in &["issuedAt", "expiresAt", "usedAt"] {
            if let Some(mongodb::bson::Bson::String(s)) = doc.get(*key).cloned() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                    doc.insert(
                        *key,
                        mongodb::bson::Bson::DateTime(mongodb::bson::DateTime::from_millis(
                            dt.timestamp_millis(),
                        )),
                    );
                }
            }
        }

        Ok(doc)
    }
}
