use crate::infrastructure::database::DatabaseAdapters;

use crate::domain::repositories::{
    PermissionRepository, RolePermissionRepository, RoleRepository, UserRoleRepository,
};

#[derive(Clone)]
pub struct RbacService {
    pub(crate) role_repository: RoleRepository,
    pub(crate) user_role_repository: UserRoleRepository,
    pub(crate) permission_repository: PermissionRepository,
    pub(crate) role_permission_repository: RolePermissionRepository,
}

impl RbacService {
    pub fn new(database_adapters: &DatabaseAdapters) -> Self {
        Self {
            role_repository: RoleRepository::new(database_adapters.role_adapter.clone()),
            user_role_repository: UserRoleRepository::new(
                database_adapters.user_role_adapter.clone(),
            ),
            permission_repository: PermissionRepository::new(
                database_adapters.permission_adapter.clone(),
            ),
            role_permission_repository: RolePermissionRepository::new(
                database_adapters.role_permission_adapter.clone(),
            ),
        }
    }
}
