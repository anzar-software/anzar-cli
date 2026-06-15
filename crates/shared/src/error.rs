use chrono::{DateTime, Duration, Utc};

pub type Result<T> = core::result::Result<T, CoreError>;

// ── Top-level core Error ─────────────────────────────────────────────────────
// NOTE: no actix_web, no HTTP status codes — that belongs in `api`

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Unauthenticated(#[from] AuthError),
    #[error(transparent)]
    Forbidden(#[from] ForbiddenReason),
    #[error(transparent)]
    NotFound(#[from] ResourceKind),
    #[error(transparent)]
    Conflict(#[from] ConflictReason),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("rate limit exceeded: {limit} requests allowed per {window:?}")]
    RateLimitExceeded { limit: u32, window: Duration },
    #[error(transparent)]
    Internal(#[from] InternalError),
}

// ── Supporting types ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CredentialField {
    Username,
    Email,
    Password,
    EmailOrPassword,
    Token,
    ApiKey,
    ObjectId,
}

#[derive(Debug, Clone)]
pub enum TokenErrorType {
    Token,
    AccessToken,
    RefreshToken,
    SessionToken,
    PasswordResetToken,
    EmailVerificationToken,
}

// ── Sub-errors ───────────────────────────────────────────────────────────────

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
    TokenInvalid { token_type: TokenErrorType },
    #[error("invalid credentials for {field:?}")]
    InvalidCredentials { field: CredentialField },
    #[error("account is not verified")]
    AccountNotVerified,
    #[error("jwt is not configured")]
    JwtNotConfigured,
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
    #[error("role not found (id: {id:?})")]
    Role { id: Option<String> },
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
    #[error("malformed {field:?}")]
    Malformed { field: CredentialField },
    #[error("missing required field: {field:?}")]
    Missing { field: CredentialField },
    #[error("invalid value for '{0}'")]
    Invalid(String),
}

#[derive(thiserror::Error, Debug)]
pub enum InternalError {
    #[error("database error: {0}")]
    Database(String),
    #[error("hashing failure")]
    Hashing,
    #[error("crypto failure - {0}")]
    Crypto(String),
    #[error("failed to create {token_type:?}")]
    TokenCreation { token_type: TokenErrorType },
    #[error("failed to deliver email to {to}")]
    EmailDelivery { to: String },
    #[error("TLS configuration error at {path}")]
    Tls { path: String },
    #[error("I/O error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("cache error: {0}")]
    Cache(String),
    #[error("session error: {0}")]
    Session(String),
    #[error("JWT error: {0}")]
    Jwt(String),
    #[error("missing app data: {0}")]
    MissingConfiguration(String),
    #[error("missing configuration field: {field} - {reason}")]
    MissingField { field: String, reason: String },
    #[error("invalid config")]
    InvalidConfig(Vec<CoreError>), // store messages, not Error — avoids recursive type
}

// ── ValidationError From impls ─────────────────────────────────────────────────
impl From<validator::ValidationErrors> for ValidationError {
    fn from(e: validator::ValidationErrors) -> Self {
        ValidationError::Invalid(e.to_string())
    }
}

impl From<validator::ValidationError> for ValidationError {
    fn from(e: validator::ValidationError) -> Self {
        ValidationError::Invalid(e.to_string())
    }
}

// ── InternalError From impls ─────────────────────────────────────────────────
impl From<std::io::Error> for InternalError {
    fn from(e: std::io::Error) -> Self {
        InternalError::Io(e.to_string())
    }
}
impl From<std::env::VarError> for InternalError {
    fn from(e: std::env::VarError) -> Self {
        InternalError::Io(e.to_string())
    }
}

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
impl From<config::ConfigError> for InternalError {
    fn from(e: config::ConfigError) -> Self {
        InternalError::Serialization(e.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for InternalError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        InternalError::Jwt(e.to_string())
    }
}

impl From<openssl::error::ErrorStack> for InternalError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        InternalError::Crypto(e.to_string())
    }
}
impl From<hex::FromHexError> for InternalError {
    fn from(e: hex::FromHexError) -> Self {
        InternalError::Crypto(e.to_string())
    }
}

// Convenience From impls so callers can use `?` without going through InternalError explicitly

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<std::env::VarError> for CoreError {
    fn from(e: std::env::VarError) -> Self {
        CoreError::Internal(e.into())
    }
}

impl From<sqlx::Error> for CoreError {
    fn from(e: sqlx::Error) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<sqlx::migrate::MigrateError> for CoreError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        CoreError::Internal(e.into())
    }
}

impl From<mongodb::error::Error> for CoreError {
    fn from(e: mongodb::error::Error) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<mongodb::bson::ser::Error> for CoreError {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        CoreError::Internal(e.into())
    }
}

impl From<redis::RedisError> for CoreError {
    fn from(e: redis::RedisError) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<memcache::MemcacheError> for CoreError {
    fn from(e: memcache::MemcacheError) -> Self {
        CoreError::Internal(e.into())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<serde_yaml::Error> for CoreError {
    fn from(e: serde_yaml::Error) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<config::ConfigError> for CoreError {
    fn from(e: config::ConfigError) -> Self {
        CoreError::Internal(e.into())
    }
}

impl From<openssl::error::ErrorStack> for CoreError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        CoreError::Internal(e.into())
    }
}
impl From<hex::FromHexError> for CoreError {
    fn from(e: hex::FromHexError) -> Self {
        CoreError::Internal(e.into())
    }
}

// JWT maps to Unauthenticated, not Internal — keep this behaviour from your original
impl From<jsonwebtoken::errors::Error> for CoreError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        CoreError::Unauthenticated(AuthError::TokenInvalid {
            token_type: TokenErrorType::AccessToken,
        })
    }
}

impl From<validator::ValidationErrors> for CoreError {
    fn from(e: validator::ValidationErrors) -> Self {
        CoreError::Validation(e.into())
    }
}
impl From<validator::ValidationError> for CoreError {
    fn from(e: validator::ValidationError) -> Self {
        CoreError::Validation(e.into())
    }
}

impl From<CoreError> for std::io::Error {
    fn from(err: CoreError) -> Self {
        std::io::Error::other(err.to_string())
    }
}
