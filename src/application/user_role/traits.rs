use crate::domain::model::UserRole;
use crate::error::Result;

pub trait UserRoleServiceTrait {
    fn insert_user_role(&self, user_id: &str, role_id: &str) -> impl Future<Output = Result<()>>;
    fn find_user_role(&self, user_id: &str) -> impl Future<Output = Result<UserRole>>;
}
