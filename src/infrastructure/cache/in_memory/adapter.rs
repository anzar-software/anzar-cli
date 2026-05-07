use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::domain::cache::CacheAdapter;
use crate::error::{Error, InternalError, Result};

struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

pub struct InMemoryAdapter {
    store: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl InMemoryAdapter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheAdapter for InMemoryAdapter {
    async fn insert(&self, key: &str, value: &str, expiration: u64) -> Result<()> {
        let mut store = self.store.lock().await;
        store.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at: Some(Instant::now() + Duration::from_secs(expiration)),
            },
        );
        Ok(())
    }

    async fn find_one(&self, key: &str) -> Result<Option<String>> {
        let store = self.store.lock().await;

        match store.get(key) {
            Some(entry) => {
                if let Some(expires_at) = entry.expires_at {
                    if Instant::now() > expires_at {
                        return Ok(None);
                    }
                }
                Ok(Some(entry.value.clone()))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, key: &str, value: &str, expiration: u64) -> Result<()> {
        let mut store = self.store.lock().await;
        store.entry(key.to_string()).and_modify(|v| {
            *v = CacheEntry {
                value: value.to_string(),
                expires_at: Some(Instant::now() + Duration::from_secs(expiration)),
            };
        });

        Ok(())
    }

    async fn increment(&self, key: &str) -> Result<u64> {
        let mut store = self.store.lock().await;
        let entry = store.entry(key.to_string()).or_insert(CacheEntry {
            value: "0".to_string(),
            expires_at: None,
        });

        let new_val: u64 = entry.value.parse().unwrap_or(0) + 1;
        entry.value = new_val.to_string();

        Ok(new_val)
    }

    async fn delete_one(&self, key: &str) -> Result<()> {
        let mut store = self.store.lock().await;
        if store.remove(key).is_none() {
            return Err(Error::Internal(InternalError::Cache(
                "Failed deleting an entry form cache".to_string(),
            )));
        }

        Ok(())
    }

    async fn flush_all(&self) -> Result<()> {
        let mut store = self.store.lock().await;
        store.clear();

        Ok(())
    }
}
