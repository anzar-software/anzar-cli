use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::application::model::{ExpiringLink, IssuedTokens, User};

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

// ------------ AUTH RESPONESE ---------------------
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
