use crate::domain::model::EmailVerificationToken;
use crate::error::Result;

pub trait EmailVerificationTokenServiceTrait {
    fn insert_email_verification_token(
        &self,
        otp: EmailVerificationToken,
    ) -> impl Future<Output = Result<()>>;

    fn create_verification_email(&self, user_id: &str) -> impl Future<Output = Result<String>>;

    fn validate_email_verification_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<EmailVerificationToken>>;

    fn invalidate_email_verification_token(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<EmailVerificationToken>>;
    fn revoke_email_verification_token(&self, user_id: &str) -> impl Future<Output = Result<()>>;
}
