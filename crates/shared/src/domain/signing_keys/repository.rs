use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::error::{CoreError, InternalError, ResourceKind, Result, TokenErrorType};

use super::model::SigningKey;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct SigningKeysRepository {
    adapter: Arc<dyn DatabaseAdapter<SigningKey>>,
}

impl SigningKeysRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<SigningKey>>) -> Self {
        Self { adapter }
    }
}

impl SigningKeysRepository {
    #[tracing::instrument(name = "db.signing_key.insert", skip(self, signing_key))]
    pub async fn insert(&self, signing_key: SigningKey) -> Result<String> {
        match self.adapter.insert(signing_key).await {
            Ok(id) => Ok(id),
            Err(err) => {
                tracing::error!("Failed to insert signing_key to database - {err}");
                Err(CoreError::Internal(InternalError::Database(
                    err.to_string(),
                )))
            }
        }
    }

    #[tracing::instrument(name = "db.signing_keys.find", skip(self))]
    pub async fn find(&self) -> Result<SigningKey> {
        let filter = QueryBuilder::default().eq("status", "active");

        match self.adapter.find_one(filter).await {
            Ok(Some(key)) => Ok(key),
            // FIXME use Permission not Role
            Ok(None) => Err(CoreError::NotFound(ResourceKind::Token {
                token_type: TokenErrorType::Token,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.signingkeys.find_valid_keys", skip(self))]
    pub async fn find_valid_keys(&self) -> Result<Vec<SigningKey>> {
        let filter = QueryBuilder::default()
            .ne("status", "revoked")
            .gt("expiresAt", Utc::now());

        match self.adapter.find_all(filter).await {
            Ok(permissions) => Ok(permissions),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.signingkeys.findall", skip(self))]
    pub async fn find_all(&self) -> Result<Vec<SigningKey>> {
        let filter = QueryBuilder::default();

        match self.adapter.find_all(filter).await {
            Ok(keys) => Ok(keys),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.signing_keys.find", skip(self))]
    pub async fn retire_oldkey(&self, max_token_ttl: i64) -> Result<SigningKey> {
        let filter = QueryBuilder::default().eq("status", "active");
        let update = QueryBuilder::default()
            .set("status", "retired")
            .set("rotatedAt", Utc::now())
            .set("expiresAt", Utc::now() + Duration::seconds(max_token_ttl));

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(key)) => Ok(key),
            Ok(None) => Err(CoreError::NotFound(ResourceKind::Token {
                token_type: TokenErrorType::Token,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.signing_keys.revoke", skip(self))]
    pub async fn revoke_key(&self, kid: &str) -> Result<SigningKey> {
        let filter = QueryBuilder::default().eq("kid", kid);
        let update = QueryBuilder::default().set("status", "revoked");

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(key)) => Ok(key),
            Ok(None) => Err(CoreError::NotFound(ResourceKind::Token {
                token_type: TokenErrorType::Token,
            })),
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
