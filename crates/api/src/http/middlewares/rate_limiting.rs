use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};
use std::{net::IpAddr, str::FromStr};

use crate::http::extractors::extract_app_state;

fn extract_ipadd(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| IpAddr::from_str(s.trim()).ok())
        .or_else(|| req.peer_addr().map(|a| a.ip()))
        .map(|a| a.to_canonical().to_string())
}

pub async fn ip_rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let app_state = extract_app_state(&req)?;
    let key = extract_ipadd(&req).unwrap_or_else(|| "unknown".to_string());

    let ratelimit_config = app_state.configuration.security.rate_limit;
    if ratelimit_config.enabled {
        app_state
            .rate_limiter
            .check(&key, &ratelimit_config.ip)
            .map_err(crate::error::Error::from)?;
    }

    next.call(req).await
}
