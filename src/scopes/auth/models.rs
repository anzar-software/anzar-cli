use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::scopes::user::User;
use crate::services::jwt::IssuedTokens;
use crate::utils::validation::validate_password;

use crate::config::PasswordRequirements;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"email": "example@email.com", "password": "password"}))]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"username": "name", "email": "example@email.com", "password": "password"}))]
pub struct RegisterRequest {
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
#[schema(example = json!({"token": "edc365fa5e13751XXXXXXX"}))]
pub struct TokenQuery {
    // #[validate(custom(function = "validate_token"))]
    #[schema(value_type = String, format = Password)]
    pub token: secrecy::SecretString,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
#[schema(example = json!({"email": "example@email.com"}))]
pub struct EmailRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
#[schema(example = json!({"refresh_token": "edc365fa5e13751XXXXXXX"}))]
pub struct RefreshTokenRequest {
    #[schema(value_type = String, format = Password)]
    pub refresh_token: secrecy::SecretString,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"link": String::default(), "expires_at": "2026-02-19T22:42:23.467Z"}))]
pub struct ExpiringLink {
    pub link: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[validate(context = "PasswordRequirements")]
#[schema(example = json!({"token": String::default(), "csrf_token": String::default(), "password": String::default()}))]
pub struct ResetPasswordRequest {
    #[schema(value_type = String, format = Password)]
    pub token: secrecy::SecretString,

    #[schema(value_type = String, format = Password)]
    pub csrf_token: secrecy::SecretString,

    #[validate(custom(function = "validate_password", use_context))]
    pub password: String,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema)]
#[schema(description = "SessionTokens model", example = json!({"access": String::default(), "expires_in": 3600, "token_type": "Bearer", "refresh": String::default()}))]
pub struct SessionTokens {
    pub access: String,
    pub expires_in: i64,
    pub token_type: String,
    pub refresh: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"user": User::default(), "tokens": Some(SessionTokens::default()), "verification": Some(ExpiringLink::default())}))]
pub struct AuthResponse {
    pub user: User,
    pub tokens: Option<SessionTokens>,
    pub verification: Option<ExpiringLink>,
}

impl AuthResponse {
    pub fn new(user: User) -> Self {
        Self {
            user,
            tokens: None,
            verification: None,
        }
    }
}

impl AuthResponse {
    pub fn with_jwt(mut self, tokens: IssuedTokens, expires_in: i64) -> Self {
        let _ = self.tokens.insert(SessionTokens {
            access: tokens.access_token,
            expires_in,
            token_type: "Bearer".to_string(),
            refresh: tokens.refresh_token,
        });
        self
    }
    pub fn with_verification(mut self, link: &str, expires_in: i64) -> Self {
        let expiry_timestamp = chrono::Utc::now() + chrono::Duration::seconds(expires_in);

        let _ = self.verification.insert(ExpiringLink {
            link: link.into(),
            expires_at: expiry_timestamp,
        });
        self
    }
}
