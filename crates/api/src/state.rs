use std::collections::HashMap;
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
use shared::utils::crypto::SecureToken;
use shared::utils::{crypto::Crypto, rate_limiting::RateLimiter};
use sqlx::types::Uuid;

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
        let db = DB::new(&configuration.database)
            .await
            .map_err(Error::from)?;

        let auth_service = AuthService::new(&db.database, &db.cache, &crypto, &configuration);
        let session_service = SessionService::new(&db.database, &crypto, &configuration);
        let rbac_service = RbacService::new(&db.database);
        let key_service = KeyService::new(&db.database, &crypto, &configuration);

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
    fn resolve_role_permissions(&self) -> HashMap<String, Vec<String>> {
        let rbac = &self.configuration.auth.rbac;

        let mut hashmap: HashMap<String, Vec<String>> = rbac
            .roles
            .iter()
            .map(|role| (role.name.clone(), role.permissions.clone()))
            .collect();

        for role in &rbac.roles {
            for role_name in &role.inherits {
                let inherited_permissions = hashmap.get(role_name).cloned().unwrap_or_default();

                let entry = hashmap.entry(role.name.clone()).or_default();
                for perm in inherited_permissions {
                    if !entry.contains(&perm) {
                        entry.push(perm);
                    }
                }
            }
        }

        hashmap
    }

    async fn sync_rbac(self) -> Result<()> {
        let rbac = &self.configuration.auth.rbac;

        if !rbac.enabled {
            return Ok(());
        }

        let hashmap = self.resolve_role_permissions();

        for role in &rbac.roles {
            let permissions = hashmap.get(&role.name).cloned().unwrap_or_default();

            let role_id = self.rbac_service.upsert_role(&role.name).await?;
            let permission_ids = self
                .rbac_service
                .upsert_permissions(permissions.clone())
                .await?;

            self.rbac_service
                .upsert_role_permissions(&role_id, permission_ids)
                .await?;
        }

        Ok(())
    }

    async fn sync_signing_keys(self) -> Result<Crypto> {
        let crypto = if let Ok(jwt_config) = self.configuration.auth.jwt() {
            let (private_key, signing_key) = match self.key_service.load_active_key().await {
                Ok(response) => response,
                Err(_) => {
                    self.key_service.save_new_key().await?;
                    self.key_service.load_active_key().await?
                }
            };

            Crypto::from_configuration(&self.configuration).with_jwt(
                &private_key,
                &signing_key,
                jwt_config,
            )
        } else {
            Crypto::from_configuration(&self.configuration)
        };

        Ok(crypto)
    }

    pub async fn startup(self) -> Result<Crypto> {
        let _ = self.clone().sync_rbac().await;

        let crypto = self.clone().sync_signing_keys().await?;
        Ok(crypto)
    }

    pub async fn collect_user_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        let mut full_permissions: Vec<String> = Vec::new();

        if !&self.configuration.auth.rbac.enabled {
            return Ok(full_permissions);
        }

        let hashmap = self.resolve_role_permissions();

        let roles = self.rbac_service.find_roles_by_user_id(user_id).await?;
        for role in roles {
            let mut permissions = hashmap.get(&role.name).cloned().unwrap_or_default();
            full_permissions.append(&mut permissions);
        }

        Ok(full_permissions)

        // let mut full_permissions: Vec<String> = Vec::new();
        //
        // if !&self.configuration.auth.rbac.enabled {
        //     return Ok(full_permissions);
        // }
        //
        // let roles = self.rbac_service.find_roles_by_user_id(user_id).await?;
        // for role in roles {
        //     let role_id = role.id()?;
        //     let response = self
        //         .rbac_service
        //         .find_permissions_by_role_id(role_id)
        //         .await?;
        //
        //     let mut permissions = response.iter().map(|r| r.name.clone()).collect();
        //     full_permissions.append(&mut permissions);
        // }
        //
        // Ok(full_permissions)
    }
}

impl AppState {
    pub async fn testing(address: &str) -> Result<Self> {
        let configuration = build_config(address).await?;
        configuration.validate()?;

        let crypto = Crypto::from_configuration(&configuration);
        let db = DB::new(&configuration.database)
            .await
            .map_err(Error::from)?;

        let auth_service = AuthService::new(&db.database, &db.cache, &crypto, &configuration);
        let session_service = SessionService::new(&db.database, &crypto, &configuration);
        let rbac_service = RbacService::new(&db.database);
        let key_service = KeyService::new(&db.database, &crypto, &configuration);

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
