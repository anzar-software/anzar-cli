use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::super::serde::{deserialize_object_id, serialize_object_id_as_string};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
#[schema(example = json!({ "userId": String::default(), "roleId": String::default() }))]
pub struct UserRole {
    #[sqlx(rename = "userId")]
    #[serde(
        rename = "userId",
        default,
        serialize_with = "serialize_object_id_as_string",
        deserialize_with = "deserialize_object_id"
    )]
    pub user_id: String,

    #[sqlx(rename = "roleId")]
    #[serde(
        rename = "roleId",
        default,
        serialize_with = "serialize_object_id_as_string",
        deserialize_with = "deserialize_object_id"
    )]
    pub role_id: String,

    #[sqlx(rename = "issuedAt")]
    #[serde(rename = "issuedAt")]
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

impl UserRole {
    pub fn new(user_id: &str, role_id: &str) -> Self {
        Self {
            user_id: user_id.into(),
            role_id: role_id.into(),
            issued_at: chrono::Utc::now(),
        }
    }
}
