use std::sync::Arc;

use crate::error::{CoreError, InternalError, ResourceKind, Result};

use super::model::User;
use super::ports::cache::CacheAdapter;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

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
    #[tracing::instrument(name = "cache.user.insert", skip(self))]
    pub async fn add(&self, key: &str, expiration: u64) -> Result<()> {
        self.cache.insert(key, "1", expiration).await
    }
    #[tracing::instrument(name = "cache.user.increment", skip(self))]
    pub async fn increment(&self, key: &str, expiration: u64) -> u8 {
        match self.cache.find_one(key).await {
            Ok(Some(_)) => self
                .cache
                .increment(key)
                .await
                .map(|v| v as u8)
                .unwrap_or(1),
            Ok(None) | Err(_) => self.add(key, expiration).await.map(|_| 1).unwrap_or(1),
        }
    }
    #[tracing::instrument(name = "cache.user.get_increment", skip(self))]
    pub async fn get_attempts(&self, key: &str) -> u8 {
        if let Ok(Some(val)) = self.cache.find_one(key).await {
            return val.parse::<u8>().unwrap_or(0);
        }
        0
    }
    #[tracing::instrument(name = "cache.user.insert", skip(self))]
    pub async fn put_cookie_in_lockout(&self, key: &str, expiration: u64) -> Result<()> {
        self.cache
            .insert(&format!("lockout:{}", key), "locked", expiration)
            .await
    }
    #[tracing::instrument(name = "cache.user.islocked", skip(self))]
    pub async fn is_locked(&self, key: &str) -> bool {
        self.cache.find_one(key).await.is_ok_and(|v| v.is_some())
    }
    #[tracing::instrument(name = "cache.user.reset", skip(self))]
    pub async fn reset_attempts(&self, key: &str, expiration: u64) -> Result<()> {
        self.cache.update(key, "0", expiration).await
    }
    #[tracing::instrument(name = "cache.user.clear", skip(self))]
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
                Err(CoreError::Internal(InternalError::Database(e.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.user.find", skip(self, user_id))]
    pub async fn find(&self, user_id: &str) -> Result<User> {
        let filter = QueryBuilder::default().eq("id", user_id);

        match self.adapter.find_one(filter).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(CoreError::NotFound(ResourceKind::User {
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
            Ok(None) => Err(CoreError::NotFound(ResourceKind::User {
                id: None,
                email: Some(email.into()),
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
