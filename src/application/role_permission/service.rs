use crate::config::AppState;
use crate::error::Result;

use super::traits::RolePermissionServiceTrait;
use crate::domain::model::RolePermission;

impl RolePermissionServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.insert_role_permission",
        skip(self, role_id, permission_id)
    )]
    async fn insert_role_permission(&self, role_id: &str, permission_id: &str) -> Result<()> {
        let role_permission = RolePermission::new(role_id, permission_id);

        self.repositories
            .role_permission_repository
            .insert(role_permission)
            .await
    }

    #[tracing::instrument(
        name = "auth.insert_role_permissions",
        skip(self, role_id, permission_ids)
    )]
    async fn upsert_role_permissions(
        &self,
        role_id: &str,
        permission_ids: Vec<String>,
    ) -> Result<()> {
        let role_permissions: Vec<RolePermission> = permission_ids
            .iter()
            .map(|p_id| RolePermission::new(role_id, p_id))
            .collect();

        self.repositories
            .role_permission_repository
            .upsert_many(role_permissions)
            .await
    }

    #[tracing::instrument(
        name = "auth.find_role_permission", skip(self), fields(user.id = user_id)
    )]
    async fn find_role_permissions(&self, user_id: &str) -> Result<RolePermission> {
        self.repositories
            .role_permission_repository
            .find(user_id)
            .await
    }
}
