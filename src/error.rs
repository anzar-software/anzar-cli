use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{HttpResponse, http::StatusCode};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // -- Auth (401)
    #[error(transparent)]
    Unauthenticated(#[from] AuthError),

    // -- Authorization (403)
    #[error(transparent)]
    Forbidden(#[from] ForbiddenReason),

    // -- Not Found (404)
    #[error(transparent)]
    NotFound(#[from] ResourceKind),

    // -- Conflict (409)
    #[error(transparent)]
    Conflict(#[from] ConflictReason),

    // -- Validation (400)
    #[error(transparent)]
    Validation(#[from] ValidationError),

    // -- Rate Limiting (429)
    #[error("Rate limit exceeded: {limit} requests allowed per {window:?}")]
    RateLimitExceeded { limit: u32, window: Duration },

    // -- Internal (500)
    #[error(transparent)]
    Internal(#[from] InternalError),
}

// Sub Errors
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("{token_type:?} expired at {expired_at}")]
    TokenExpired {
        token_type: TokenErrorType,
        expired_at: DateTime<Utc>,
    },

    #[error("{token_type:?} token has an invalid signature")]
    TokenInvalidSignature { token_type: TokenErrorType },

    #[error("{token_type:?} token has an invalid audience")]
    TokenInvalidAudience { token_type: TokenErrorType },

    #[error("{token_type:?} token has an invalid issuer")]
    TokenInvalidIssuer { token_type: TokenErrorType },

    #[error("{token_type:?} token uses an invalid or missing algorithm")]
    TokenInvalidAlgorithm { token_type: TokenErrorType },

    #[error("{token_type:?} token is malformed")]
    TokenMalformed { token_type: TokenErrorType },

    #[error("{token_type:?} has already been used")]
    TokenReplay { token_type: TokenErrorType },

    #[error("{token_type:?} is invalid")]
    TokenInvalid { token_type: TokenErrorType }, // bad sig, malformed, etc

    #[error("invalid credentials for {field:?}")]
    InvalidCredentials { field: CredentialField },

    #[error("account is not verified")]
    AccountNotVerified,
}

#[derive(thiserror::Error, Debug)]
pub enum ForbiddenReason {
    #[error("insufficient permissions to perform this action")]
    InsufficientPermissions,

    #[error("account has been suspended")]
    AccountSuspended,
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceKind {
    #[error("user not found (id: {id:?}, email: {email:?})")]
    User {
        id: Option<String>,
        email: Option<String>,
    },

    #[error("{token_type:?} not found")]
    Token { token_type: TokenErrorType },
}

#[derive(thiserror::Error, Debug)]
pub enum ConflictReason {
    #[error("{field:?} already exists")]
    AlreadyExists { field: CredentialField },
}

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("validation fields failed: {0}")]
    Fields(String),

    #[error("malformed {field:?}")]
    Malformed { field: CredentialField },

    #[error("missing required field: {field:?}")]
    Missing { field: CredentialField },

    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("bad request: {0}")]
    BadRequest(String),
}

// -- Fields
impl From<validator::ValidationErrors> for ValidationError {
    fn from(e: validator::ValidationErrors) -> Self {
        ValidationError::Fields(e.to_string())
    }
}
impl From<validator::ValidationErrors> for Error {
    fn from(e: validator::ValidationErrors) -> Self {
        Error::Validation(e.into())
    }
}
impl From<validator::ValidationError> for ValidationError {
    fn from(e: validator::ValidationError) -> Self {
        ValidationError::Fields(e.to_string())
    }
}
impl From<validator::ValidationError> for Error {
    fn from(e: validator::ValidationError) -> Self {
        Error::Validation(e.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("{0}")]
    Database(String),

    #[error("hashing failure")]
    Hashing,

    #[error("failed to create {token_type:?}")]
    TokenCreation { token_type: TokenErrorType },

    #[error("failed to deliver email to {to}")]
    EmailDelivery { to: String },

    #[error("TLS configuration error at {path}")]
    Tls { path: String },

    #[error("Actix web error: {0}")]
    Actix(#[from] actix_web::Error),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("missing app data: {0}")]
    MissingAppData(String),
}

// Categories Errors
#[derive(Debug)]
pub enum CredentialField {
    Username,
    Email,
    Password,
    EmailOrPassword,
    Token,
    ApiKey,
    ObjectId,
}
#[derive(Debug)]
pub enum TokenErrorType {
    Token,
    AccessToken,
    RefreshToken,
    SessionToken,
    PasswordResetToken,
    EmailVerificationToken,
}

// Froms
impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        std::io::Error::other(err.to_string())
    }
}
// -- I/O
impl From<std::io::Error> for InternalError {
    fn from(e: std::io::Error) -> Self {
        InternalError::Io(e.to_string())
    }
}

// -- Database
impl From<sqlx::Error> for InternalError {
    fn from(e: sqlx::Error) -> Self {
        InternalError::Database(e.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for InternalError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        InternalError::Database(e.to_string())
    }
}

impl From<mongodb::error::Error> for InternalError {
    fn from(e: mongodb::error::Error) -> Self {
        InternalError::Database(e.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for InternalError {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        InternalError::Database(e.to_string())
    }
}

// -- Cache
impl From<redis::RedisError> for InternalError {
    fn from(e: redis::RedisError) -> Self {
        InternalError::Cache(e.to_string())
    }
}

impl From<memcache::MemcacheError> for InternalError {
    fn from(e: memcache::MemcacheError) -> Self {
        InternalError::Cache(e.to_string())
    }
}

// -- Serialization
impl From<serde_json::Error> for InternalError {
    fn from(e: serde_json::Error) -> Self {
        InternalError::Serialization(e.to_string())
    }
}

impl From<serde_yaml::Error> for InternalError {
    fn from(e: serde_yaml::Error) -> Self {
        InternalError::Serialization(e.to_string())
    }
}

// -- Session
impl From<SessionInsertError> for InternalError {
    fn from(e: SessionInsertError) -> Self {
        InternalError::Session(e.to_string())
    }
}

impl From<SessionGetError> for InternalError {
    fn from(e: SessionGetError) -> Self {
        InternalError::Session(e.to_string())
    }
}

// -- Convenience: external errors -> Error directly via InternalError
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        Error::Internal(e.into())
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<mongodb::bson::ser::Error> for Error {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Error::Internal(e.into())
    }
}

impl From<memcache::MemcacheError> for Error {
    fn from(e: memcache::MemcacheError) -> Self {
        Error::Internal(e.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::Internal(e.into())
    }
}

impl From<SessionInsertError> for Error {
    fn from(e: SessionInsertError) -> Self {
        Error::Internal(e.into())
    }
}

impl From<SessionGetError> for Error {
    fn from(e: SessionGetError) -> Self {
        Error::Internal(e.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Internal(e.into())
    }
}
// -- JWT and Actix stay at top-level Error since they're not purely internal
impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        Error::Unauthenticated(AuthError::TokenInvalid {
            token_type: TokenErrorType::AccessToken,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema, Debug)]
pub enum ErrorCode {
    // Auth
    TokenExpired,
    TokenReplay,
    TokenInvalid,
    InvalidCredentials,
    AccountNotVerified,
    TokenInvalidSignature,
    TokenInvalidAudience,
    TokenInvalidIssuer,
    TokenInvalidAlgorithm,
    TokenMalformed,

    // Forbidden
    InsufficientPermissions,
    AccountSuspended,
    // Not Found
    UserNotFound,
    TokenNotFound,
    // Conflict
    AlreadyExists,
    // Validation
    ValidationFailed,
    MalformedField,
    MissingField,
    UnsupportedMediaType,
    BadRequest,
    // Rate Limiting
    RateLimitExceeded,
    // Internal
    Internal,
}

impl Error {
    pub fn to_code(&self) -> ErrorCode {
        match self {
            Error::Unauthenticated(auth) => match auth {
                AuthError::TokenExpired { .. } => ErrorCode::TokenExpired,
                AuthError::TokenReplay { .. } => ErrorCode::TokenReplay,
                AuthError::TokenInvalid { .. } => ErrorCode::TokenInvalid,
                AuthError::InvalidCredentials { .. } => ErrorCode::InvalidCredentials,
                AuthError::AccountNotVerified => ErrorCode::AccountNotVerified,
                AuthError::TokenInvalidSignature { .. } => ErrorCode::TokenInvalidSignature,
                AuthError::TokenInvalidAudience { .. } => ErrorCode::TokenInvalidAudience,
                AuthError::TokenInvalidIssuer { .. } => ErrorCode::TokenInvalidIssuer,
                AuthError::TokenInvalidAlgorithm { .. } => ErrorCode::TokenInvalidAlgorithm,
                AuthError::TokenMalformed { .. } => ErrorCode::TokenMalformed,
            },
            Error::Forbidden(reason) => match reason {
                ForbiddenReason::InsufficientPermissions => ErrorCode::InsufficientPermissions,
                ForbiddenReason::AccountSuspended => ErrorCode::AccountSuspended,
            },
            Error::NotFound(resource) => match resource {
                ResourceKind::User { .. } => ErrorCode::UserNotFound,
                ResourceKind::Token { .. } => ErrorCode::TokenNotFound,
            },
            Error::Conflict(reason) => match reason {
                ConflictReason::AlreadyExists { .. } => ErrorCode::AlreadyExists,
            },
            Error::Validation(v) => match v {
                ValidationError::Fields(_) => ErrorCode::ValidationFailed,
                ValidationError::Malformed { .. } => ErrorCode::MalformedField,
                ValidationError::Missing { .. } => ErrorCode::MissingField,
                ValidationError::UnsupportedMediaType(_) => ErrorCode::UnsupportedMediaType,
                ValidationError::BadRequest(_) => ErrorCode::BadRequest,
            },
            Error::RateLimitExceeded { .. } => ErrorCode::RateLimitExceeded,
            Error::Internal(_) => ErrorCode::Internal,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({"message": "An Error occured"}))]
pub struct ErrorResponse {
    code: ErrorCode,
    message: String,
}
impl actix_web::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse {
        let error_response = ErrorResponse {
            code: self.to_code(),
            message: self.to_string(),
        };

        HttpResponse::build(self.status_code()).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
