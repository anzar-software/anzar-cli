use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"role": "admin"}))]
pub struct RoleRequest {
    pub role: String,
}
