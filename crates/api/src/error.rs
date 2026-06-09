use actix_session::{SessionGetError, SessionInsertError};
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use serde::Serialize;
use shared::error::{
    AuthError, ConflictReason, CoreError, ForbiddenReason, ResourceKind, ValidationError,
};
pub use shared::error::{CredentialField, TokenErrorType};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("session error: {0}")]
    Session(String),

    #[error("actix error: {0}")]
    Actix(#[from] actix_web::Error),
}

// Session errors only make sense in api
impl From<SessionInsertError> for Error {
    fn from(e: SessionInsertError) -> Self {
        Error::Session(e.to_string())
    }
}
impl From<SessionGetError> for Error {
    fn from(e: SessionGetError) -> Self {
        Error::Session(e.to_string())
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        std::io::Error::other(e.to_string())
    }
}

// ── Error codes ───────────────────────────────────────────────────────────────

#[derive(Serialize, utoipa::ToSchema, Debug)]
pub enum ErrorCode {
    BadRequest,
    ValidationFailed,
    MalformedField,
    MissingField,
    UnsupportedMediaType,
    TokenExpired,
    TokenReplay,
    TokenInvalid,
    InvalidCredentials,
    AccountNotVerified,
    JwtNotConfigured,
    TokenInvalidSignature,
    TokenInvalidAudience,
    TokenInvalidIssuer,
    TokenInvalidAlgorithm,
    TokenMalformed,
    InsufficientPermissions,
    AccountSuspended,
    RoleNotFound,
    UserNotFound,
    TokenNotFound,
    AlreadyExists,
    RateLimitExceeded,
    Internal,
}

impl Error {
    pub fn to_code(&self) -> ErrorCode {
        match self {
            Error::Core(e) => core_error_to_code(e), // delegate
            Error::Session(_) => ErrorCode::Internal,
            Error::Actix(_) => ErrorCode::Internal,
        }
    }
}
fn core_error_to_code(e: &CoreError) -> ErrorCode {
    // Move your existing to_code() match arms here
    match e {
        CoreError::BadRequest(_) => ErrorCode::BadRequest,
        CoreError::Validation(v) => match v {
            ValidationError::Invalid(_) => ErrorCode::ValidationFailed,
            ValidationError::Malformed { .. } => ErrorCode::MalformedField,
            ValidationError::Missing { .. } => ErrorCode::MissingField,
        },
        CoreError::Unauthenticated(auth) => match auth {
            AuthError::TokenExpired { .. } => ErrorCode::TokenExpired,
            AuthError::TokenReplay { .. } => ErrorCode::TokenReplay,
            AuthError::TokenInvalid { .. } => ErrorCode::TokenInvalid,
            AuthError::InvalidCredentials { .. } => ErrorCode::InvalidCredentials,
            AuthError::AccountNotVerified => ErrorCode::AccountNotVerified,
            AuthError::JwtNotConfigured => ErrorCode::JwtNotConfigured,
            AuthError::TokenInvalidSignature { .. } => ErrorCode::TokenInvalidSignature,
            AuthError::TokenInvalidAudience { .. } => ErrorCode::TokenInvalidAudience,
            AuthError::TokenInvalidIssuer { .. } => ErrorCode::TokenInvalidIssuer,
            AuthError::TokenInvalidAlgorithm { .. } => ErrorCode::TokenInvalidAlgorithm,
            AuthError::TokenMalformed { .. } => ErrorCode::TokenMalformed,
        },
        CoreError::Forbidden(reason) => match reason {
            ForbiddenReason::InsufficientPermissions => ErrorCode::InsufficientPermissions,
            ForbiddenReason::AccountSuspended => ErrorCode::AccountSuspended,
        },
        CoreError::NotFound(resource) => match resource {
            ResourceKind::User { .. } => ErrorCode::UserNotFound,
            ResourceKind::Role { .. } => ErrorCode::RoleNotFound,
            ResourceKind::Token { .. } => ErrorCode::TokenNotFound,
        },
        CoreError::Conflict(reason) => match reason {
            ConflictReason::AlreadyExists { .. } => ErrorCode::AlreadyExists,
        },
        CoreError::UnsupportedMediaType { .. } => ErrorCode::UnsupportedMediaType,
        CoreError::RateLimitExceeded { .. } => ErrorCode::RateLimitExceeded,
        CoreError::Internal(_) => ErrorCode::Internal,
    }
}

// ── HTTP response ─────────────────────────────────────────────────────────────

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"code": "Internal", "message": "An error occurred"}))]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

impl actix_web::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse {
        let error_response = ErrorResponse {
            code: self.to_code(),
            message: self.to_string(),
        };

        let mut builder = HttpResponse::build(self.status_code());

        if let Error::Core(CoreError::RateLimitExceeded { window, .. }) = self {
            builder.insert_header(("Retry-After", window.as_seconds_f32().to_string()));
        }

        builder.json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::Core(e) => core_status(e),
            Error::Session(_) | Error::Actix(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn core_status(e: &CoreError) -> StatusCode {
    match e {
        CoreError::BadRequest(_) | CoreError::Validation(_) => StatusCode::BAD_REQUEST,
        CoreError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
        CoreError::Forbidden(_) => StatusCode::FORBIDDEN,
        CoreError::NotFound(_) => StatusCode::NOT_FOUND,
        CoreError::Conflict(_) => StatusCode::CONFLICT,
        CoreError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        CoreError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        CoreError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
    }
}

// Now valid: ApiError is defined IN this crate
// api/src/error.rs
macro_rules! impl_from_via_core {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Error {
                fn from(e: $t) -> Self {
                    Error::Core(e.into())  // CoreError already has From<$t>
                }
            }
        )*
    };
}

impl_from_via_core!(
    std::io::Error,
    std::env::VarError,
    sqlx::Error,
    sqlx::migrate::MigrateError,
    mongodb::error::Error,
    mongodb::bson::ser::Error,
    serde_json::Error,
    serde_yaml::Error,
    redis::RedisError,
    config::ConfigError,
    validator::ValidationError,
    validator::ValidationErrors
);
