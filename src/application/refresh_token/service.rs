use crate::application::traits::{PermissionServiceTrait, RoleServiceTrait};
use crate::config::AppState;
use crate::error::{AuthError, Error, Result, TokenErrorType};

use super::traits::JwtServiceTrait;
use crate::domain::model::{Claims, IssuedTokens, RefreshToken, TokenType};

impl JwtServiceTrait for AppState {
    #[tracing::instrument(name = "auth.consume_refresh_token", skip(self, refresh_token))]
    async fn consume_refresh_token(&self, refresh_token: &str) -> Result<String> {
        let claims: Claims = self.crypto.jwt()?.decode(refresh_token)?;

        if claims.token_type != TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        match self
            .repositories
            .jwt_repository
            .find_and_consume(&claims)
            .await
        {
            Ok(_) => Ok(claims.sub),
            Err(Error::Unauthenticated { .. }) => {
                // Potential breach — revoke everything for this user
                // TODO: Send an email indicating a breach
                self.repositories.jwt_repository.revoke(&claims.sub).await?;
                Err(Error::Unauthenticated(AuthError::TokenReplay {
                    token_type: TokenErrorType::RefreshToken,
                }))
            }
            Err(e) => Err(e), // NotFound, Expired bubble up as-is
        }
    }

    #[tracing::instrument(
        name = "auth.issue_jwt", skip(self), fields(user.id = user_id)
    )]
    async fn issue_jwt(&self, user_id: &str) -> Result<IssuedTokens> {
        let jwt_config = self.configuration.auth.jwt()?;
        let rbac_config = &self.configuration.auth.rbac;
        let jwt = self.crypto.jwt()?;

        // permissions = get list of permissions by role_id
        let mut full_permissions: Vec<String> = Vec::new();

        if self.configuration.auth.rbac.enabled {
            let roles = self.find_roles_by_user_id(user_id).await?;

            for role in roles {
                let role_id = role.id()?;
                let response = self.find_permissions_by_role_id(role_id).await?;

                let mut permissions = response.iter().map(|r| r.name.clone()).collect();

                // NOTE
                // YOU CAN USE LOCAL FETCH FROM `config.yml`
                // let mut permissions: Vec<String> = rbac_config
                //     .roles
                //     .iter()
                //     .filter(|r| r.name == role.name)
                //     .flat_map(|r| r.permissions.iter().cloned())
                //     .collect();

                full_permissions.append(&mut permissions);
            }
        }

        let (access_claims, refresh_claims) = Claims::new(user_id, &rbac_config.default_role)
            .with_issuer(&jwt_config.issuer)
            .with_audience(&jwt_config.audience)
            .with_permissions(full_permissions)
            .into_token_pair(jwt_config);
        let jti = refresh_claims.jti;

        let refresh_token = RefreshToken::new(&jti.to_string())
            .with_user_id(user_id)
            .with_expire_at(jwt_config.refresh_token_expires_in);
        self.repositories
            .jwt_repository
            .insert(refresh_token)
            .await?;

        let access = jwt.encode(access_claims)?;
        let refresh = jwt.encode(refresh_claims)?;
        Ok(IssuedTokens::default()
            .with_access_token(&access)
            .with_refresh_token(&refresh)
            .with_jti(jti))
    }

    #[tracing::instrument(name = "auth.invalidate_jwt", skip(self, refresh_token))]
    async fn invalidate_jwt(&self, refresh_token: &str) -> Result<()> {
        let claims: Claims = self.crypto.jwt()?.decode(refresh_token)?;

        if claims.token_type != TokenType::RefreshToken {
            return Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            }));
        }

        self.repositories
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
        self.repositories.jwt_repository.revoke(user_id).await?;
        self.repositories.session_repository.revoke(user_id).await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.find_jwt_by_jti", skip(self, jti))]
    async fn find_jwt_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        self.repositories.jwt_repository.find_by_jti(jti).await
    }
}
