use crate::config::AppState;
use crate::domain::model::EmailVerificationToken;
use crate::error::Result;

use super::traits::EmailVerificationTokenServiceTrait;

impl EmailVerificationTokenServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.create_verification_email",
        skip(self),
        fields(user.id = user_id)
    )]
    async fn create_verification_email(&self, user_id: &str) -> Result<String> {
        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        let expiry = self.configuration.auth.email.verification.token_expires_in;
        let otp = EmailVerificationToken::default()
            .with_user_id(user_id)
            .with_token_hash(&hashed_token)
            .with_expiray(chrono::Duration::seconds(expiry));

        self.repositories
            .email_verification_token_repository
            .insert(otp)
            .await?;

        Ok(token)
    }

    #[tracing::instrument(name = "auth.validate_email_verification_token", skip(self, token))]
    async fn consume_email_verification_token(
        &self,
        token: &str,
    ) -> Result<EmailVerificationToken> {
        let hash = self.crypto.token.hash(token);

        self.repositories
            .email_verification_token_repository
            .consume(&hash)
            .await
    }

    #[tracing::instrument(name = "auth.invalidate_email_verification_token", skip(self, id))]
    async fn invalidate_email_verification_token(
        &self,
        id: &str,
    ) -> Result<EmailVerificationToken> {
        self.repositories
            .email_verification_token_repository
            .invalidate(id)
            .await
    }

    #[tracing::instrument(name = "auth.revoke_email_verification_token", skip(self), fields(user.id = user_id))]
    async fn revoke_email_verification_token(&self, user_id: &str) -> Result<()> {
        self.repositories
            .email_verification_token_repository
            .revoke(user_id)
            .await
    }
}
