use crate::config::AppState;
use crate::domain::model::Permission;
use crate::error::Result;

use super::traits::PermissionServiceTrait;

impl PermissionServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_permission", skip(self, name))]
    async fn upsert_permission(&self, name: &str) -> Result<String> {
        let permission = Permission::new(name);

        self.repositories
            .permission_repository
            .insert(permission)
            .await
    }

    #[tracing::instrument(name = "auth.insert_permissions", skip(self, names))]
    async fn upsert_permissions(&self, names: Vec<String>) -> Result<Vec<String>> {
        let permissions: Vec<Permission> = names.iter().map(|n| Permission::new(n)).collect();

        self.repositories
            .permission_repository
            .upsert_many(permissions)
            .await
    }

    #[tracing::instrument(name = "auth.find_permission", skip(self, role_id))]
    async fn find_permissions_by_role_id(&self, role_id: &str) -> Result<Vec<Permission>> {
        let role_permissions = self
            .repositories
            .role_permission_repository
            .find_all(role_id)
            .await?;

        let permission_ids: Vec<String> = role_permissions
            .iter()
            .map(|rp| rp.permission_id.clone())
            .collect();

        self.repositories
            .permission_repository
            .find_by_ids(permission_ids)
            .await
    }
}
