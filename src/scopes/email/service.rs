use crate::error::{CredentialField, Error, Result, TokenErrorType, ValidationError};
use crate::scopes::auth::service::AuthService;
use crate::scopes::email::model::EmailVerificationToken;
use crate::utils::{SecureToken, TokenHasher};

pub trait EmailVerificationTokenServiceTrait {
    fn validate_email_verification_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<EmailVerificationToken>>;

    fn invalidate_email_verification_token(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<EmailVerificationToken>>;
    fn revoke_email_verification_token(&self, user_id: &str) -> impl Future<Output = Result<()>>;
    fn insert_email_verification_token(
        &self,
        otp: EmailVerificationToken,
    ) -> impl Future<Output = Result<()>>;
}
impl EmailVerificationTokenServiceTrait for AuthService {
    async fn validate_email_verification_token(
        &self,
        token: &str,
    ) -> Result<EmailVerificationToken> {
        let hash = SecureToken::hash(token);

        // 2. Checks the database for a matching token
        let verification_token = self.email_verification_token_repository.find(&hash).await?;
        let verification_token_id = verification_token.id.as_ref().ok_or_else(|| {
            Error::Validation(ValidationError::Malformed {
                field: CredentialField::ObjectId,
            })
        })?;

        // 3. Verify token isn't expired or already used
        if verification_token.used_at.is_some() {
            return Err(Error::Unauthenticated(
                crate::error::AuthError::TokenReplay {
                    token_type: TokenErrorType::EmailVerificationToken,
                },
            ));
        }
        if chrono::Utc::now() > verification_token.expires_at {
            self.password_reset_token_repository
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

    async fn invalidate_email_verification_token(
        &self,
        id: &str,
    ) -> Result<EmailVerificationToken> {
        self.email_verification_token_repository
            .invalidate(id)
            .await
    }
    async fn revoke_email_verification_token(&self, user_id: &str) -> Result<()> {
        self.email_verification_token_repository
            .revoke(user_id)
            .await
    }
    async fn insert_email_verification_token(&self, otp: EmailVerificationToken) -> Result<()> {
        self.email_verification_token_repository.insert(otp).await
    }
}
