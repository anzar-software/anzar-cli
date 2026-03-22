use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;

use crate::config::Configuration;

pub fn configure_session(configuration: &Configuration) -> SessionMiddleware<CookieSessionStore> {
    let session_config = configuration.auth.session.clone();

    let key = Key::from(configuration.security.secret_key.as_bytes());
    SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_secure(session_config.secure)
        .cookie_same_site(session_config.same_site.clone().into())
        .cookie_http_only(session_config.http_only)
        .cookie_name(session_config.name.clone())
        .build()
}
