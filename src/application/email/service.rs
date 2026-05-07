use crate::config::AppState;
use crate::domain::model::EmailVerificationToken;
use crate::error::{Error, Result, TokenErrorType};

use super::traits::EmailVerificationTokenServiceTrait;

impl EmailVerificationTokenServiceTrait for AppState {
    #[tracing::instrument(name = "auth.insert_email_verification_token", skip(self, otp))]
    async fn insert_email_verification_token(&self, otp: EmailVerificationToken) -> Result<()> {
        self.repositories
            .email_verification_token_repository
            .insert(otp)
            .await
    }

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
        self.insert_email_verification_token(otp).await?;

        Ok(token)
    }

    #[tracing::instrument(name = "auth.validate_email_verification_token", skip(self, token))]
    async fn validate_email_verification_token(
        &self,
        token: &str,
    ) -> Result<EmailVerificationToken> {
        let hash = self.crypto.token.hash(token);

        // 2. Checks the database for a matching token
        let verification_token = self
            .repositories
            .email_verification_token_repository
            .find(&hash)
            .await?;
        let verification_token_id = verification_token.id()?;

        // 3. Verify token isn't expired or already used
        if verification_token.used_at.is_some() {
            return Err(Error::Unauthenticated(
                crate::error::AuthError::TokenReplay {
                    token_type: TokenErrorType::EmailVerificationToken,
                },
            ));
        }
        if chrono::Utc::now() > verification_token.expires_at {
            self.repositories
                .email_verification_token_repository
                .invalidate(verification_token_id)
                .await?;
            return Err(Error::Unauthenticated(
                crate::error::AuthError::TokenExpired {
                    token_type: TokenErrorType::EmailVerificationToken,
                    expired_at: verification_token.expires_at,
                },
            ));
        }

        Ok(verification_token)
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
