use actix_session::SessionExt;
use actix_web::{
    Error, HttpMessage,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
};

use crate::error::{Error as AuthError, TokenErrorType};
use crate::extract_service_response;
use crate::http::{cookies, extractors::extract_app_state};

use shared::{
    application::model::{Claims, Session, TokenType},
    config::AuthStrategy,
    error::CoreError,
};

fn extract_token_from_header(req: &ServiceRequest, key: String) -> Option<&str> {
    req.headers()
        .get(key)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

async fn validate_token(req: &ServiceRequest) -> Result<(), Error> {
    let app_state = extract_app_state(req)?;

    match app_state.configuration.auth.strategy {
        AuthStrategy::Session(..) => {
            let data: Option<String> = req.get_session().get(cookies::SESSION_COOKIE)?;

            if let Some(session_id) = data {
                let session = app_state
                    .session_service
                    .find_session(&session_id)
                    .await
                    .map_err(crate::error::Error::from)?;

                if chrono::Utc::now() > session.expires_at {
                    tracing::error!(
                        error.code = "AuthError::TokenExpired",
                        "Session token is expired"
                    );

                    return Err(AuthError::Core(CoreError::from(
                        shared::error::AuthError::TokenExpired {
                            token_type: TokenErrorType::SessionToken,
                            expired_at: session.expires_at,
                        },
                    ))
                    .into());
                }

                // NOTE Only expires after true inactivity period
                app_state
                    .session_service
                    .extend_timeout(session.id().map_err(crate::error::Error::from)?)
                    .await
                    .map_err(crate::error::Error::from)?;

                req.extensions_mut().insert::<Session>(session);
            }
        }
        AuthStrategy::Jwt(..) => {
            let access_token = extract_token_from_header(req, header::AUTHORIZATION.to_string());

            if let Some(token) = access_token {
                let claims: Claims = app_state
                    .crypto
                    .jwt()
                    .map_err(crate::error::Error::from)?
                    .decode(token)
                    .map_err(crate::error::Error::from)?;

                if claims.token_type != TokenType::AccessToken {
                    tracing::error!(
                        error.code = "AuthError::TokenInvalid",
                        "Expected an AccessToken, got a RefreshToken"
                    );

                    return Err(AuthError::Core(CoreError::from(
                        shared::error::AuthError::TokenInvalid {
                            token_type: TokenErrorType::AccessToken,
                        },
                    ))
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
