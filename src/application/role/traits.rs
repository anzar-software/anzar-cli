use crate::domain::model::Role;
use crate::error::Result;

pub trait RoleServiceTrait {
    fn insert_role(&self, role: &str) -> impl Future<Output = Result<()>>;
    fn find_role(&self, user_id: &str) -> impl Future<Output = Result<Role>>;
}
