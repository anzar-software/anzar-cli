use std::sync::Arc;

use crate::error::{Error, InternalError, ResourceKind, Result, TokenErrorType};

use super::model::SigningKeys;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct SigningKeysRepository {
    adapter: Arc<dyn DatabaseAdapter<SigningKeys>>,
}

impl SigningKeysRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<SigningKeys>>) -> Self {
        Self { adapter }
    }
}

impl SigningKeysRepository {
    #[tracing::instrument(name = "db.permission.insert", skip(self, permission))]
    pub async fn insert(&self, permission: SigningKeys) -> Result<String> {
        match self.adapter.upsert(permission).await {
            Ok(id) => Ok(id),
            Err(err) => {
                tracing::error!("Failed to insert permission to database - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.signing_keys.find", skip(self))]
    pub async fn find(&self) -> Result<SigningKeys> {
        let filter = QueryBuilder::default().eq("active", true);

        match self.adapter.find_one(filter).await {
            Ok(Some(key)) => Ok(key),
            // FIXME use Permission not Role
            Ok(None) => Err(Error::NotFound(ResourceKind::Token {
                token_type: TokenErrorType::Token,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.permission.find", skip(self))]
    pub async fn find_all(&self) -> Result<Vec<SigningKeys>> {
        let filter = QueryBuilder::default();

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
