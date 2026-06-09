use super::environment::*;
use super::server::ServerConfig;

use super::auth::AuthDriver;
use super::cache::{CacheConfig, CacheDriver};
use super::database::{DatabaseConfig, DatabaseDriver};
use crate::config::AuthStrategy;

#[derive(Debug, serde::Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub config_dir: String,
    pub config_path: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub auth: AuthStrategy,
}

impl AppConfig {
    fn env() -> Environment {
        std::env::var("ENV")
            .unwrap_or_else(|_| Environment::Dev.as_str().into())
            .try_into()
            .expect("Failed to parse ENV")
    }
    fn db() -> DatabaseDriver {
        std::env::var("DB")
            .unwrap_or_else(|_| DatabaseDriver::SQLite.as_str().into())
            .try_into()
            .expect("Failed to parse DB")
    }
    fn cache() -> CacheDriver {
        std::env::var("CACHE")
            .unwrap_or_else(|_| CacheDriver::InMemory.as_str().into())
            .try_into()
            .expect("Failed to parse CACHE")
    }
    fn auth() -> AuthDriver {
        std::env::var("AUTH")
            .unwrap_or_else(|_| AuthDriver::Jwt.as_str().into())
            .try_into()
            .expect("Failed to parse AUTH")
    }

    pub fn load() -> Result<AppConfig, config::ConfigError> {
        let environment: Environment = Self::env();
        let environment_path = format!("{}.yaml", environment.as_str());

        let environment_database: DatabaseDriver = Self::db();
        let database_path = format!(
            "database/{}/{}.yaml",
            environment_database.as_str(),
            environment.as_str(),
        );

        let environment_cache = Self::cache();
        let cache_path = format!(
            "cache/{}/{}.yaml",
            environment_cache.as_str(),
            environment.as_str(),
        );

        let auth_driver = Self::auth();
        let auth_path = format!("auth/{}.yaml", auth_driver.as_str(),);

        let value = match environment {
            Environment::Dev => "../../app/configuration",
            Environment::Prod => "/app/configuration",
        };
        let config_dir = std::path::PathBuf::from(value);

        let settings = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.yaml")).required(true))
            .add_source(config::File::from(config_dir.join(environment_path)))
            .add_source(config::File::from(config_dir.join(database_path)))
            .add_source(config::File::from(config_dir.join(cache_path)))
            .add_source(config::File::from(config_dir.join(auth_path)))
            .build()?;

        settings.try_deserialize::<AppConfig>()
    }
}
