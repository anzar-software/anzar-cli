use std::sync::Arc;

use super::User;
use crate::{
    adapters::{cache::CacheAdapter, database::DatabaseAdapter},
    error::{CredentialField, Error, InternalError, ResourceKind, Result, ValidationError},
    utils::query::QueryBuilder,
};

#[derive(Clone)]
pub struct UserRepository {
    adapter: Arc<dyn DatabaseAdapter<User>>,
    cache: Arc<dyn CacheAdapter>,
}

impl UserRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<User>>, cache: Arc<dyn CacheAdapter>) -> Self {
        Self { adapter, cache }
    }
}

impl UserRepository {
    // Cache
    pub async fn increment(&self, key: &str) -> u8 {
        self.cache.increment(key, 1).await.unwrap_or(1) as u8
    }
    pub async fn get_attempts(&self, key: &str) -> u8 {
        if let Ok(Some(val)) = self.cache.find_one(key).await {
            return val.parse::<u8>().unwrap_or(0);
        }
        0
    }
    pub async fn put_cookie_in_lockout(&self, key: &str, expiration: u64) -> Result<()> {
        self.cache
            .insert(&format!("lockout:{}", key), "locked", expiration)
            .await
    }
    pub async fn is_locked(&self, key: &str) -> bool {
        self.cache.find_one(key).await.is_ok_and(|v| v.is_some())
    }
    pub async fn reset_attempts(&self, key: &str) -> Result<()> {
        self.cache.update(key, "0", 1000000).await
    }
    pub async fn clear_key(&self, key: &str) -> Result<()> {
        self.cache.delete_one(key).await
    }

    // Database
    #[tracing::instrument(
        name = "db.user.insert",
        skip(self, user),
        fields(db.table = "users", db.operation = "INSERT")
    )]
    pub async fn insert(&self, user: &User) -> Result<String> {
        match self.adapter.insert(user.to_owned()).await {
            Ok(id) => Ok(id),
            Err(e) => {
                tracing::error!("Failed to insert user to database - {e}");
                Err(Error::Internal(InternalError::Database(e.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.user.find", skip(self, user_id))]
    pub async fn find(&self, user_id: &str) -> Result<User> {
        let filter = QueryBuilder::default().eq("id", user_id);

        match self.adapter.find_one(filter).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::NotFound(ResourceKind::User {
                id: Some(user_id.into()),
                email: None,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.user.find_by_email", skip(self, email))]
    pub async fn find_by_email(&self, email: &str) -> Result<User> {
        let filter = QueryBuilder::default().eq("email", email);

        match self.adapter.find_one(filter).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::NotFound(ResourceKind::User {
                id: None,
                email: Some(email.into()),
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.user.validate_account", skip(self, user_id))]
    pub async fn validate_account(&self, user_id: &str) -> Result<User> {
        let filter = QueryBuilder::default().eq("id", user_id);
        let update = QueryBuilder::default().set("verified", true);

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::Validation(ValidationError::Missing {
                field: CredentialField::ObjectId,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
