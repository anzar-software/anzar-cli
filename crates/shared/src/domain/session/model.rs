use chrono::{DateTime, Duration, Utc};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::super::serde::{
    deserialize_datetime, deserialize_object_id, deserialize_object_id_as_string,
    deserialize_option_datetime,
};
use crate::domain::query::IntoBsonDocument;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"id": Some(String::default()), "user_id": String::default(), "issued_at": "2026-02-19T22:42:23.467Z", "expires_at": "2026-02-19T22:42:23.467Z", "used_at": Some("2026-02-19T22:42:23.467Z"), "token": String::default()}))]
pub struct Session {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    #[serde(
        rename = "userId",
        default,
        // serialize_with = "serialize_object_id_as_string",
        deserialize_with = "deserialize_object_id"
    )]
    pub user_id: String,

    #[serde(rename = "issuedAt", deserialize_with = "deserialize_datetime")]
    pub issued_at: DateTime<Utc>,
    #[serde(rename = "expiresAt", deserialize_with = "deserialize_datetime")]
    pub expires_at: DateTime<Utc>,
    #[serde(rename = "usedAt", deserialize_with = "deserialize_option_datetime")]
    pub used_at: Option<DateTime<Utc>>,

    pub token: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

// Custom FromRow for SQLite JSON columns
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Session {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let roles_str: String = row.try_get("roles")?;
        let permissions_str: String = row.try_get("permissions")?;

        Ok(Session {
            id: row.try_get("id")?,
            user_id: row.try_get("userId")?,
            token: row.try_get("token")?,
            issued_at: row.try_get("issuedAt")?,
            expires_at: row.try_get("expiresAt")?,
            used_at: row.try_get("usedAt")?,
            roles: serde_json::from_str(&roles_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "roles".to_string(),
                source: Box::new(e),
            })?,
            permissions: serde_json::from_str(&permissions_str).map_err(|e| {
                sqlx::Error::ColumnDecode {
                    index: "permissions".to_string(),
                    source: Box::new(e),
                }
            })?,
        })
    }
}

// Postgres: roles/permissions are native TEXT[] arrays
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Session {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Session {
            id: row.try_get("id")?,
            user_id: row.try_get("userId")?,
            token: row.try_get("token")?,
            issued_at: row.try_get("issuedAt")?,
            expires_at: row.try_get("expiresAt")?,
            used_at: row.try_get("usedAt")?,
            roles: row.try_get("roles")?,             // native TEXT[]
            permissions: row.try_get("permissions")?, // native TEXT[]
        })
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: None,
            user_id: String::default(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            used_at: None,
            token: String::default(),
            roles: vec![],
            permissions: vec![],
        }
    }
}

impl Session {
    pub fn from_request(session: Session) -> Self {
        session
    }
}

impl Session {
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.into();
        self
    }
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = token.into();
        self
    }
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
    pub fn with_role(mut self, role: &str) -> Self {
        self.roles = vec![role.into()];
        self
    }
}
impl Session {
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

impl IntoBsonDocument for Session {
    fn into_bson_document(self) -> Result<bson::Document, bson::ser::Error> {
        let mut doc = bson::to_document(&self)?;

        for key in &["expiresAt", "issuedAt", "usedAt"] {
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
