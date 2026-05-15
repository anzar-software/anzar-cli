use actix_web::{
    HttpResponse, Scope,
    web::{self},
};
use secrecy::ExposeSecret;

use crate::error::{ErrorResponse, Result};

use crate::application::model::TokenQuery;
use crate::application::traits::{AccountServiceTrait, EmailVerificationTokenServiceTrait};
use crate::http::extractors::{AppStateExtractor, ValidatedQuery};

#[utoipa::path(
    get,
    path = "/email/verify",
    tag = "Email",
    summary = "Verify user email",
    description = "Validates the email verification token and activates the user account. \
                   On success, redirects the user to the confirmation page.",
    params(
        ("token" = String, Query, description = "One-time email verification token issued during registration"),
    ),
    responses(
        (
            status = FOUND,
            description = "Token is valid — redirects to the success page",
            headers(
                ("Location" = String, description = "URL of the post-verification success page")
            )
        ),
        (
            status = BAD_REQUEST,
            description = "Token is missing, malformed, or already used",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = INTERNAL_SERVER_ERROR,
            description = "Unexpected server error",
            body = ErrorResponse,
            content_type = "application/json"
        ),
    ),
    security(()),
    )]
#[tracing::instrument(name = "Email Verification", skip(app_state, query))]
#[actix_web::get("/verify")]
async fn verify_email(
    AppStateExtractor(app_state): AppStateExtractor,
    ValidatedQuery(query): ValidatedQuery<TokenQuery>,
) -> Result<HttpResponse> {
    let token = query.token;

    // FIXME merge into one
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
    web::scope("/email").service(verify_email)
}
