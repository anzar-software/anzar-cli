use actix_web::{
    HttpResponse, Scope,
    web::{self},
};
use secrecy::ExposeSecret;

use crate::error::{ErrorResponse, Result};
use crate::extractors::{AppStateExtractor, ValidatedQuery};
use crate::scopes::{
    auth::TokenQuery, email::service::EmailVerificationTokenServiceTrait,
    user::service::UserServiceTrait,
};

#[utoipa::path(
    get,
    path = "/email",
    tag = "Email",
    summary = "Verify user email",
    description = "Validates the email token and update the user account.",
    params(
        ("token" = TokenQuery, Query, description = "Email Verification Token")
    ),
    responses(
        (status = 302, description = "Redirect to success page", 
         headers(
             ("Location" = String, description = "Redirect URL")
         )
        ),
        (status = BAD_REQUEST, description = "invalid request", body = ErrorResponse),
    ),
)]
#[tracing::instrument(name = "Email Verification", skip(app_state, query))]
async fn verify_email(
    AppStateExtractor(app_state): AppStateExtractor,
    // FIXME to be validated
    ValidatedQuery(query): ValidatedQuery<TokenQuery>,
) -> Result<HttpResponse> {
    let token = query.token;

    let email_verificaiton_token = app_state
        .validate_email_verification_token(token.expose_secret())
        .await?;

    let verification_token_id = email_verificaiton_token.id()?;
    app_state
        .invalidate_email_verification_token(verification_token_id)
        .await?;

    app_state
        .validate_account(&email_verificaiton_token.user_id)
        .await?;

    let success_redirect = match app_state
        .configuration
        .auth
        .email
        .verification
        .success_redirect
    {
        Some(url) => url,
        None => app_state.configuration.app.url,
    };
    Ok(HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, success_redirect))
        .finish())
}

pub fn email_scope() -> Scope {
    web::scope("/email").route("/verify", web::get().to(verify_email))
}
