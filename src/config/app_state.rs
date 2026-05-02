use std::fs;
use uuid::Uuid;

use crate::adapters::cache::{
    CacheAdapters, in_memory::InMemoryAdapter, memcache::MemCache, redis::Redis,
};
use crate::adapters::database::{
    DatabaseAdapters, mongodb::MongoDB, postgres::PostgreSQL, sqlite::SQLite,
};
use crate::config::database::cache_driver::CacheDriver;
use crate::config::{AnzarConfiguration, AppConfig, AuthStrategy, Database, DatabaseDriver};
use crate::error::Result;
use crate::scopes::auth::service::AuthService;
use crate::utils::crypto::{Crypto, SecureToken};

#[derive(Clone)]
pub struct AppState {
    pub crypto: Crypto,
    pub auth_service: AuthService,
    pub configuration: AnzarConfiguration,
}

impl AppState {
    pub async fn production(app_config: &AppConfig) -> Result<Self> {
        let content = fs::read_to_string(&app_config.config_path)?;
        let configuration: AnzarConfiguration = serde_yaml::from_str(content.as_str())?;
        let auth_service = AuthService::from_database(&configuration.database).await?;

        let crypto = {
            let base = Crypto::with_argon()
                .with_hmac_secret(&configuration.security.secret_key)
                .with_token_size64();

            match configuration.auth.strategy {
                AuthStrategy::Jwt => base.with_jwt(configuration.clone()),
                _ => base,
            }
        };
        crypto.validate()?;

        Ok(Self {
            crypto,
            auth_service,
            configuration,
        })
    }

    pub async fn testing(address: &str) -> Result<Self> {
        let configuration = Self::build_config(address).await?;
        let auth_service = Self::build_authservice(&configuration.database).await?;

        let crypto = {
            let base = Crypto::with_argon()
                .with_hmac_secret(&configuration.security.secret_key)
                .with_token_size64();

            match configuration.auth.strategy {
                AuthStrategy::Jwt => base.with_jwt(configuration.clone()),
                _ => base,
            }
        };

        crypto.validate()?;
        Ok(Self {
            crypto,
            auth_service,
            configuration,
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

    async fn build_authservice(database: &Database) -> Result<AuthService> {
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

        Ok(AuthService::new(database_adapters, cache_adapters))
    }
}
