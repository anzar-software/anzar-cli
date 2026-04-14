mod adapter;
mod adapters;

pub mod memcache;
pub mod redis;

pub use adapter::CacheAdapter;
pub use adapters::CacheAdapters;
