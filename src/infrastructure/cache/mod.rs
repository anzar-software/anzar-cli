mod adapters;

pub mod in_memory;
pub mod memcache;
pub mod redis;

pub use adapters::CacheAdapters;
