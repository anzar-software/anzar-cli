use crate::error::CoreError;
use async_trait::async_trait;

#[async_trait]
pub trait CacheAdapter: Send + Sync {
    async fn insert(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError>;
    async fn find_one(&self, key: &str) -> Result<Option<String>, CoreError>;
    async fn update(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError>;
    async fn increment(&self, key: &str) -> Result<u64, CoreError>;
    async fn delete_one(&self, key: &str) -> Result<(), CoreError>;

    async fn flush_all(&self) -> Result<(), CoreError>;
}
