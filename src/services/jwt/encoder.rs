use jsonwebtoken::{Header, encode};

use super::model::Tokens;
use crate::config::Configuration;
use crate::error::{CredentialField, Error, Result};
use crate::extractors::Claims;
use crate::scopes::user::User;

pub struct JwtEncoder<'a> {
    user: &'a User,
    config: &'a Configuration,
}
impl<'a> JwtEncoder<'a> {
    pub fn new(user: &'a User, config: &'a Configuration) -> Self {
        Self { user, config }
    }

    pub fn encode(&self) -> Result<Tokens> {
        let encoding_secret =
            jsonwebtoken::EncodingKey::from_secret(self.config.security.secret_key.as_bytes());
        let jwt_config = &self.config.auth.jwt;

        let user_id = self.user.id.as_ref().ok_or(Error::MalformedData {
            field: CredentialField::ObjectId,
        })?;

        let (access_claims, refresh_claims) =
            Claims::new(user_id, self.user.role.clone()).into_token_pair(jwt_config);

        let access_token = encode(&Header::default(), &access_claims, &encoding_secret)?;
        let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_secret)?;

        Ok(Tokens::default()
            .with_access_token(&access_token)
            .with_refresh_token(&refresh_token)
            .with_jti(refresh_claims.jti))
    }
}
