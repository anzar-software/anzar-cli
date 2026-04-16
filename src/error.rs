use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{HttpResponse, http::StatusCode};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Reason {
    InvalidSignature,
    InvalidIssuer,
    InvalidAudience,
    InvalidAlgorithm,
    NotFound,
    AlreadyExist,
    Expired,
    Malformed,
    HashMismatch,
    UnauthorizedSource,
    Empty,
    Any,
    Unknown,
}
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

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        std::io::Error::other(err.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // -- Tokens
    #[error("Invalid token: {token_type:?} ({reason:?})")]
    InvalidToken {
        token_type: TokenErrorType,
        reason: Reason,
    },
    #[error("Token not found: {token_id}")]
    TokenNotFound { token_id: String },
    #[error("Token of type {token_type:?} expired at {expired_at}")]
    TokenExpired {
        token_type: TokenErrorType,
        expired_at: DateTime<Utc>,
    },
    #[error("Token has already been used: {token_id}")]
    TokenAlreadyUsed { token_id: String },
    #[error("Failed to create token of type: {token_type:?}")]
    TokenCreationFailed { token_type: TokenErrorType },
    #[error("Failed to revoke token: {token_id}")]
    TokenRevocationFailed { token_id: String },

    // -- Accounts
    #[error("Account not verified for field: {field:?}")]
    AccountNotVerified { field: CredentialField },
    #[error("Invalid credentials for {field:?}: {reason:?}")]
    InvalidCredentials {
        field: CredentialField,
        reason: Reason,
    },
    #[error("Missing required credentials for field: {field:?}")]
    MissingCredentials { field: CredentialField },
    #[error("Account has been suspended")]
    AccountSuspended {},
    #[error("User not found (ID: {user_id:?}, Email: {email:?})")]
    UserNotFound {
        user_id: Option<String>,
        email: Option<String>,
    },

    // -- Rate Limiting
    #[error("Rate limit exceeded: {limit} requests allowed per {window:?}")]
    RateLimitExceeded { limit: u32, window: Duration },

    // -- Communication
    #[error("Failed to send email to: {to}")]
    EmailSendFailed { to: String },
    #[error("TLS configuration error at {path}: {reason:?}")]
    TlsConfig { path: String, reason: Reason },

    #[error("Internal hashing failure")]
    HashingFailure,
    #[error("Malformed data in field: {field:?}")]
    MalformedData { field: CredentialField },
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    // -- Externals
    #[error("Actix web error: {0}")]
    Actix(#[from] actix_web::Error),
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("YAML serialization error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Session insertion error: {0}")]
    SessionInsert(#[from] SessionInsertError),
    #[error("Session retrieval error: {0}")]
    SessionGet(#[from] SessionGetError),
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationError),
    #[error("Validation errors: {0}")]
    Validations(#[from] validator::ValidationErrors),
    #[error("Memcache error: {0}")]
    MemCache(#[from] memcache::MemcacheError),
    #[error("MongoDB error: {0}")]
    MongoDB(#[from] mongodb::error::Error),
    #[error("MongoBoson error: {0}")]
    MongoBoson(#[from] mongodb::bson::ser::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Serialize, utoipa::ToSchema, Debug)]
pub enum ErrorCode {
    InvalidToken,
    TokenNotFound,
    TokenExpired,
    TokenAlreadyUsed,
    TokenCreationFailed,
    TokenRevocationFailed,
    AccountNotVerified,
    InvalidCredentials,
    MissingCredentials,
    AccountSuspended,
    UserNotFound,
    RateLimitExceeded,
    EmailSendFailed,
    HashingFailure,
    MalformedData,
    DatabaseError,
    InvalidRequest,
    UnsupportedMediaType,
    BadRequest,
    InternalServerError,
}

impl Error {
    pub fn to_code(&self) -> ErrorCode {
        match self {
            Error::InvalidToken {
                token_type: _,
                reason: _,
            } => ErrorCode::InvalidToken,
            Error::TokenNotFound { token_id: _ } => ErrorCode::TokenNotFound,
            Error::TokenExpired {
                token_type: _,
                expired_at: _,
            } => ErrorCode::TokenExpired,
            Error::TokenAlreadyUsed { token_id: _ } => ErrorCode::TokenAlreadyUsed,
            Error::TokenCreationFailed { token_type: _ } => ErrorCode::TokenCreationFailed,
            Error::TokenRevocationFailed { token_id: _ } => ErrorCode::TokenRevocationFailed,
            Error::AccountNotVerified { field: _ } => ErrorCode::AccountNotVerified,
            Error::InvalidCredentials {
                field: _,
                reason: _,
            } => ErrorCode::InvalidCredentials,
            Error::MissingCredentials { field: _ } => ErrorCode::MissingCredentials,
            Error::AccountSuspended {} => ErrorCode::AccountSuspended,
            Error::UserNotFound {
                user_id: _,
                email: _,
            } => ErrorCode::UserNotFound,
            Error::RateLimitExceeded {
                limit: _,
                window: _,
            } => ErrorCode::RateLimitExceeded,
            Error::EmailSendFailed { to: _ } => ErrorCode::EmailSendFailed,
            Error::HashingFailure => ErrorCode::HashingFailure,
            Error::MalformedData { field: _ } => ErrorCode::MalformedData,
            Error::DatabaseError(_) => ErrorCode::DatabaseError,
            Error::InvalidRequest => ErrorCode::InvalidRequest,
            Error::UnsupportedMediaType(_) => ErrorCode::UnsupportedMediaType,
            Error::BadRequest(_) => ErrorCode::BadRequest,
            _ => ErrorCode::InternalServerError,
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
            Error::InvalidToken {
                token_type: _,
                reason: _,
            }
            | Error::AccountNotVerified { field: _ }
            | Error::InvalidCredentials {
                field: _,
                reason: _,
            }
            | Error::MissingCredentials { field: _ } => StatusCode::UNAUTHORIZED,

            Error::AccountSuspended {} => StatusCode::FORBIDDEN,

            Error::RateLimitExceeded {
                limit: _,
                window: _,
            } => StatusCode::TOO_MANY_REQUESTS,

            Error::UserNotFound {
                user_id: _,
                email: _,
            }
            | Error::TokenNotFound { token_id: _ } => StatusCode::NOT_FOUND,

            Error::BadRequest(_)
            | Error::InvalidRequest
            | Error::TokenExpired {
                token_type: _,
                expired_at: _,
            }
            | Error::MalformedData { field: _ }
            | Error::TokenAlreadyUsed { token_id: _ }
            | Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Validations(_) => StatusCode::BAD_REQUEST,

            Error::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,

            Error::TokenCreationFailed { token_type: _ }
            | Error::HashingFailure
            | Error::DatabaseError(_)
            | Error::EmailSendFailed { to: _ }
            | Error::TokenRevocationFailed { token_id: _ }
            | Error::TlsConfig { path: _, reason: _ }
            | Error::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Actix(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IO(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SerdeYaml(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SerdeJson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::JWT(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SessionInsert(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SessionGet(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MemCache(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MongoDB(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::MongoBoson(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
