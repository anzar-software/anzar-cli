use crate::config::AppState;
use crate::error::{AuthError, Error, Result, TokenErrorType};
use crate::extractors::{Claims, TokenType};
use crate::scopes::user::User;

use super::{IssuedTokens, RefreshToken};

pub trait JwtServiceTrait {
    fn consume_refresh_token(&self, refresh_token: &str) -> impl Future<Output = Result<String>>;
    fn issue_jwt(&self, user: &User) -> impl Future<Output = Result<IssuedTokens>>;
    fn invalidate_jwt(&self, refresh_token: &str) -> impl Future<Output = Result<()>>;
    // fn logout(&self, payload: AuthPayload) -> impl Future<Output = Result<()>>;
    fn logout_all(&self, user_id: &str) -> impl Future<Output = Result<()>>;
    fn find_jwt_by_jti(&self, jti: &str) -> impl Future<Output = Result<RefreshToken>>;
}
impl JwtServiceTrait for AppState {
    #[tracing::instrument(name = "auth.consume_refresh_token", skip(self, refresh_token))]
    async fn consume_refresh_token(&self, refresh_token: &str) -> Result<String> {
        let claims: Claims = self.crypto.jwt()?.decode(refresh_token)?;

        if claims.token_type != crate::extractors::TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        match self
            .auth_service
            .jwt_repository
            .find_and_consume(&claims)
            .await
        {
            Ok(_) => Ok(claims.sub),
            Err(Error::Unauthenticated { .. }) => {
                // Potential breach — revoke everything for this user
                // TODO: Send an email indicating a breach
                self.auth_service.jwt_repository.revoke(&claims.sub).await?;
                Err(Error::Unauthenticated(AuthError::TokenReplay {
                    token_type: TokenErrorType::RefreshToken,
                }))
            }
            Err(e) => Err(e), // NotFound, Expired bubble up as-is
        }
    }

    #[tracing::instrument(
        name = "auth.issue_jwt", skip(self, user), fields(user.id = user.id)
    )]
    async fn issue_jwt(&self, user: &User) -> Result<IssuedTokens> {
        let jwt_config = self.configuration.auth.jwt()?;
        let jwt = self.crypto.jwt()?;

        let user_id = user.id()?;
        let (access_claims, refresh_claims) = Claims::new(user_id, user.role.clone())
            .with_issuer(&jwt_config.issuer)
            .with_audience(&jwt_config.audience)
            .into_token_pair(jwt_config);

        let jti = refresh_claims.jti;
        let access = jwt.encode(access_claims)?;
        let refresh = jwt.encode(refresh_claims)?;

        let tokens = IssuedTokens::default()
            .with_access_token(&access)
            .with_refresh_token(&refresh)
            .with_jti(jti);

        let refresh_token = RefreshToken::new(&tokens)
            .with_user_id(user_id)
            .with_expire_at(jwt_config.refresh_token_expires_in);

        self.auth_service
            .jwt_repository
            .insert(refresh_token)
            .await?;
        Ok(tokens)
    }

    #[tracing::instrument(name = "auth.invalidate_jwt", skip(self, refresh_token))]
    async fn invalidate_jwt(&self, refresh_token: &str) -> Result<()> {
        let claims: Claims = self.crypto.jwt()?.decode(refresh_token)?;

        if claims.token_type != TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        self.auth_service
            .jwt_repository
            .invalidate(claims.jti)
            .await?;
        Ok(())
    }

    // async fn logout(&self, payload: AuthPayload) -> Result<()> {
    //     self.jwt_service.invalidate(&payload.jti).await?;
    //     self.session_service.revoke(&payload.user_id).await?;
    //     Ok(())
    // }
    #[tracing::instrument(name = "auth.logout_all", skip(self), fields(user.id = user_id))]
    async fn logout_all(&self, user_id: &str) -> Result<()> {
        self.auth_service.jwt_repository.revoke(user_id).await?;
        self.auth_service.session_repository.revoke(user_id).await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.find_jwt_by_jti", skip(self, jti))]
    async fn find_jwt_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        self.auth_service.jwt_repository.find_by_jti(jti).await
    }
}
