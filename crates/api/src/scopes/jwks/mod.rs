use actix_web::web;
use actix_web::{HttpResponse, Scope};
use serde_json::json;

use crate::error::Result;
use crate::http::extractors::AppStateExtractor;

// #[tracing::instrument(name = "Jwks")]
async fn load_jwks(AppStateExtractor(app_state): AppStateExtractor) -> Result<HttpResponse> {
    let mut response = Vec::new();

    let signing_keys = app_state.key_service.get_jwks().await?;
    for signing_key in signing_keys {
        let key = signing_key.key;
        let jwk = app_state.crypto.openssl.clone().pem_to_jwk(key)?;

        response.push(jwk);
    }

    let payload = json!({ "keys": response });

    Ok(HttpResponse::Ok()
        .insert_header((
            actix_web::http::header::CACHE_CONTROL,
            "public, max-age=3600",
        ))
        .json(payload))
}

pub fn jwks_scope() -> Scope {
    web::scope("/.well-known/jwks.json").route("", web::get().to(load_jwks))
}
