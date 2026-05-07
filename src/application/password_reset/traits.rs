use crate::domain::model::{ExpiringLink, PasswordResetToken};
use crate::error::Result;

pub trait PasswordResetTokenServiceTrait {
    fn insert_password_reset_token(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<ExpiringLink>>;
    fn validate_reset_password_token(
        &self,
        token: &str,
    ) -> impl Future<Output = Result<PasswordResetToken>>;
    fn invalidate_password_reset_token(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<PasswordResetToken>>;
    fn revoke_password_reset_token(&self, user_id: &str) -> impl Future<Output = Result<()>>;
}
