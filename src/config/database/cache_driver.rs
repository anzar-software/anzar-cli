use utoipa::ToSchema;

#[derive(
    Debug, Default, Clone, Copy, serde::Deserialize, serde::Serialize, Eq, PartialEq, ToSchema,
)]
pub enum CacheDriver {
    #[default]
    MemCached,
    Redis,
    InMemory,
}

impl CacheDriver {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheDriver::MemCached => "memcached",
            CacheDriver::Redis => "redis",
            CacheDriver::InMemory => "in_memory",
        }
    }
}
impl std::fmt::Display for CacheDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl TryFrom<String> for CacheDriver {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "memcached" => Ok(Self::MemCached),
            "redis" => Ok(Self::Redis),
            "in_memory" => Ok(Self::InMemory),
            other => Err(format!(
                "{} is not supported database. Use either `memcached`, `redis` or `in_memory`",
                other
            )),
        }
    }
}
