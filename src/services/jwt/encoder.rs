use jsonwebtoken::{Header, encode};

use super::model::IssuedTokens;
use crate::config::AnzarConfiguration;
use crate::error::{CredentialField, Error, Result, ValidationError};
use crate::extractors::Claims;
use crate::scopes::user::User;

pub struct JwtEncoder<'a> {
    user: &'a User,
    config: &'a AnzarConfiguration,
    header: Header,
}
impl<'a> JwtEncoder<'a> {
    pub fn new(user: &'a User, config: &'a AnzarConfiguration) -> Self {
        let header = jsonwebtoken::Header::new(config.auth.jwt.algorithm.clone().into());

        Self {
            user,
            config,
            header,
        }
    }

    pub fn encode(&self) -> Result<IssuedTokens> {
        let encoding_secret =
            jsonwebtoken::EncodingKey::from_secret(self.config.security.secret_key.as_bytes());
        let jwt_config = &self.config.auth.jwt;

        let user_id = self.user.id.as_ref().ok_or_else(|| {
            Error::Validation(ValidationError::Malformed {
                field: CredentialField::ObjectId,
            })
        })?;

        let (access_claims, refresh_claims) = Claims::new(user_id, self.user.role.clone())
            .with_issuer(&self.config.auth.jwt.issuer)
            .with_audience(&self.config.auth.jwt.audience)
            .into_token_pair(jwt_config);

        let access_token = encode(&self.header, &access_claims, &encoding_secret)?;
        let refresh_token = encode(&self.header, &refresh_claims, &encoding_secret)?;

        Ok(IssuedTokens::default()
            .with_access_token(&access_token)
            .with_refresh_token(&refresh_token)
            .with_jti(refresh_claims.jti))
    }
}
