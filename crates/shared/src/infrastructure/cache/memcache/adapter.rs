use async_trait::async_trait;

use crate::domain::cache::CacheAdapter;
use crate::error::CoreError;

#[derive(Clone)]
pub struct MemCacheAdapter {
    client: memcache::Client,
}

impl MemCacheAdapter {
    pub fn new(client: memcache::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl CacheAdapter for MemCacheAdapter {
    async fn insert(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError> {
        self.client
            .add(key, value, expiration as u32)
            .map_err(|e| e.into())
    }

    async fn find_one(&self, key: &str) -> Result<Option<String>, CoreError> {
        self.client.get::<String>(key).map_err(|e| e.into())
    }

    async fn update(&self, key: &str, value: &str, expiration: u64) -> Result<(), CoreError> {
        self.client
            .replace(key, value, expiration as u32)
            .map_err(|e| e.into())
    }

    async fn increment(&self, key: &str) -> Result<u64, CoreError> {
        self.client.increment(key, 1).map_err(|e| e.into())
    }

    async fn delete_one(&self, key: &str) -> Result<(), CoreError> {
        self.client.delete(key).map(|_| ()).map_err(|e| e.into())
    }

    async fn flush_all(&self) -> Result<(), CoreError> {
        self.client.flush().map(|_| ()).map_err(|e| e.into())
    }
}
