use std::fs;

use uuid::Uuid;

use crate::adapters::cache::CacheAdapters;
use crate::adapters::cache::memcache::MemCache;
use crate::adapters::database::{DatabaseAdapters, mongodb::MongoDB, sqlite::SQLite};
use crate::config::database::cache_driver::CacheDriver;
use crate::config::{
    AnzarConfiguration, App, AppConfig, Authentication, Cache, Database, DatabaseDriver, Server,
};
use crate::error::Result;
use crate::scopes::auth::service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub configuration: AnzarConfiguration,
}

impl AppState {
    pub async fn production(app_config: &AppConfig) -> Result<Self> {
        let content = fs::read_to_string(&app_config.config_path)?;
        let configuration: AnzarConfiguration = serde_yaml::from_str(content.as_str())?;
        let auth_service = AuthService::from_database(&configuration.database).await?;

        Ok(Self {
            auth_service,
            configuration,
        })
    }

    pub async fn testing(address: &str) -> Result<Self> {
        let configuration = Self::build_config(address).await.expect("booo");
        let auth_service = Self::build_authservice(&configuration.database).await?;

        Ok(Self {
            auth_service,
            configuration,
        })
    }

    async fn build_config(address: &str) -> Result<AnzarConfiguration> {
        let mut app_config = AppConfig::load().expect("Failed to read configuration");

        let mut configuration = match fs::read_to_string(&app_config.config_path) {
            Ok(content) => {
                let mut configuration: AnzarConfiguration = serde_yaml::from_str(content.as_str())?;

                configuration.app.url = address.into();
                configuration.database.driver = app_config.database.driver;

                configuration
            }
            Err(_) => AnzarConfiguration {
                app: App {
                    environment: "dev".into(),
                    url: address.into(),
                },
                database: Database {
                    driver: app_config.database.driver,
                    connection_string: "".into(),
                    cache: Cache {
                        driver: CacheDriver::MemCached,
                        url: "memcache://localhost:11211".into(),
                    },
                },
                server: Server::default(),
                auth: Authentication::default(),
                security: super::Security {
                    secret_key: "f8afd6dc9f2352e2dfff4b789e3458448a000aa4fb7010d379b998bec89679cd"
                        .into(),
                    headers: vec![],
                },
            },
        };

        if configuration.database.driver == DatabaseDriver::MongoDB {
            let db_name = Uuid::new_v4().to_string();
            app_config.database.name = db_name;
        }

        configuration.database.connection_string = app_config.database.connection_string();
        Ok(configuration)
    }

    async fn build_authservice(database: &Database) -> Result<AuthService> {
        let cache_adapter = match database.cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&database.cache.url).await?;
                CacheAdapters::memcached(client)
            }
            CacheDriver::Redis => todo!(),
        };

        let database_adapter = match database.driver {
            DatabaseDriver::SQLite => {
                let db = SQLite::start(&database.connection_string).await?;
                if &database.connection_string == "sqlite::memory:" {
                    // let path = std::path::Path::new("migrations");
                    // if path.exists() {
                    //     let migrator = Migrator::new(path).await?;
                    //     migrator.run(&db).await.expect("migrations to run");
                    // }

                    sqlx::migrate!("./migrations")
                        .run(&db)
                        .await
                        .expect("migrations to run");
                }
                DatabaseAdapters::sqlite(&db)
            }
            DatabaseDriver::MongoDB => {
                let client = MongoDB::start(&database.connection_string).await?;
                let db_name = database.name().unwrap_or_default();
                DatabaseAdapters::mongodb(&client, db_name)
            }
            DatabaseDriver::PostgreSQL => todo!(),
        };

        Ok(AuthService::new(
            database_adapter,
            database.driver,
            cache_adapter,
        ))
    }
}
