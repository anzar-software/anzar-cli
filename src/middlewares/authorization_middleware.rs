use actix_web::{
    Error, HttpMessage, ResponseError,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

use crate::{error::ForbiddenReason, extract_service_response};
use crate::{
    error::{Error as AuthError, TokenErrorType},
    services::account::service::AccountServiceTrait,
};

use super::support;
use crate::scopes::user::{User, service::UserServiceTrait};
use crate::{extractors::Claims, services::session::model::Session};

async fn validate_user(req: &ServiceRequest, user_id: &str) -> Result<User, AuthError> {
    let auth_service = support::extract_auth_service(req)?;

    let user: User = auth_service.find_user(user_id).await?;
    let account = auth_service.find_account(user_id).await?;

    if account.locked {
        tracing::warn!(
            user.id = %user_id,
            error.code = "ForbiddenReason::AccountSuspended",
            "Account is suspended"
        );
        return Err(AuthError::Forbidden(ForbiddenReason::AccountSuspended));
    }

    Ok(user)
}

fn extract_user_id_from_extensions(req: &ServiceRequest) -> Result<String, AuthError> {
    let configuration = support::extract_configuration_service(req)?;

    match configuration.auth.strategy {
        crate::config::AuthStrategy::Session => {
            if let Some(session) = req.extensions().get::<Session>() {
                return Ok(session.user_id.clone());
            }

            tracing::error!(
                error.code = "AuthError::TokenInvalid",
                "Session token was not saved in actix extenstion"
            );
            Err(AuthError::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::SessionToken,
                },
            ))
        }
        crate::config::AuthStrategy::Jwt => {
            if let Some(claims) = req.extensions().get::<Claims>() {
                return Ok(claims.sub.clone());
            }

            tracing::error!(
                error.code = "AuthError::TokenInvalid",
                "Access token (Claims) was not saved in actix extenstion"
            );
            Err(AuthError::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::AccessToken,
                },
            ))
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
