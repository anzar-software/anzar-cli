use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RoleName {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PermissionId {
    pub permission_id: String,
}
