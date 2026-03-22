use crate::config::Configuration;
use actix_web::middleware;

pub fn build_default_headers(configuration: &Configuration) -> middleware::DefaultHeaders {
    configuration
        .security
        .headers
        .iter()
        .fold(middleware::DefaultHeaders::new(), |acc, (key, value)| {
            acc.add((key.as_str(), value.as_str()))
        })
}
