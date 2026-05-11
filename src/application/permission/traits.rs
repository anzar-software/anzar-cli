use crate::domain::model::Permission;
use crate::error::Result;

pub trait PermissionServiceTrait {
    fn upsert_permission(&self, name: &str) -> impl Future<Output = Result<String>>;
    fn upsert_permissions(&self, names: Vec<String>) -> impl Future<Output = Result<Vec<String>>>;
    fn find_permissions_by_role_id(
        &self,
        role_id: &str,
    ) -> impl Future<Output = Result<Vec<Permission>>>;
}
