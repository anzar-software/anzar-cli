use crate::config::AppState;
use crate::domain::model::Role;
use crate::error::Result;

use super::traits::RoleServiceTrait;

impl RoleServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_role", skip(self, name))]
    async fn upsert_role(&self, name: &str) -> Result<String> {
        let role = Role::new(name);
        self.repositories.role_repository.insert(role).await
    }

    #[tracing::instrument(name = "auth.find_role", skip(self, name))]
    async fn find_role(&self, name: &str) -> Result<Role> {
        self.repositories.role_repository.find(name).await
    }

    #[tracing::instrument(
        name = "auth.find_roles_by_user_id", skip(self), fields(user.id = user_id)
    )]
    async fn find_roles_by_user_id(&self, user_id: &str) -> Result<Vec<Role>> {
        let user_roles = self
            .repositories
            .user_role_repository
            .find_all(user_id)
            .await?;

        let role_ids: Vec<String> = user_roles.iter().map(|ur| ur.role_id.clone()).collect();

        self.repositories
            .role_repository
            .find_by_ids(role_ids)
            .await
    }
}
