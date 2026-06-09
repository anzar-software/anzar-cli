use actix_web::{
    Error, HttpMessage, ResponseError,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

use crate::error::{Error as AuthError, TokenErrorType};
use crate::http::extractors::extract_app_state;

use shared::{
    application::model::{Claims, Session, User},
    error::{CoreError, ForbiddenReason},
};

use crate::extract_service_response;

async fn validate_user(req: &ServiceRequest, user_id: &str) -> Result<User, AuthError> {
    let app_state = extract_app_state(req)?;

    let user = app_state.auth_service.find_user(user_id).await?;
    let account = app_state.auth_service.find_account(user_id).await?;

    if account.locked {
        tracing::warn!(
            user.id = %user_id,
            error.code = "ForbiddenReason::AccountSuspended",
            "Account is suspended"
        );
        return Err(CoreError::from(ForbiddenReason::AccountSuspended).into());
    }

    Ok(user)
}

fn extract_user_id_from_extensions(req: &ServiceRequest) -> Result<String, AuthError> {
    let app_state = extract_app_state(req)?;

    match app_state.configuration.auth.strategy {
        shared::config::AuthStrategy::Session(..) => {
            if let Some(session) = req.extensions().get::<Session>() {
                return Ok(session.user_id.clone());
            }

            tracing::error!(
                error.code = "AuthError::TokenInvalid",
                "Session token was not saved in actix extenstion"
            );

            Err(AuthError::Core(CoreError::from(
                shared::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::SessionToken,
                },
            )))

            // Err(AuthError::Unauthenticated(
            //     crate::error::AuthError::TokenInvalid {
            //         token_type: TokenErrorType::SessionToken,
            //     },
            // ))
        }
        shared::config::AuthStrategy::Jwt(..) => {
            if let Some(claims) = req.extensions().get::<Claims>() {
                return Ok(claims.sub.clone());
            }

            tracing::error!(
                error.code = "AuthError::TokenInvalid",
                "Access token (Claims) was not saved in actix extenstion"
            );
            Err(AuthError::Core(CoreError::from(
                shared::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::AccessToken,
                },
            )))
        }
    }
}

pub async fn authorization_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // pre-processing
    let user_id = extract_service_response!(req, extract_user_id_from_extensions(&req));
    let user = extract_service_response!(req, validate_user(&req, &user_id).await);

    req.extensions_mut().insert::<User>(user);
    next.call(req).await
    // post-processing
}
