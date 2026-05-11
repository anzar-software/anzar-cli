use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::validation::validate_objectid;
use crate::config::JwtConfig;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    #[default]
    AccessToken,
    RefreshToken,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Validate)]
pub struct Claims {
    #[validate(length(equal = 24), custom(function = "validate_objectid"))]
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
    pub jti: uuid::Uuid,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub token_type: TokenType,
}

impl Claims {
    pub fn new(user_id: &str, role: &str) -> Self {
        Claims {
            sub: user_id.into(),
            roles: vec![role.into()],
            iat: Local::now().timestamp() as usize,
            jti: uuid::Uuid::new_v4(),
            ..Default::default()
        }
    }
}

impl Claims {
    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.iss = issuer.into();
        self
    }
    pub fn with_audience(mut self, audience: &str) -> Self {
        self.aud = audience.into();
        self
    }
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
    fn with_expiry(mut self, expires_in: i64) -> Self {
        self.exp = (Local::now() + Duration::seconds(expires_in)).timestamp() as usize;
        self
    }
    fn with_token_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type;
        self
    }
}
impl Claims {
    pub fn into_token_pair(self, jwt_config: &JwtConfig) -> (Claims, Claims) {
        let access = self
            .clone()
            .with_expiry(jwt_config.access_token_expires_in)
            .with_token_type(TokenType::AccessToken);

        let refresh = self
            .with_expiry(jwt_config.refresh_token_expires_in)
            .with_token_type(TokenType::RefreshToken);
        (access, refresh)
    }
}
