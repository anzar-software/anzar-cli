use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::super::serde::{
    deserialize_object_id, deserialize_object_id_as_string, serialize_object_id_as_string,
};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize, FromRow, ToSchema)]
#[schema(example = json!({ "permissionId": String::default(), "roleId": String::default() }))]
pub struct RolePermission {
    #[serde(
        rename = "_id",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_object_id_as_string"
    )]
    pub id: Option<String>,

    #[sqlx(rename = "permissionId")]
    #[serde(
        rename = "permissionId",
        default,
        serialize_with = "serialize_object_id_as_string",
        deserialize_with = "deserialize_object_id"
    )]
    pub permission_id: String,

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

impl RolePermission {
    pub fn new(role_id: &str, permission_id: &str) -> Self {
        Self {
            id: None,
            role_id: role_id.into(),
            permission_id: permission_id.into(),
            issued_at: chrono::Utc::now(),
        }
    }
}
