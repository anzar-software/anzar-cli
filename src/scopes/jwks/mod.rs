use actix_web::web;
use actix_web::{HttpResponse, Scope};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use openssl::rsa::Rsa;
use serde_json::json;

use crate::application::traits::SigningKeysServiceTrait;
use crate::http::extractors::AppStateExtractor;

use crate::error::Result;

// #[tracing::instrument(name = "Jwks")]
async fn load_jwks(AppStateExtractor(app_state): AppStateExtractor) -> Result<HttpResponse> {
    let signing_keys = app_state.find_signing_keys().await?;

    let mut response = Vec::new();

    for signing_key in signing_keys {
        let key = signing_key.key;
        let public_key_pem: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(key.public_key).unwrap();

        let rsa =
            Rsa::public_key_from_pem(&public_key_pem).expect("Failed to parse public key PEM");

        let n = BASE64_URL_SAFE_NO_PAD.encode(rsa.n().to_vec());
        let e = BASE64_URL_SAFE_NO_PAD.encode(rsa.e().to_vec());

        response.push(json!({
            "kty": key.kty,
            "alg": key.algorithm,
            "use": "sig",
            "kid": key.kid,
            "n": n,
            "e": e
        }))
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
