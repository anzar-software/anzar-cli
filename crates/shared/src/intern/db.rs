use crate::error::Result;
use crate::{
    config::{Database, cache::CacheDriver, database::DatabaseDriver},
    infrastructure::{
        cache::{CacheAdapters, in_memory::InMemoryAdapter, memcache::MemCache, redis::Redis},
        database::{DatabaseAdapters, mongodb::MongoDB, postgres::PostgreSQL, sqlite::SQLite},
    },
};

pub struct DB {
    pub database: DatabaseAdapters,
    pub cache: CacheAdapters,
}

impl DB {
    pub async fn new(database: &Database) -> Result<Self> {
        let cache_adapter = match database.cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&database.cache.url).await?;
                CacheAdapters::memcached(client)
            }
            CacheDriver::Redis => {
                let connection = Redis::start(&database.cache.url).await?;
                CacheAdapters::redis(connection)
            }
            CacheDriver::InMemory => {
                let store = InMemoryAdapter::default();
                CacheAdapters::in_memory(store)
            }
        };

        let database_adapter = match database.driver {
            DatabaseDriver::SQLite => {
                let sqlite = SQLite::start(&database.connection_string).await?;
                sqlite.run_migrations().await?;

                DatabaseAdapters::sqlite(&sqlite.pool)
            }
            DatabaseDriver::MongoDB => {
                let client = MongoDB::start(&database.connection_string).await?;
                let db_name = database.name().unwrap_or_default();
                DatabaseAdapters::mongodb(&client, db_name)
            }
            DatabaseDriver::PostgreSQL => {
                let postgresql = PostgreSQL::start(&database.connection_string).await?;
                postgresql.run_migrations().await?;

                DatabaseAdapters::postgres(&postgresql.pool)
            }
        };

        Ok(Self {
            database: database_adapter,
            cache: cache_adapter,
        })
    }
}
