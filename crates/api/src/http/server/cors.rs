use actix_cors::Cors;

use shared::config::AnzarConfiguration;

pub fn configure_cors(configuration: &AnzarConfiguration) -> Cors {
    let cors_config = configuration.server.cors.clone();
    let allowed_origins = configuration.server.cors.allowed_origins.clone();

    // NOTE maybe implement cors mannually and remove this package
    if cors_config.enabled {
        Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                if let Ok(origin_str) = origin.to_str() {
                    return allowed_origins.contains(&origin_str.to_string());
                }
                false
            })
            .allowed_methods(
                cors_config
                    .allowed_methods
                    .iter()
                    .map(|m| m.as_str())
                    .collect::<Vec<&str>>(),
            )
            .allowed_headers(
                cors_config
                    .allowed_headers
                    .iter()
                    .map(|h| h.as_str())
                    .collect::<Vec<&str>>(),
            )
            .max_age(cors_config.max_age as usize)
            .supports_credentials()
    } else {
        Cors::default()
    }
}
