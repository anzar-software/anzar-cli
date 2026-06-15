use crate::error::Result;
use crate::{
    config::{Cache, Database, cache::CacheDriver, database::DatabaseDriver},
    infrastructure::{
        cache::{CacheAdapters, in_memory::InMemoryAdapter, memcache::MemCache, redis::Redis},
        database::{DatabaseAdapters, mongodb::MongoDB, postgres::PostgreSQL, sqlite::SQLite},
    },
};

pub struct DB {
    pub database: DatabaseAdapters,
}
pub struct CacheDb {
    pub cache: CacheAdapters,
}

impl DB {
    pub async fn connect(database: &Database) -> Result<DatabaseAdapters> {
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

        Ok(database_adapter)
    }

    pub async fn migrate(database: &Database, path: &std::path::Path) -> Result<()> {
        // let path = std::path::Path::new("../../migrations/sqlite");
        // let path = std::path::Path::new("../../migrations/postgres");
        if !path.exists() {
            tracing::warn!("path must be valid");
            return Ok(());
        }

        match database.driver {
            DatabaseDriver::SQLite => {
                let migrator = sqlx::migrate::Migrator::new(path).await?;
                let sqlite = SQLite::start(&database.connection_string).await?;
                migrator
                    .run(&sqlite.pool)
                    .await
                    .inspect_err(|e| tracing::error!("Failed to run migrations - {e}"))?;

                Ok(())
            }
            DatabaseDriver::PostgreSQL => {
                let migrator = sqlx::migrate::Migrator::new(path).await?;
                let postgresql = PostgreSQL::start(&database.connection_string).await?;
                migrator
                    .run(&postgresql.pool)
                    .await
                    .inspect_err(|e| tracing::error!("Failed to run migrations - {e}"))?;

                Ok(())
            }
            DatabaseDriver::MongoDB => todo!(),
        }
    }
}

impl CacheDb {
    pub async fn connect(cache: &Cache) -> Result<CacheAdapters> {
        let cache_adapter = match cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&cache.url).await?;
                CacheAdapters::memcached(client)
            }
            CacheDriver::Redis => {
                let connection = Redis::start(&cache.url).await?;
                CacheAdapters::redis(connection)
            }
            CacheDriver::InMemory => {
                let store = InMemoryAdapter::default();
                CacheAdapters::in_memory(store)
            }
        };

        Ok(cache_adapter)
    }
}
