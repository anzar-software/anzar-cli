use actix_web::{
    HttpResponse, Scope,
    middleware::from_fn,
    web::{self},
};
use secrecy::ExposeSecret;
use validator::{Validate, ValidateArgs};

use crate::{
    application::traits::UserRoleServiceTrait,
    error::{
        AuthError, ConflictReason, CredentialField, Error, ErrorResponse, ForbiddenReason, Result,
    },
};

use crate::config::AuthStrategy;

use crate::application::model::{
    AccountStatus, CreateUserOutcome, ExpiringLink, LoginRequest, RegisterRequest,
    ResetPasswordRequest, Session, TokenQuery, User,
};
use crate::application::traits::{
    AccountServiceTrait, EmailVerificationTokenServiceTrait, JwtServiceTrait,
    PasswordResetTokenServiceTrait, SessionServiceTrait, UserServiceTrait,
};
use crate::http::{
    cookies,
    extractors::{AppStateExtractor, ValidatedQuery},
    middlewares::{auth_middleware, authorization_middleware, rate_limiting::RATE_LIMITS},
};

use super::model::{AuthResponse, EmailRequest, RefreshTokenRequest};
use super::utils as throttle;

#[utoipa::path(
        post,
        path = "/auth/login",
        tag = "Auth",
        operation_id = "login",
        summary = "User login",
        description = "Authenticates a user with email and password.\n\n\
            **Session strategy**: Returns a session cookie (`id`) managed by the server.\n\
            **JWT strategy**: Returns `access` and `refresh` in the response body.",
        security(()),
        request_body(
            description = "User login credentials",
            content = LoginRequest,
            content_type = "application/json"
        ),
        responses(
            (
                status = OK,
                description = "Authentication successful",
                body = AuthResponse,
                content_type = "application/json",
                headers(
                    ("Set-Cookie" = String,
                     description = "Session cookie (session strategy only). \
                                    Format: id=<session_id>; HttpOnly; Secure; SameSite=Strict")
                )
            ),
            (
                status = BAD_REQUEST,
                description = "Malformed request or validation error",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = UNAUTHORIZED,
                description = "Invalid email or password",
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
    )]
#[tracing::instrument(
    name = "auth.login"
    skip(req, app_state, session),
    fields(
        user.id = tracing::field::Empty,
        user.email = %req.email,
        login.attempted = true,
        attempts.remaining = tracing::field::Empty
    )
)]
#[actix_web::post("/login")]
pub async fn login(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<LoginRequest>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let auth_config = &app_state.configuration.auth;
    req.validate_with_args(&auth_config.password.requirements)?;

    // START TIMING-SAFE EXECUTION BLOCK
    let start = std::time::Instant::now();

    // Fetch device cookie
    let raw = session.get::<String>(cookies::DEVICE_COOKIE).ok().flatten();
    let device_cookie = raw.as_deref();

    // ALWAYS execute the same operations regardless of user existence
    let (user, account, account_status, attempts) =
        app_state.authenticate_user(&req, device_cookie).await?;

    // ENFORCE CONSTANT-TIME EXECUTION
    throttle::throttle_since(start).await;

    // Now handle responses
    // NOTE this can be removed, if it's not needed
    if auth_config.email.verification.required && !account.verified {
        tracing::warn!(
            user_id = %user.id()?,
            error_code = "AuthError::AccountNotVerified",
            "Email verification is enabled - Account not verified"
        );
        return Err(Error::Unauthenticated(AuthError::AccountNotVerified));
    }

    match account_status {
        AccountStatus::Active => {
            // Issue new device cookie

            let cookie = app_state.crypto.hmac.issue(&req.email)?;
            let user_id = user.id()?;

            match &app_state.configuration.auth.strategy {
                AuthStrategy::Session(..) => {
                    app_state.issue_session(user_id).await.map(|token| {
                        session.clear();
                        session.renew();
                        session.insert(cookies::DEVICE_COOKIE, &cookie)?;
                        session.insert(cookies::SESSION_COOKIE, token)?;
                        Ok(HttpResponse::Ok().json(AuthResponse::new(user)))
                    })?
                }
                AuthStrategy::Jwt(jwt) => app_state.issue_jwt(user_id).await.map(|tokens| {
                    session.clear();
                    session.renew();
                    session.insert(cookies::DEVICE_COOKIE, &cookie)?;

                    Ok(HttpResponse::Ok().json(
                        AuthResponse::new(user).with_jwt(tokens, jwt.access_token_expires_in),
                    ))
                })?,
            }
        }
        // NOTE fully mask account existence
        // only return InvalidCredentials
        AccountStatus::Suspended => Err(Error::Forbidden(ForbiddenReason::AccountSuspended)),
        _ => {
            throttle::delay(attempts as u32).await;
            return Err(Error::Unauthenticated(AuthError::InvalidCredentials {
                field: CredentialField::EmailOrPassword,
            }));
        }
    }
}

#[utoipa::path(
        post,
        path = "/auth/register",
        tag = "Auth",
        operation_id = "register",
        summary = "Register a new user",
        description = "Creates a new user account with email and password.\n\n\
            **Session strategy**: Returns a session cookie (`id`) managed by the server.\n\
            **JWT strategy**: Returns `access` and `refresh` in the response body.",
        security(()),
        request_body(
            description = "User register credentials",
            content = RegisterRequest,
            content_type = "application/json"
        ),
        responses(
            (
                status = CREATED,
                description = "User registered successfully",
                body = AuthResponse,
                content_type = "application/json",
                headers(
                    ("Set-Cookie" = String,
                     description = "Session cookie (session strategy only). \
                     Format: id=<session_id>; HttpOnly; Secure; SameSite=Strict")
                )
            ),
            (
                status = BAD_REQUEST,
                description = "Malformed request or validation error",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = CONFLICT,
                description = "Email already registered",
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
    )]
#[tracing::instrument(
    name = "Register user",
    skip(req, app_state, session),
    fields(user_email = %req.email, user_name = %req.username)
)]
#[actix_web::post("/register")]
pub async fn register(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RegisterRequest>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    req.validate_with_args(&app_state.configuration.auth.password.requirements)?;
    // let mut bucket = RATE_LIMITS.entry(req.email.clone()).or_default();
    // bucket.run()?;

    // START TIMING-SAFE EXECUTION BLOCK
    let start = std::time::Instant::now();

    let result = async {
        match app_state.create_user(req.into_inner()).await? {
            CreateUserOutcome::Created(user) => {
                let mut response = AuthResponse::new(user.clone());

                let user_id = user.id()?;

                if app_state.configuration.auth.rbac.enabled {
                    let role = &app_state.configuration.auth.rbac.default_role;
                    app_state.insert_user_role(user_id, role).await?;
                }

                let email_config = &app_state.configuration.auth.email.verification;
                if email_config.required {
                    let token = app_state.create_verification_email(user_id).await?;
                    let link = format!(
                        "{}/email/verify?token={}",
                        &app_state.configuration.app.url, &token
                    );

                    response = response.with_verification(&link, email_config.token_expires_in);
                }

                let http_response = match &app_state.configuration.auth.strategy {
                    AuthStrategy::Session(..) => {
                        let token = app_state.issue_session(user_id).await?;

                        session.clear();
                        session.renew();
                        session.insert(cookies::SESSION_COOKIE, token)?;

                        response
                    }

                    AuthStrategy::Jwt(jwt) => {
                        let tokens = app_state.issue_jwt(user_id).await?;
                        response.with_jwt(tokens, jwt.access_token_expires_in)
                    }
                };

                Ok(HttpResponse::Created().json(http_response))
            }
            CreateUserOutcome::AlreadyExists => {
                Err(Error::Conflict(ConflictReason::AlreadyExists {
                    field: CredentialField::Email,
                }))
            }
        }
    }
    .await;

    // ENFORCE CONSTANT-TIME EXECUTION
    throttle::throttle_since(start).await;
    result
}

#[utoipa::path(
    get,
    path = "/auth/session",
    tag = "Auth",
    operation_id = "getSession",
    summary = "Get current session",
    description = "Returns the currently authenticated user's session data. \
                   Requires either a valid Bearer token or an active session cookie.",
    security(
        ("bearer_auth" = []),
    ),
    security(
        ("session_auth" = []),
    ),
    responses(
        (
            status = OK,
            description = "Session data returned successfully",
            body = Session,
            content_type = "application/json"
        ),
        (
            status = UNAUTHORIZED,
            description = "Missing or expired credentials",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = FORBIDDEN,
            description = "Account suspended or unverified",
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
)]
#[tracing::instrument(name = "Get user session", skip(session))]
#[actix_web::get("/session")]
pub async fn get_session(session: Session) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(session))
}

#[utoipa::path(
        post,
        path = "/auth/refresh-token",
        tag = "Auth",
        operation_id = "refreshToken",
        summary = "Refresh access token",
        description = "Issues a new access token using a valid refresh token. \
                   Refresh tokens are rotated on each call — the old token is invalidated immediately.",
        security(
            ("bearer_auth" = []),
        ),
        security(
            ("session_auth" = []),
        ),
        request_body(
            description = "Valid refresh token",
            content = RefreshTokenRequest,
            content_type = "application/json"
        ),
        responses(
            (
                status = OK,
                description = "New access token issued successfully",
                body = AuthResponse,
                content_type = "application/json"
            ),
            (
                status = UNAUTHORIZED,
                description = "Refresh token invalid, expired, or already rotated",
                body = ErrorResponse,
                content_type = "application/json"
            ),
            (
                status = BAD_REQUEST,
                description = "Malformed request or missing refresh token",
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
    )]
#[tracing::instrument(name = "Refresh user accessToken", skip(req, app_state))]
#[actix_web::post("/refresh-token")]
pub async fn refresh_token(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse> {
    let jwt = app_state.configuration.auth.jwt()?;

    let user_id = app_state
        .consume_refresh_token(req.0.refresh_token.expose_secret())
        .await?;

    // FIXME maybe don't return User here, its not needed
    let user: User = app_state.find_user(&user_id).await?;

    app_state.issue_jwt(&user_id).await.map(|tokens| {
        let expiry = jwt.access_token_expires_in;
        Ok(HttpResponse::Ok().json(AuthResponse::new(user).with_jwt(tokens, expiry)))
    })?
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    summary = "Logout",
    description = "Invalidates the current session and refresh token. \
                   The client should discard all stored tokens and cookies after this call.",
    security(
        ("bearer_auth" = []),
    ),
    security(
        ("session_auth" = []),
    ),
    request_body(
        description = "Refresh token to invalidate",
        content = RefreshTokenRequest,
        content_type = "application/json"
    ),
    responses(
        (
            status = OK,
            description = "Logged out successfully — session and refresh token invalidated",
        ),
        (
            status = UNAUTHORIZED,
            description = "Missing or expired credentials",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = BAD_REQUEST,
            description = "Malformed request or missing refresh token",
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
)]
#[tracing::instrument(name = "Logout user", skip(req, app_state, session, session_manager))]
#[actix_web::post("/logout")]
async fn logout(
    AppStateExtractor(app_state): AppStateExtractor,
    req: web::Json<RefreshTokenRequest>,
    session: Session,
    session_manager: actix_session::Session,
) -> Result<HttpResponse> {
    let result = match app_state.configuration.auth.strategy {
        AuthStrategy::Session(..) => app_state.invalidate_session(&session.token).await,
        AuthStrategy::Jwt(..) => {
            app_state
                .invalidate_jwt(req.0.refresh_token.expose_secret())
                .await
        }
    };

    session_manager.purge();
    result.map(|_| Ok(HttpResponse::Ok().finish()))?
}

#[utoipa::path(
        post,
        path = "/auth/password/forgot",
        tag = "Auth",
        operation_id = "forgotPassword",
        summary = "Request a password reset",

        description = "Sends a password reset link to the provided email address if an account exists.\n\n\
                       **Note**: Always returns `200` regardless of whether the email exists \
                       to prevent user enumeration attacks.",
        security(()),
        request_body(
            description = "Email address to send the reset link to",
            content = EmailRequest,
            content_type = "application/json"
        ),
        responses(
            (
                status = OK,
                description = "Reset email sent if an account with that address exists",
                body = ExpiringLink,
                content_type = "application/json"
            ),
            (
                status = BAD_REQUEST,
                description = "Malformed request or invalid email format",
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
    )]
#[tracing::instrument(name = "Forgot password", skip(app_state))]
#[actix_web::post("/password/forgot")]
async fn request_password_reset(
    req: web::Json<EmailRequest>,
    AppStateExtractor(app_state): AppStateExtractor,
) -> Result<HttpResponse> {
    req.validate()?;

    let start = std::time::Instant::now();

    let email: String = req.into_inner().email;
    // 2.
    let mut bucket = RATE_LIMITS.entry(email.clone()).or_default();
    bucket.run()?;

    let result = async {
        let user = app_state.find_user_by_email(&email).await?;
        let user_id = user.id()?;

        // 3.
        app_state.revoke_password_reset_token(user_id).await?;

        // 4.
        let expiring_link = app_state.insert_password_reset_token(user_id).await?;

        // 5.
        let mut bucket = RATE_LIMITS.entry(user_id.to_string()).or_default();
        bucket.run()?;

        Ok::<ExpiringLink, Error>(expiring_link)
    }
    .await;

    throttle::throttle_since(start).await;

    // NOTE Use HMAC or JWT signing so you can later verify it server-side.
    // NOTE maybe use for email_verification
    match result {
        Ok(reset_link) => Ok(HttpResponse::Ok().json(reset_link)),
        Err(err) => {
            tracing::error!("Password reset failed for email {}: {}", email, err);
            Err(err)
        }
    }
}

#[utoipa::path(
    get,
    path = "/auth/password/reset",
    tag = "Auth",
    operation_id = "renderPasswordResetForm",
    summary = "Render password reset form",
    description = "Validates the password reset token from the email link and renders an HTML form \
                   to set a new password. The token is single-use and expires after a short window.",
    security(()),
    params(
        ("token" = String, Query, description = "Single-use password reset token from the email link"),
    ),
    responses(
        (
            status = OK,
            description = "Token valid — password reset form returned",
            content_type = "text/html",
            body = String
        ),
        (
            status = BAD_REQUEST,
            description = "Malformed request or missing token",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = UNAUTHORIZED,
            description = "Reset token invalid or expired",
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
)]
#[tracing::instrument(name = "Render Reset Form", skip(app_state, session))]
#[actix_web::get("/password/reset")]
async fn render_reset_form(
    AppStateExtractor(app_state): AppStateExtractor,
    ValidatedQuery(query): ValidatedQuery<TokenQuery>,
    session: actix_session::Session,
) -> Result<HttpResponse> {
    let token: &str = query.token.expose_secret();
    app_state.validate_reset_password_token(token).await?;

    let csrf_token = app_state.crypto.token.generate()?;
    session.clear();
    session.insert(cookies::CSRF_COOKIE, &csrf_token)?;

    let script_nonce = app_state.crypto.token.generate()?;
    let style_nonce = app_state.crypto.token.generate()?;

    let body = include_str!("templates/update_password.html")
        .replace("{{STYLE_NONCE}}", &style_nonce)
        .replace("{{SCRIPT_NONCE}}", &script_nonce)
        .replace("{{TOKEN}}", token)
        .replace("{{CSRF_TOKEN}}", &csrf_token);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header((
            "Content-Security-Policy",
            format!(
                "default-src 'self'; style-src 'nonce-{}'; script-src 'nonce-{}'",
                &style_nonce, &script_nonce
            ),
        ))
        .body(body))
}

#[utoipa::path(
    post,
    path = "/auth/password/reset",
    tag = "Auth",
    operation_id = "submitNewPassword",
    summary = "Submit new password",
    description = "Submits a new password using a valid reset token. \
    The token is invalidated immediately after use. \
                       Accepts a form submission (not JSON) since it is rendered from an HTML form.",
    security(()),
    request_body(
        description = "Reset token, CSRF token, and new password",
        content = ResetPasswordRequest,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses(
        (
            status = FOUND,
            description = "Password reset successful — redirects to success page",
            headers(
                ("Location" = String, description = "URL of the post-reset success page")
            )
        ),
        (
            status = BAD_REQUEST,
            description = "Malformed request or validation error",
            body = ErrorResponse,
            content_type = "application/json"
        ),
        (
            status = UNAUTHORIZED,
            description = "Reset token or CSRF token invalid or expired",
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
)]
#[tracing::instrument(name = "Update Password", skip(app_state, session, form))]
#[actix_web::post("/password/reset")]
async fn submit_new_password(
    AppStateExtractor(app_state): AppStateExtractor,
    session: actix_session::Session,
    form: web::Form<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    // 0. validat password strength
    form.validate_with_args(&app_state.configuration.auth.password.requirements)?;

    // 0.5 Verify CSRF token
    if let Some(expected) = session.get::<String>(cookies::CSRF_COOKIE)? {
        if !app_state
            .crypto
            .token
            .verify(&expected, form.csrf_token.expose_secret())
        {
            return Ok(HttpResponse::Forbidden().body("Invalid CSRF token"));
        }
    } else {
        return Ok(HttpResponse::Forbidden().body("Missing CSRF token"));
    }
    session.remove(cookies::CSRF_COOKIE);

    let token: &str = form.token.expose_secret();

    // 1. ReValidate token
    let reset_token = app_state.validate_reset_password_token(token).await?;
    let user_id = &reset_token.user_id;
    let reset_token_id = reset_token.id()?;

    // NOTE → Rate limit per token

    // 2. Ensure password is different then previous
    let account = app_state.find_account(user_id).await?;
    if account.locked {
        return Err(Error::Forbidden(ForbiddenReason::AccountSuspended));
    }
    if app_state
        .crypto
        .password_hasher
        .verify(&form.password, &account.password)?
    {
        tracing::warn!("Password reuse attempt for user: {}", user_id);
        return Err(Error::Unauthenticated(AuthError::InvalidCredentials {
            field: CredentialField::Password,
        }));
    }

    // 3.
    let hashed_password = app_state.crypto.password_hasher.hash(&form.password)?;
    app_state
        .update_user_password(user_id, &hashed_password)
        .await?;

    // 4.
    app_state
        .invalidate_password_reset_token(reset_token_id)
        .await?;

    // 6. TODO it may not be neccessary
    app_state.logout_all(user_id).await?;

    let success_redirect = match app_state.configuration.auth.password.reset.success_redirect {
        Some(url) => url,
        None => app_state.configuration.app.url,
    };
    Ok(HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, success_redirect))
        .finish())
}

pub fn auth_scope() -> Scope {
    let protected = web::scope("")
        .wrap(from_fn(authorization_middleware))
        .wrap(from_fn(auth_middleware))
        .service(get_session)
        .service(logout);

    web::scope("/auth")
        .service(login)
        .service(register)
        .service(refresh_token)
        .service(request_password_reset)
        .service(render_reset_form)
        .service(submit_new_password)
        .service(protected)
}
