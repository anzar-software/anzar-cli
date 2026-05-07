use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::utils::mongodb_serde::*;

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
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl Role {
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

impl Role {
    pub fn new(name: &str) -> Self {
        Self {
            id: None,
            name: name.into(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"name": "Admin"}))]
pub struct RoleName {
    pub name: String,
}
