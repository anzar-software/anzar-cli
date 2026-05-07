use crate::config::AppState;
use crate::domain::model::Role;
use crate::error::Result;

use super::traits::RoleServiceTrait;

impl RoleServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_role", skip(self, name))]
    async fn insert_role(&self, name: &str) -> Result<()> {
        let role = Role::new(name);
        self.repositories.role_repository.insert(role).await
    }

    #[tracing::instrument(name = "auth.find_role", skip(self, name))]
    async fn find_role(&self, name: &str) -> Result<Role> {
        self.repositories.role_repository.find(name).await
    }
}
