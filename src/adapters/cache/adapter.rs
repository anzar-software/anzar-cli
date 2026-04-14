use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait CacheAdapter: Send + Sync {
    async fn insert(&self, key: &str, value: String, expiration: u64) -> Result<(), Error>;
    async fn find_one(&self, key: &str) -> Result<Option<String>, Error>;
    async fn update(&self, key: &str, value: String, expiration: u64) -> Result<(), Error>;
    async fn increment(&self, key: &str, step: u64) -> Result<u64, Error>;
    async fn delete_one(&self, key: &str) -> Result<(), Error>;
}
