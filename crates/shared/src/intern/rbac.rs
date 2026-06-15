use std::collections::{HashMap, HashSet};

use crate::config::RbacConfig;
use crate::domain::model::Role;
use crate::infrastructure::database::DatabaseAdapters;

use crate::domain::repositories::{
    PermissionRepository, RolePermissionRepository, RoleRepository, UserRoleRepository,
};

use crate::error::Result;

#[derive(Clone)]
pub struct RbacService {
    pub(crate) role_repository: RoleRepository,
    pub(crate) user_role_repository: UserRoleRepository,
    pub(crate) permission_repository: PermissionRepository,
    pub(crate) role_permission_repository: RolePermissionRepository,
    pub(crate) rbac_policy: RbacPolicy,
}

impl RbacService {
    pub fn new(database_adapters: &DatabaseAdapters, config: &RbacConfig) -> Self {
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
            rbac_policy: RbacPolicy::from_config(config),
        }
    }
}
impl RbacService {
    pub async fn get_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        let roles = self.find_roles_by_user_id(user_id).await?;
        let permissions = self.rbac_policy.resolve_permissions(roles);

        Ok(permissions)
    }

    pub async fn sync_permission(&self) -> Result<()> {
        for (role_name, permissions) in &self.rbac_policy.resolved {
            let role_id = self.upsert_role(role_name).await?;
            let permission_ids = self.upsert_permissions(permissions).await?;

            self.upsert_role_permissions(&role_id, permission_ids)
                .await?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct RbacPolicy {
    resolved: HashMap<String, HashSet<String>>,
    enabled: bool,
}

impl RbacPolicy {
    fn from_config(config: &RbacConfig) -> Self {
        let mut resolved: HashMap<String, HashSet<String>> = config
            .roles
            .iter()
            .map(|role| {
                (
                    role.name.clone(),
                    role.permissions.iter().cloned().collect(),
                )
            })
            .collect();

        let mut changed = true;
        while changed {
            changed = false;
            for parent in &config.roles {
                for inherited_role in &parent.inherits {
                    let permissions = resolved.get(inherited_role).cloned().unwrap_or_default();
                    let entry = resolved.entry(parent.name.clone()).or_default();
                    let before = entry.len();
                    entry.extend(permissions);
                    if entry.len() != before {
                        changed = true;
                    }
                }
            }
        }

        Self {
            resolved,
            enabled: config.enabled,
        }
    }
}
impl RbacPolicy {
    pub fn resolve_permissions(&self, roles: Vec<Role>) -> Vec<String> {
        if !self.enabled {
            return vec![];
        }

        roles
            .iter()
            .filter_map(|role| self.resolved.get(&role.name))
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect()

        // let mut full_permissions: Vec<String> = Vec::new();
        //
        // if !&self.configuration.auth.rbac.enabled {
        //     return Ok(full_permissions);
        // }
        //
        // let roles = self.rbac_service.find_roles_by_user_id(user_id).await?;
        // for role in roles {
        //     let role_id = role.id()?;
        //     let response = self
        //         .rbac_service
        //         .find_permissions_by_role_id(role_id)
        //         .await?;
        //
        //     let mut permissions = response.iter().map(|r| r.name.clone()).collect();
        //     full_permissions.append(&mut permissions);
        // }
        //
        // Ok(full_permissions)
    }
}
