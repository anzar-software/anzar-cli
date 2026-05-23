use crate::config::AppState;
use crate::domain::model::{ExpiringLink, PasswordResetToken};
use crate::error::Result;

use super::traits::PasswordResetTokenServiceTrait;

impl PasswordResetTokenServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_password_reset_token", skip(self, user_id))]
    async fn insert_password_reset_token(&self, user_id: &str) -> Result<ExpiringLink> {
        // 4.
        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        // 5.
        let expiry_timestamp = chrono::Utc::now()
            + chrono::Duration::seconds(self.configuration.auth.password.reset.token_expires_in);

        let password_reset_token = PasswordResetToken::default()
            .with_user_id(user_id)
            .with_token_hash(&hashed_token)
            .with_expiray(&expiry_timestamp);

        self.repositories
            .password_reset_token_repository
            .insert(password_reset_token)
            .await?;

        let link = format!(
            "{}/auth/password/reset?token={}",
            self.configuration.app.url, &token
        );

        Ok(ExpiringLink {
            link,
            expires_at: expiry_timestamp,
        })
    }

    #[tracing::instrument(name = "auth.validate_reset_password_token", skip(self, token))]
    async fn validate_reset_password_token(&self, token: &str) -> Result<PasswordResetToken> {
        let hash = self.crypto.token.hash(token);

        self.repositories
            .password_reset_token_repository
            .consume(&hash)
            .await
    }

    #[tracing::instrument(name = "auth.invalidate_password_reset_token", skip(self, id))]
    async fn invalidate_password_reset_token(&self, id: &str) -> Result<PasswordResetToken> {
        self.repositories
            .password_reset_token_repository
            .invalidate(id)
            .await
    }

    #[tracing::instrument(
        name = "auth.revoke_password_reset_token", skip(self), fields(user.id = user_id)
    )]
    async fn revoke_password_reset_token(&self, user_id: &str) -> Result<()> {
        self.repositories
            .password_reset_token_repository
            .revoke(user_id)
            .await
    }
}
