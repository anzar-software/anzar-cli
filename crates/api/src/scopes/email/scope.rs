use actix_web::{
    HttpResponse, Scope,
    web::{self},
};
use secrecy::ExposeSecret;

use crate::error::{ErrorResponse, Result};
use crate::http::extractors::{AppStateExtractor, ValidatedQuery};

use shared::application::model::TokenQuery;

#[utoipa::path(
    post,
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
#[actix_web::post("/verify")]
async fn verify_email(
    AppStateExtractor(app_state): AppStateExtractor,
    ValidatedQuery(query): ValidatedQuery<TokenQuery>,
) -> Result<HttpResponse> {
    let token = query.token;

    let email_verificaiton_token = app_state
        .auth_service
        .consume_email_verification_token(token.expose_secret())
        .await?;

    app_state
        .auth_service
        .validate_account(&email_verificaiton_token.user_id)
        .await?;

    let email_config = app_state.configuration.auth.email;
    let success_redirect = match email_config.verification.success_redirect {
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
