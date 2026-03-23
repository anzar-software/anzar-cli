use std::sync::Arc;

use super::CacheAdapter;
use crate::adapters::cache::memcache::MemCacheAdapter;

pub struct CacheAdapters {
    pub cache_adapter: Arc<dyn CacheAdapter>,
}

impl CacheAdapters {
    pub fn memcached(client: memcache::Client) -> Self {
        Self {
            cache_adapter: Arc::new(MemCacheAdapter::new(client)),
        }
    }

    pub fn redis() -> Self {
        todo!()
    }
}
