use std::future::{Ready, ready};

use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::config::JWT;
use crate::error::Error;
use crate::scopes::user::Role;
use crate::utils::validation::validate_objectid;

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
    pub jti: uuid::Uuid,
    pub role: Role,
    pub token_type: TokenType,
}

impl Claims {
    pub fn new(user_id: &str, role: Role) -> Self {
        Claims {
            sub: user_id.into(),
            role,
            iat: Local::now().timestamp() as usize,
            jti: uuid::Uuid::new_v4(),
            ..Default::default()
        }
    }
}

impl Claims {
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
    pub fn into_token_pair(self, jwt_config: &JWT) -> (Claims, Claims) {
        let access = self
            .clone()
            .with_expiry(jwt_config.expires_in)
            .with_token_type(TokenType::AccessToken);
        let refresh = self
            .with_expiry(jwt_config.refresh_expires_in)
            .with_token_type(TokenType::RefreshToken);
        (access, refresh)
    }
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(claims.clone())),
            None => ready(Ok(Claims::default())),
        }
    }
}
