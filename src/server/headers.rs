use crate::config::AnzarConfiguration;
use actix_web::middleware;

pub fn build_default_headers(configuration: &AnzarConfiguration) -> middleware::DefaultHeaders {
    configuration
        .security
        .headers
        .iter()
        .fold(middleware::DefaultHeaders::new(), |acc, (key, value)| {
            acc.add((key.as_str(), value.as_str()))
        })
}
