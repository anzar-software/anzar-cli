use std::sync::Arc;

use crate::error::{Error, InternalError, ResourceKind, Result};

use super::model::Permission;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct PermissionRepository {
    adapter: Arc<dyn DatabaseAdapter<Permission>>,
}

impl PermissionRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<Permission>>) -> Self {
        Self { adapter }
    }
}

impl PermissionRepository {
    #[tracing::instrument(name = "db.permission.insert", skip(self, permission))]
    pub async fn insert(&self, permission: Permission) -> Result<String> {
        match self.adapter.upsert(permission).await {
            Ok(id) => Ok(id),
            Err(err) => {
                tracing::error!("Failed to insert permission to database - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }
    #[tracing::instrument(name = "db.permission.insert", skip(self, permissions))]
    pub async fn upsert_many(&self, permissions: Vec<Permission>) -> Result<Vec<String>> {
        match self.adapter.upsert_many(permissions).await {
            Ok(ids) => Ok(ids),
            Err(err) => {
                tracing::error!("Failed to insert permission to database - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.permission.find", skip(self, name))]
    pub async fn find(&self, name: &str) -> Result<Permission> {
        let filter = QueryBuilder::default().eq("name", name);

        match self.adapter.find_one(filter).await {
            Ok(Some(permission)) => Ok(permission),
            // FIXME use Permission not Role
            Ok(None) => Err(Error::NotFound(ResourceKind::Role {
                id: Some(name.into()),
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.permission.find", skip(self))]
    pub async fn find_all(&self) -> Result<Vec<Permission>> {
        let filter = QueryBuilder::default();

        match self.adapter.find_all(filter).await {
            Ok(permissions) => Ok(permissions),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.permission.find_by_id", skip(self, ids))]
    pub async fn find_by_ids(&self, ids: Vec<String>) -> Result<Vec<Permission>> {
        let filter = QueryBuilder::default().in_("id", ids);

        match self.adapter.find_all(filter).await {
            Ok(permissions) => Ok(permissions),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.permission.find", skip(self, id))]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("id", id);

        match self.adapter.delete_one(filter).await {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
