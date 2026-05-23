use std::env::var;
use uuid::Uuid;

use crate::error::Result;

use crate::http::middlewares::rate_limiting::RateLimiter;
use crate::infrastructure::{
    cache::{CacheAdapters, in_memory::InMemoryAdapter, memcache::MemCache, redis::Redis},
    database::{DatabaseAdapters, mongodb::MongoDB, postgres::PostgreSQL, sqlite::SQLite},
};
use crate::utils::crypto::{Crypto, SecureToken};

use super::repository_registry::RepositoryRegistry;

use super::boot::AppConfig;
use super::boot::cache::CacheDriver;
use super::boot::database::DatabaseDriver;
use super::configuration::{AnzarConfiguration, Database};

#[derive(Clone)]
pub struct AppState {
    pub crypto: Crypto,
    pub repositories: RepositoryRegistry,
    pub configuration: AnzarConfiguration,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub async fn production(app_config: &AppConfig) -> Result<Self> {
        dotenvy::dotenv().ok();

        let env_overrides =
            config::Environment::default().source(Some(std::collections::HashMap::from([
                ("APP.URL".into(), var("API_BASE_URL")?),
                ("SECURITY.SECRET_KEY".into(), var("SECRET_KEY")?),
                ("DATABASE.CONNECTION_STRING".into(), var("DATABASE_URL")?),
                ("CACHE.URL".into(), var("CACHE_URL")?),
            ])));

        let configuration = config::Config::builder()
            .add_source(config::File::with_name(&app_config.config_path))
            .add_source(env_overrides)
            .build()?
            .try_deserialize::<AnzarConfiguration>()?;

        let repositories = RepositoryRegistry::from_database(&configuration.database).await?;

        configuration.validate()?;
        let crypto = Crypto::from_configuration(&configuration)?;

        Ok(Self {
            crypto,
            repositories,
            configuration,
            rate_limiter: RateLimiter::default(),
        })
    }

    pub async fn testing(address: &str) -> Result<Self> {
        let configuration = Self::build_config(address).await?;
        let repositories = Self::build_authservice(&configuration.database).await?;

        configuration.validate()?;
        let crypto = Crypto::from_configuration(&configuration)?;

        Ok(Self {
            crypto,
            repositories,
            configuration,
            rate_limiter: RateLimiter::default(),
        })
    }

    async fn build_config(address: &str) -> Result<AnzarConfiguration> {
        let mut app_config = AppConfig::load().expect("Failed to read configuration");

        app_config.database.name = match app_config.database.driver {
            DatabaseDriver::SQLite => app_config.database.name,
            DatabaseDriver::MongoDB => Uuid::new_v4().to_string(),
            DatabaseDriver::PostgreSQL => {
                let name = Uuid::new_v4().to_string();

                PostgreSQL::start(&app_config.database.connection_string())
                    .await?
                    .create_database(&name)
                    .await?;

                name
            }
        };

        let secret = SecureToken::with_size64().generate()?;
        Ok(AnzarConfiguration::new(app_config)
            .with_appurl(address)
            .with_secret(&secret))
    }

    async fn build_authservice(database: &Database) -> Result<RepositoryRegistry> {
        let cache_adapters = match database.cache.driver {
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

        let database_adapters = match database.driver {
            DatabaseDriver::SQLite => {
                let sqlite = SQLite::start(&database.connection_string).await?;
                sqlite.run_migrations().await?;

                DatabaseAdapters::sqlite(&sqlite.pool)
            }
            DatabaseDriver::MongoDB => {
                let client = MongoDB::start(&database.connection_string).await?;
                let conn_string = database.name().unwrap_or_default();
                DatabaseAdapters::mongodb(&client, conn_string)
            }
            DatabaseDriver::PostgreSQL => {
                let postgresql = PostgreSQL::start(&database.connection_string).await?;
                postgresql.run_migrations().await?;

                DatabaseAdapters::postgres(&postgresql.pool)
            }
        };

        Ok(RepositoryRegistry::new(database_adapters, cache_adapters))
    }
}
