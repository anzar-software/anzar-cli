use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"action": "read", "resource": "user"}))]
pub struct PermissionName {
    pub action: String,
    pub resource: String,
}
