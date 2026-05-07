use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({"name": "Admin"}))]
pub struct RoleName {
    pub name: String,
}
