use crate::config::database::cache_driver::CacheDriver;

#[derive(Default, Debug, serde::Deserialize)]
pub struct CacheConfig {
    pub url: String,
    pub driver: CacheDriver,
}
