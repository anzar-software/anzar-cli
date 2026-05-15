use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleName {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PermissionId {
    pub permission_id: String,
}
