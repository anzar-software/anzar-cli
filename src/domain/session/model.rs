use crate::{domain::query::IntoBsonDocument, error::Error};
use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::future::{Ready, ready};
use utoipa::ToSchema;

use super::super::serde::{
    deserialize_datetime, deserialize_object_id, deserialize_object_id_as_string,
    deserialize_option_datetime,
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
#[schema(example = json!({"id": Some(String::default()), "user_id": String::default(), "issued_at": "2026-02-19T22:42:23.467Z", "expires_at": "2026-02-19T22:42:23.467Z", "used_at": Some("2026-02-19T22:42:23.467Z"), "token": String::default()}))]
pub struct Session {
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

impl Default for Session {
    fn default() -> Self {
        Self {
            id: None,
            user_id: String::default(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            used_at: None,
            token: String::default(),
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
}
impl Session {
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

impl IntoBsonDocument for Session {
    fn into_bson_document(self) -> Result<mongodb::bson::Document, mongodb::bson::ser::Error> {
        let mut doc = mongodb::bson::to_document(&self)?;

        for key in &["expiresAt", "issuedAt", "usedAt"] {
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

impl FromRequest for Session {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Session>() {
            Some(session) => ready(Ok(session.clone())),
            None => ready(Ok(Session::default())),
        }
    }
}
