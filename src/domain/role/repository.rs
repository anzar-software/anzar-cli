use std::sync::Arc;

use crate::error::{Error, InternalError, ResourceKind, Result};

use super::model::Role;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct RoleRepository {
    adapter: Arc<dyn DatabaseAdapter<Role>>,
}

impl RoleRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<Role>>) -> Self {
        Self { adapter }
    }
}

impl RoleRepository {
    #[tracing::instrument(name = "db.role.insert", skip(self, role))]
    pub async fn insert(&self, role: Role) -> Result<String> {
        match self.adapter.upsert(role).await {
            Ok(id) => Ok(id),
            Err(err) => {
                tracing::error!("Failed to insert role to database - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.role.find", skip(self, name))]
    pub async fn find(&self, name: &str) -> Result<Role> {
        let filter = QueryBuilder::default().eq("name", name);

        match self.adapter.find_one(filter).await {
            Ok(Some(role)) => Ok(role),
            Ok(None) => Err(Error::NotFound(ResourceKind::Role {
                id: Some(name.into()),
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.role.find_by_id", skip(self, ids))]
    pub async fn find_by_ids(&self, ids: Vec<String>) -> Result<Vec<Role>> {
        let filter = QueryBuilder::default().in_("id", ids);

        match self.adapter.find_all(filter).await {
            Ok(roles) => Ok(roles),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.role.find", skip(self, id))]
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
