use async_trait::async_trait;
use redis::AsyncTypedCommands;

use crate::domain::cache::CacheAdapter;
use crate::error::CoreError;

#[derive(Clone)]
pub struct RedisAdapter {
    connection: redis::aio::ConnectionManager,
}

impl RedisAdapter {
    pub fn new(connection: redis::aio::ConnectionManager) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl CacheAdapter for RedisAdapter {
    async fn insert(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError> {
        self.connection
            .clone()
            .set_ex(key, value, expiration)
            .await
            .map_err(|e| e.into())
    }

    async fn find_one(&self, key: &str) -> Result<Option<String>, CoreError> {
        self.connection.clone().get(key).await.map_err(|e| e.into())
    }

    async fn update(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError> {
        self.connection
            .clone()
            .set_ex(key, value, expiration)
            .await
            .map_err(|e| e.into())
    }

    async fn increment(&self, key: &str) -> Result<u64, CoreError> {
        self.connection
            .clone()
            .incr(key, 1)
            .await
            .map(|v| v as u64)
            .map_err(|e| e.into())
    }

    async fn delete_one(&self, key: &str) -> Result<(), CoreError> {
        self.connection
            .clone()
            .del(key)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }

    async fn flush_all(&self) -> Result<(), CoreError> {
        self.connection
            .clone()
            .flushall()
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
