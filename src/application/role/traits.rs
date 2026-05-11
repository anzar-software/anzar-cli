use crate::domain::model::Role;
use crate::error::Result;

pub trait RoleServiceTrait {
    fn upsert_role(&self, role: &str) -> impl Future<Output = Result<String>>;
    fn find_role(&self, user_id: &str) -> impl Future<Output = Result<Role>>;

    fn find_roles_by_user_id(&self, user_id: &str) -> impl Future<Output = Result<Vec<Role>>>;
}
