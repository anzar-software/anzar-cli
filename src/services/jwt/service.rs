use crate::config::Configuration;
use crate::error::{CredentialField, Error, Result};
use crate::extractors::{Claims, TokenType};
use crate::scopes::auth::service::AuthService;
use crate::scopes::user::User;
use crate::services::jwt::JwtDecoder;

use super::{IssuedTokens, JwtEncoder, RefreshToken};

pub trait JwtServiceTrait {
    fn consume_refresh_token(
        &self,
        refresh_token: &str,
        configuration: &Configuration,
    ) -> impl Future<Output = Result<String>>;
    fn issue_jwt(
        &self,
        user: &User,
        configuration: &Configuration,
    ) -> impl Future<Output = Result<IssuedTokens>>;
    fn invalidate_jwt(
        &self,
        refresh_token: &str,
        configuration: &Configuration,
    ) -> impl Future<Output = Result<()>>;
    // fn logout(&self, payload: AuthPayload) -> impl Future<Output = Result<()>>;
    fn logout_all(&self, user_id: &str) -> impl Future<Output = Result<()>>;
    fn find_jwt_by_jti(&self, jti: &str) -> impl Future<Output = Result<RefreshToken>>;
}
impl JwtServiceTrait for AuthService {
    async fn consume_refresh_token(
        &self,
        refresh_token: &str,
        configuration: &Configuration,
    ) -> Result<String> {
        let claims: Claims = JwtDecoder::new(refresh_token, configuration).decode()?;

        if self
            .jwt_repository
            .find_and_consume(&claims, refresh_token)
            .await
            .is_err()
        {
            // TODO: send an email indicating a breach
            self.jwt_repository.revoke(&claims.sub).await?;
            return Err(Error::InvalidToken {
                token_type: crate::error::TokenErrorType::RefreshToken,
                reason: crate::error::Reason::Expired,
            });
        }

        Ok(claims.sub)
    }
    async fn issue_jwt(&self, user: &User, configuration: &Configuration) -> Result<IssuedTokens> {
        let user_id = user.id.as_ref().ok_or(Error::MalformedData {
            field: CredentialField::ObjectId,
        })?;

        let tokens: IssuedTokens = JwtEncoder::new(user, configuration).encode()?;

        let refresh_token = RefreshToken::new(&tokens)
            .with_user_id(user_id)
            .with_expire_at(configuration.auth.jwt.refresh_token_expires_in);

        self.jwt_repository.insert(refresh_token).await?;
        Ok(tokens)
    }

    async fn invalidate_jwt(
        &self,
        refresh_token: &str,
        configuration: &Configuration,
    ) -> Result<()> {
        let claims: Claims = JwtDecoder::new(refresh_token, configuration).decode()?;

        if claims.token_type != TokenType::RefreshToken {
            return Err(Error::InvalidToken {
                token_type: crate::error::TokenErrorType::RefreshToken,
                reason: crate::error::Reason::Malformed,
            });
        }

        self.jwt_repository.invalidate(claims.jti).await?;
        Ok(())
    }

    // async fn logout(&self, payload: AuthPayload) -> Result<()> {
    //     self.jwt_service.invalidate(&payload.jti).await?;
    //     self.session_service.revoke(&payload.user_id).await?;
    //     Ok(())
    // }
    async fn logout_all(&self, user_id: &str) -> Result<()> {
        self.jwt_repository.revoke(user_id).await?;
        self.session_repository.revoke(user_id).await?;
        Ok(())
    }

    async fn find_jwt_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        self.jwt_repository.find_by_jti(jti).await
    }
}
