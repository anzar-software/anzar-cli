use std::sync::Arc;

use super::CacheAdapter;
use crate::adapters::cache::{memcache::MemCacheAdapter, redis::RedisAdapter};

pub struct CacheAdapters {
    pub cache_adapter: Arc<dyn CacheAdapter>,
}

impl CacheAdapters {
    pub fn memcached(client: memcache::Client) -> Self {
        Self {
            cache_adapter: Arc::new(MemCacheAdapter::new(client)),
        }
    }

    pub fn redis(connection: redis::aio::ConnectionManager) -> Self {
        Self {
            cache_adapter: Arc::new(RedisAdapter::new(connection)),
        }
    }
}
