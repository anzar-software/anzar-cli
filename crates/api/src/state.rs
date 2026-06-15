use shared::intern::db::CacheDb;
use shared::utils::crypto::JwtSigner;
use sqlx::types::Uuid;
use std::env::var;

use shared::config::database::DatabaseDriver;
use shared::config::{AnzarConfiguration, AppConfig};
use shared::infrastructure::database::postgres::PostgreSQL;
use shared::intern::{
    auth::{AuthContext, AuthService},
    db::DB,
    key::KeyService,
    rbac::RbacService,
    session::SessionService,
};
use shared::utils::{crypto::Crypto, rate_limiting::RateLimiter};

use crate::error::{Error, Result};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub session_service: SessionService,
    pub rbac_service: RbacService,
    pub key_service: KeyService,
    pub crypto: Crypto,
    pub configuration: AnzarConfiguration,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn _auth_ctx(&self) -> AuthContext<'_> {
        AuthContext {
            service: &self.auth_service,
            crypto: &self.crypto,
            configuration: &self.configuration,
        }
    }
}

impl AppState {
    pub async fn new(anzar_config_path: &str) -> Result<Self> {
        dotenvy::dotenv().ok();

        let env_overrides =
            config::Environment::default().source(Some(std::collections::HashMap::from([
                (
                    "SECURITY.SECRET_KEY".into(),
                    var("SECRET_KEY").map_err(Error::from)?,
                ),
                (
                    "DATABASE.CONNECTION_STRING".into(),
                    var("DATABASE_URL").map_err(Error::from)?,
                ),
                (
                    "DATABASE.CACHE.URL".into(),
                    var("CACHE_URL").map_err(Error::from)?,
                ),
            ])));

        let configuration = config::Config::builder()
            .add_source(config::File::with_name(anzar_config_path))
            .add_source(env_overrides)
            .build()
            .map_err(Error::from)?
            .try_deserialize::<AnzarConfiguration>()
            .map_err(Error::from)?;
        configuration.validate()?;

        let crypto = Crypto::from_configuration(&configuration);
        let database = DB::connect(&configuration.database)
            .await
            .map_err(Error::from)?;
        let cache = CacheDb::connect(&configuration.database.cache)
            .await
            .map_err(Error::from)?;

        let auth_service = AuthService::new(&database, &cache, &crypto, &configuration);
        let session_service = SessionService::new(&database, &crypto, &configuration);
        let rbac_service = RbacService::new(&database, &configuration.auth.rbac);
        let key_service = KeyService::new(&database, &crypto, &configuration);

        Ok(Self {
            crypto,
            auth_service,
            session_service,
            rbac_service,
            key_service,
            configuration,
            rate_limiter: RateLimiter::default(),
        })
    }
}

impl AppState {
    async fn sync_signing_keys(self) -> Result<Crypto> {
        let mut crypto = Crypto::from_configuration(&self.configuration);

        if let Ok(jwt_config) = self.configuration.auth.jwt() {
            let keys = self.key_service.load_or_create().await?;
            crypto = crypto.with_jwt(keys, jwt_config);
        }

        Ok(crypto)
    }

    pub async fn startup(self) -> Result<JwtSigner> {
        if self.configuration.auth.rbac.enabled {
            self.rbac_service.sync_permission().await?;
        }

        let crypto = self.sync_signing_keys().await?;
        let jwt = crypto.jwt()?;
        Ok(jwt)
    }
}

impl AppState {
    pub async fn testing(address: &str) -> Result<Self> {
        let configuration = build_config(address).await?;
        configuration.validate()?;

        let crypto = Crypto::from_configuration(&configuration);
        let database = DB::connect(&configuration.database)
            .await
            .map_err(Error::from)?;
        let cache = CacheDb::connect(&configuration.database.cache)
            .await
            .map_err(Error::from)?;

        let auth_service = AuthService::new(&database, &cache, &crypto, &configuration);
        let session_service = SessionService::new(&database, &crypto, &configuration);
        let rbac_service = RbacService::new(&database, &configuration.auth.rbac);
        let key_service = KeyService::new(&database, &crypto, &configuration);

        Ok(Self {
            crypto,
            auth_service,
            session_service,
            rbac_service,
            key_service,
            configuration,
            rate_limiter: RateLimiter::default(),
        })
    }
}

async fn build_config(address: &str) -> Result<AnzarConfiguration> {
    let mut app_config = AppConfig::load()?;

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

    let secret = "40146ea996771990c4912566e14795d65d2cbd90988d03ba9a0a94943a6b8866";
    Ok(AnzarConfiguration::new(app_config)
        .with_appurl(address)
        .with_secret(secret))
}
