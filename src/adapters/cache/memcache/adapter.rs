use async_trait::async_trait;

use crate::{adapters::cache::CacheAdapter, error::Error};

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
    async fn insert(&self, key: &str, value: String, expiration: u32) -> Result<(), Error> {
        self.client
            .add(key, value, expiration)
            .map_err(|e| e.into())
    }

    async fn find_one(&self, key: &str) -> Result<Option<String>, Error> {
        self.client.get::<String>(key).map_err(|e| e.into())
    }

    async fn update(&self, key: &str, value: String, expiration: u32) -> Result<(), Error> {
        self.client
            .replace(key, value, expiration)
            .map_err(|e| e.into())
    }

    async fn increment(&self, key: &str, step: u64) -> Result<u64, Error> {
        self.client.increment(key, step).map_err(|e| e.into())
    }

    async fn delete_one(&self, key: &str) -> Result<bool, Error> {
        self.client.delete(key).map_err(|e| e.into())
    }
}
