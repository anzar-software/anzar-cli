use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::domain::query::IntoBsonDocument;

use super::super::serde::{deserialize_datetime, deserialize_object_id_as_string};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CreateUserOutcome {
    Created(User),
    AlreadyExists,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
#[schema(example = json!({"id": Some(String::default()), "username": String::default(), "email": String::default(), "created_at": "2026-02-19T22:42:23.467Z"}))]
pub struct User {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    pub username: String,
    pub email: String,

    #[sqlx(rename = "createdAt")]
    #[serde(rename = "createdAt", deserialize_with = "deserialize_datetime")]
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new() -> Self {
        Self {
            created_at: Utc::now(),
            ..Default::default()
        }
    }
    pub fn from_request(user: User) -> Self {
        user
    }
}
impl User {
    pub fn with_id(&mut self, id: &str) {
        self.id = Some(id.into());
    }
    pub fn with_username(mut self, username: &str) -> Self {
        self.username = username.into();
        self
    }
    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.into();
        self
    }
}
impl User {
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

impl IntoBsonDocument for User {
    fn into_bson_document(self) -> Result<mongodb::bson::Document, mongodb::bson::ser::Error> {
        let mut doc = mongodb::bson::to_document(&self)?;

        for key in &["createdAt"] {
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
