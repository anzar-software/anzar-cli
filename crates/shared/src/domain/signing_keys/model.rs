use crate::domain::query::IntoBsonDocument;
use chrono::{DateTime, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::super::serde::{
    deserialize_datetime, deserialize_object_id_as_string, deserialize_option_datetime,
};

pub struct SigningKeys {
    pub private_key: String,
    pub key: SigningKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
pub struct SigningKey {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    #[sqlx(rename = "createdAt")]
    #[serde(rename = "createdAt", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,

    #[sqlx(rename = "rotatedAt")]
    #[serde(rename = "rotatedAt", deserialize_with = "deserialize_option_datetime")]
    pub rotated_at: Option<DateTime<Utc>>,

    #[sqlx(rename = "expiresAt")]
    #[serde(rename = "expiresAt", deserialize_with = "deserialize_option_datetime")]
    pub expires_at: Option<DateTime<Utc>>,

    pub status: String, // active, retired, revoked

    pub encrypted_private_key: String,
    pub public_key: String,
    pub algorithm: String,
    pub kid: String,
    pub kty: String,
}

impl SigningKey {
    pub fn id(&self) -> Result<&str, crate::error::CoreError> {
        self.id.as_deref().ok_or_else(|| {
            tracing::error!(
                error_code = "ValidationError::Malformed",
                "Unexpected null/missing data"
            );
            crate::error::CoreError::Validation(crate::error::ValidationError::Malformed {
                field: crate::error::CredentialField::ObjectId,
            })
        })
    }
}

impl SigningKey {
    pub fn new(prv: &str, public: &str) -> Self {
        Self {
            id: None,
            created_at: Utc::now(),
            rotated_at: None,
            expires_at: None,
            status: "active".into(),
            encrypted_private_key: prv.into(),
            public_key: public.into(),
            algorithm: String::from("RS256"),
            kid: String::from("k1"),
            kty: String::from("RSA"),
        }
    }
}
impl SigningKey {
    pub fn with_algorithm(mut self, algorithm: &str) -> Self {
        self.algorithm = algorithm.into();
        self
    }
    pub fn with_kid(mut self, kid: &str) -> Self {
        self.kid = kid.into();
        self
    }
    pub fn with_kty(mut self, kty: &str) -> Self {
        self.kty = kty.into();
        self
    }
}

impl IntoBsonDocument for SigningKey {
    fn into_bson_document(self) -> Result<bson::Document, bson::ser::Error> {
        let mut doc = bson::to_document(&self)?;

        for key in &["createdAt", "rotatedAt", "expiresAt"] {
            if let Some(bson::Bson::String(s)) = doc.get(*key).cloned()
                && let Ok(dt) = DateTime::parse_from_rfc3339(&s)
            {
                doc.insert(
                    *key,
                    bson::Bson::DateTime(bson::DateTime::from_millis(dt.timestamp_millis())),
                );
            }
        }

        Ok(doc)
    }
}
