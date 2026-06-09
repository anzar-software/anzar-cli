use actix_session::{
    SessionMiddleware,
    storage::{CookieSessionStore, RedisSessionStore},
};
use actix_web::cookie::Key;

use shared::config::{AnzarConfiguration, SameSiteConfig, SessionConfig};

fn to_same_site(value: SameSiteConfig) -> actix_web::cookie::SameSite {
    match value {
        SameSiteConfig::Strict => actix_web::cookie::SameSite::Strict,
        SameSiteConfig::Lax => actix_web::cookie::SameSite::Lax,
        SameSiteConfig::None => actix_web::cookie::SameSite::None,
    }
}

pub fn configure_cookie_session(
    configuration: &AnzarConfiguration,
) -> SessionMiddleware<CookieSessionStore> {
    let session_config = match configuration.auth.session() {
        Ok(config) => config,
        Err(_) => &SessionConfig {
            name: "id".into(),
            max_age: 3600,
            secure: true,
            http_only: true,
            same_site: shared::config::SameSiteConfig::default(),
        },
    };

    let key = Key::from(configuration.security.secret_key.as_bytes());

    SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_secure(session_config.secure)
        .cookie_same_site(to_same_site(session_config.same_site.clone()))
        .cookie_http_only(session_config.http_only)
        .cookie_name(session_config.name.clone())
        .build()
}

pub async fn _configure_redis_session(
    configuration: &AnzarConfiguration,
) -> SessionMiddleware<RedisSessionStore> {
    let session_config = match configuration.auth.session() {
        Ok(config) => config,
        Err(_) => &SessionConfig {
            name: "id".into(),
            max_age: 3600,
            secure: true,
            http_only: true,
            same_site: shared::config::SameSiteConfig::default(),
        },
    };

    let store = RedisSessionStore::new(&configuration.database.cache.url)
        .await
        .unwrap();
    let key = Key::from(configuration.security.secret_key.as_bytes());

    SessionMiddleware::builder(store, key)
        .cookie_secure(session_config.secure)
        .cookie_same_site(to_same_site(session_config.same_site.clone()))
        .cookie_http_only(session_config.http_only)
        .cookie_name(session_config.name.clone())
        .build()
}
