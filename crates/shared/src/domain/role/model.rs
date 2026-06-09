use chrono::{DateTime, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::query::IntoBsonDocument;

use super::super::serde::{deserialize_datetime, deserialize_object_id_as_string};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
#[schema(example = json!({"id": Some(String::default()), "name": String::default(), "created_at": "2026-02-19T22:42:23.467Z"}))]
pub struct Role {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    pub name: String,

    #[sqlx(rename = "createdAt")]
    #[serde(rename = "createdAt", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
}

impl Role {
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

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.into(),
            created_at: Utc::now(),
        }
    }
}

impl IntoBsonDocument for Role {
    fn into_bson_document(self) -> Result<bson::Document, bson::ser::Error> {
        let mut doc = bson::to_document(&self)?;

        for key in &["createdAt"] {
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
