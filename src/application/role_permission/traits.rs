use crate::domain::model::RolePermission;
use crate::error::Result;

pub trait RolePermissionServiceTrait {
    fn insert_role_permission(
        &self,
        role_id: &str,
        permission_id: &str,
    ) -> impl Future<Output = Result<()>>;
    fn upsert_role_permissions(
        &self,
        role_id: &str,
        permission_ids: Vec<String>,
    ) -> impl Future<Output = Result<()>>;
    fn find_role_permissions(&self, user_id: &str) -> impl Future<Output = Result<RolePermission>>;
}
