use crate::config::AppState;
use crate::error::Result;

use super::model::Role;

pub trait RoleServiceTrait {
    fn insert_role(&self, role: &str) -> impl Future<Output = Result<()>>;
    fn find_role(&self, user_id: &str) -> impl Future<Output = Result<Role>>;
}

impl RoleServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_role", skip(self, name))]
    async fn insert_role(&self, name: &str) -> Result<()> {
        let role = Role::new(name);
        self.auth_service.role_repository.insert(role).await
    }

    #[tracing::instrument(name = "auth.find_role", skip(self, name))]
    async fn find_role(&self, name: &str) -> Result<Role> {
        self.auth_service.role_repository.find(name).await
    }
}
