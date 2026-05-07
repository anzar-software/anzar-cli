use async_trait::async_trait;

use crate::domain::cache::CacheAdapter;
use crate::error::Error;

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
    async fn insert(&self, key: &str, value: &str, expiration: u64) -> Result<(), Error> {
        self.client
            .add(key, value, expiration as u32)
            .map_err(|e| e.into())
    }

    async fn find_one(&self, key: &str) -> Result<Option<String>, Error> {
        self.client.get::<String>(key).map_err(|e| e.into())
    }

    async fn update(&self, key: &str, value: &str, expiration: u64) -> Result<(), Error> {
        self.client
            .replace(key, value, expiration as u32)
            .map_err(|e| e.into())
    }

    async fn increment(&self, key: &str) -> Result<u64, Error> {
        self.client.increment(key, 1).map_err(|e| e.into())
    }

    async fn delete_one(&self, key: &str) -> Result<(), Error> {
        self.client.delete(key).map(|_| ()).map_err(|e| e.into())
    }

    async fn flush_all(&self) -> Result<(), Error> {
        self.client.flush().map(|_| ()).map_err(|e| e.into())
    }
}
