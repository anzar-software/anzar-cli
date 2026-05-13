use super::user::UserDto;
use serde::{Deserialize, Serialize};

// ------------ AUTH RESPONESE ---------------------
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SessionTokens {
    pub access: String,
    pub expires_in: i64,
    pub token_type: String,
    pub refresh: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ExpiringLink {
    pub link: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponseDto {
    pub user: UserDto,
    pub tokens: Option<SessionTokens>,
    pub verification: Option<ExpiringLink>,
}
