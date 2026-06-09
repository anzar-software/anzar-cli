use actix_web::middleware;
use shared::config::AnzarConfiguration;

pub fn build_default_headers(configuration: &AnzarConfiguration) -> middleware::DefaultHeaders {
    configuration
        .security
        .headers
        .iter()
        .fold(middleware::DefaultHeaders::new(), |acc, (key, value)| {
            acc.add((key.as_str(), value.as_str()))
        })
}
