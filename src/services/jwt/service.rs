use crate::config::AnzarConfiguration;
use crate::error::{AuthError, Error, Result, TokenErrorType};
use crate::extractors::{Claims, TokenType};
use crate::scopes::auth::service::AuthService;
use crate::scopes::user::User;
use crate::services::jwt::JwtDecoder;

use super::{IssuedTokens, JwtEncoder, RefreshToken};

pub trait JwtServiceTrait {
    fn consume_refresh_token(
        &self,
        refresh_token: &str,
        configuration: &AnzarConfiguration,
    ) -> impl Future<Output = Result<String>>;
    fn issue_jwt(
        &self,
        user: &User,
        configuration: &AnzarConfiguration,
    ) -> impl Future<Output = Result<IssuedTokens>>;
    fn invalidate_jwt(
        &self,
        refresh_token: &str,
        configuration: &AnzarConfiguration,
    ) -> impl Future<Output = Result<()>>;
    // fn logout(&self, payload: AuthPayload) -> impl Future<Output = Result<()>>;
    fn logout_all(&self, user_id: &str) -> impl Future<Output = Result<()>>;
    fn find_jwt_by_jti(&self, jti: &str) -> impl Future<Output = Result<RefreshToken>>;
}
impl JwtServiceTrait for AuthService {
    #[tracing::instrument(
        name = "auth.consume_refresh_token",
        skip(self, refresh_token, configuration)
    )]
    async fn consume_refresh_token(
        &self,
        refresh_token: &str,
        configuration: &AnzarConfiguration,
    ) -> Result<String> {
        let claims: Claims = JwtDecoder::new(refresh_token, configuration).decode()?;
        if claims.token_type != crate::extractors::TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        let _ = match self.jwt_repository.find_and_consume(&claims).await {
            Ok(token) => Ok(token.user_id),
            Err(Error::Unauthenticated { .. }) => {
                // Potential breach — revoke everything for this user
                // TODO: Send an email indicating a breach
                self.jwt_repository.revoke(&claims.sub).await?;
                Err(Error::Unauthenticated(AuthError::TokenReplay {
                    token_type: TokenErrorType::RefreshToken,
                }))
            }
            Err(e) => Err(e), // NotFound, Expired bubble up as-is
        };

        Ok(claims.sub)
    }

    #[tracing::instrument(
        name = "auth.issue_jwt", skip(self, user, configuration), fields(user.id = user.id)
    )]
    async fn issue_jwt(
        &self,
        user: &User,
        configuration: &AnzarConfiguration,
    ) -> Result<IssuedTokens> {
        let user_id = user.id()?;

        let tokens: IssuedTokens = JwtEncoder::new(user, configuration).encode()?;

        let refresh_token = RefreshToken::new(&tokens)
            .with_user_id(user_id)
            .with_expire_at(configuration.auth.jwt.refresh_token_expires_in);

        self.jwt_repository.insert(refresh_token).await?;
        Ok(tokens)
    }

    #[tracing::instrument(name = "auth.invalidate_jwt", skip(self, refresh_token, configuration))]
    async fn invalidate_jwt(
        &self,
        refresh_token: &str,
        configuration: &AnzarConfiguration,
    ) -> Result<()> {
        let claims: Claims = JwtDecoder::new(refresh_token, configuration).decode()?;

        if claims.token_type != TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        self.jwt_repository.invalidate(claims.jti).await?;
        Ok(())
    }

    // async fn logout(&self, payload: AuthPayload) -> Result<()> {
    //     self.jwt_service.invalidate(&payload.jti).await?;
    //     self.session_service.revoke(&payload.user_id).await?;
    //     Ok(())
    // }
    #[tracing::instrument(name = "auth.logout_all", skip(self), fields(user.id = user_id))]
    async fn logout_all(&self, user_id: &str) -> Result<()> {
        self.jwt_repository.revoke(user_id).await?;
        self.session_repository.revoke(user_id).await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.find_jwt_by_jti", skip(self, jti))]
    async fn find_jwt_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        self.jwt_repository.find_by_jti(jti).await
    }
}
