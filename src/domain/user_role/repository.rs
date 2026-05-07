use std::sync::Arc;

use crate::error::{Error, InternalError, ResourceKind, Result};

use super::model::UserRole;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct UserRoleRepository {
    adapter: Arc<dyn DatabaseAdapter<UserRole>>,
}

impl UserRoleRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<UserRole>>) -> Self {
        Self { adapter }
    }
}

impl UserRoleRepository {
    #[tracing::instrument(name = "db.user_role.insert", skip(self, role))]
    pub async fn insert(&self, role: UserRole) -> Result<()> {
        match self.adapter.insert(role).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to assign role to user - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.user_role.find", skip(self, id))]
    pub async fn find(&self, id: &str) -> Result<UserRole> {
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

    #[tracing::instrument(name = "db.user_role.find", skip(self, user_id, role_id))]
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
