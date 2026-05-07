use crate::config::AppState;
use crate::error::Result;

use super::model::UserRole;

pub trait UserRoleServiceTrait {
    fn insert_user_role(&self, user_id: &str, role_id: &str) -> impl Future<Output = Result<()>>;
    fn find_user_role(&self, user_id: &str) -> impl Future<Output = Result<UserRole>>;
}

impl UserRoleServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_user_role", skip(self, user_id, role))]
    async fn insert_user_role(&self, user_id: &str, role: &str) -> Result<()> {
        let role = self.auth_service.role_repository.find(role).await?;
        let role_id = role.id()?;

        let user_role = UserRole::new(user_id, role_id);
        self.auth_service
            .user_role_repository
            .insert(user_role)
            .await
    }

    #[tracing::instrument(
        name = "auth.find_role", skip(self), fields(user.id = user_id)
    )]
    async fn find_user_role(&self, user_id: &str) -> Result<UserRole> {
        self.auth_service.user_role_repository.find(user_id).await
    }
}
