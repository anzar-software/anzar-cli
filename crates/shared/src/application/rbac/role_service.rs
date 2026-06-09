use crate::error::Result;

use crate::domain::model::Role;
use crate::intern::rbac::RbacService;

impl RbacService {
    #[tracing::instrument(name = "auth.insert_role", skip(self, name))]
    pub async fn upsert_role(&self, name: &str) -> Result<String> {
        let role = Role::new(name);
        self.role_repository.insert(role).await
    }

    #[tracing::instrument(name = "auth.find_role", skip(self, name))]
    pub async fn find_role(&self, name: &str) -> Result<Role> {
        self.role_repository.find(name).await
    }

    #[tracing::instrument(
        name = "auth.find_roles_by_user_id", skip(self), fields(user.id = user_id)
    )]
    pub async fn find_roles_by_user_id(&self, user_id: &str) -> Result<Vec<Role>> {
        let user_roles = self.user_role_repository.find_all(user_id).await?;

        let role_ids: Vec<String> = user_roles.iter().map(|ur| ur.role_id.clone()).collect();

        self.role_repository.find_by_ids(role_ids).await
    }
}
