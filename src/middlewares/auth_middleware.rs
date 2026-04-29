use actix_session::SessionExt;
use actix_web::{
    Error, HttpMessage,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
};

use crate::extract_service_response;
use crate::{config::AuthStrategy, extractors::Claims};

use crate::scopes::auth::support as AuthSupport;
use crate::services::session::{model::Session, service::SessionServiceTrait};
use crate::{
    error::{Error as AuthError, TokenErrorType},
    services::jwt::JwtDecoder,
};

use super::support;

fn extract_token_from_header(req: &ServiceRequest, key: String) -> Option<&str> {
    req.headers()
        .get(key)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

async fn find_session(req: &ServiceRequest, token: &str) -> Result<Session, AuthError> {
    let auth_service = support::extract_auth_service(req)?;
    auth_service.find_session(token).await
}
async fn update_session_expiray(req: &ServiceRequest, id: &str) -> Result<Session, AuthError> {
    let auth_service = support::extract_auth_service(req)?;
    auth_service.extend_timeout(id).await
}

async fn validate_token(req: &ServiceRequest) -> Result<(), Error> {
    let configuration = support::extract_configuration_service(req)?;

    match configuration.auth.strategy {
        AuthStrategy::Session => {
            let req_session = req.get_session();
            let data = req_session.get::<String>(AuthSupport::SESSION_COOKIE)?;

            if let Some(token) = data {
                let session = find_session(req, &token).await?;
                let session_id = session.id()?;

                if session.used_at.is_some() {
                    tracing::error!(
                        error.code = "AuthError::TokenReplay",
                        "Session token was already used"
                    );
                    return Err(
                        AuthError::Unauthenticated(crate::error::AuthError::TokenReplay {
                            token_type: TokenErrorType::SessionToken,
                        })
                        .into(),
                    );
                }

                if chrono::Utc::now() > session.expires_at {
                    tracing::error!(
                        error.code = "AuthError::TokenExpired",
                        "Session token is expired"
                    );
                    return Err(AuthError::Unauthenticated(
                        crate::error::AuthError::TokenExpired {
                            token_type: TokenErrorType::SessionToken,
                            expired_at: session.expires_at,
                        },
                    )
                    .into());
                }

                // NOTE Only expires after true inactivity period
                update_session_expiray(req, session_id).await?;

                req.extensions_mut().insert::<Session>(session);
            }
        }
        AuthStrategy::Jwt => {
            let access_token = extract_token_from_header(req, header::AUTHORIZATION.to_string());
            if let Some(token) = access_token {
                let claims: Claims = JwtDecoder::new(token, &configuration).decode()?;
                if claims.token_type != crate::extractors::TokenType::AccessToken {
                    tracing::error!(
                        error.code = "AuthError::TokenInvalid",
                        "Expected an AccessToken, got a RefreshToken"
                    );
                    return Err(AuthError::Unauthenticated(
                        crate::error::AuthError::TokenInvalid {
                            token_type: TokenErrorType::AccessToken,
                        },
                    )
                    .into());
                }

                req.extensions_mut().insert::<Claims>(claims.clone());
                tracing::info!(user.id = %claims.sub, "User authenticated successfully");
            }
        }
    };

    Ok(())
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // pre-processing
    extract_service_response!(req, validate_token(&req).await);
    next.call(req).await
    // post-processing
}
