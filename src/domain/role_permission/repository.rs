use std::sync::Arc;

use crate::error::{Error, InternalError, ResourceKind, Result};

use super::model::RolePermission;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct RolePermissionRepository {
    adapter: Arc<dyn DatabaseAdapter<RolePermission>>,
}

impl RolePermissionRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<RolePermission>>) -> Self {
        Self { adapter }
    }
}

impl RolePermissionRepository {
    #[tracing::instrument(name = "db.role_permission.insert", skip(self, role))]
    pub async fn insert(&self, role: RolePermission) -> Result<()> {
        match self.adapter.upsert(role).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to assign permission to role - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }
    #[tracing::instrument(name = "db.role_permission.insert", skip(self, roles))]
    pub async fn upsert_many(&self, roles: Vec<RolePermission>) -> Result<()> {
        match self.adapter.upsert_many(roles).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to assign permission to role - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.role_permission.find", skip(self, id))]
    pub async fn find(&self, id: &str) -> Result<RolePermission> {
        let filter = QueryBuilder::default().eq("id", id);

        match self.adapter.find_one(filter).await {
            Ok(Some(role)) => Ok(role),
            Ok(None) => Err(Error::NotFound(ResourceKind::Role {
                id: Some(id.into()),
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.role_permission.find_all", skip(self, role_id))]
    pub async fn find_all(&self, role_id: &str) -> Result<Vec<RolePermission>> {
        let filter = QueryBuilder::default().eq("roleId", role_id);

        match self.adapter.find_all(filter).await {
            Ok(roles) => Ok(roles),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.role_permission.find", skip(self, user_id, role_id))]
    pub async fn delete(&self, user_id: &str, role_id: &str) -> Result<()> {
        let filter = QueryBuilder::default()
            .eq("userId", user_id)
            .eq("roleId", role_id);

        match self.adapter.delete_one(filter).await {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
