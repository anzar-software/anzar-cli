use crate::domain::model::{IssuedTokens, RefreshToken};
use crate::error::Result;

pub trait JwtServiceTrait {
    fn consume_refresh_token(&self, refresh_token: &str) -> impl Future<Output = Result<String>>;
    fn issue_jwt(&self, user: &str) -> impl Future<Output = Result<IssuedTokens>>;
    fn invalidate_jwt(&self, refresh_token: &str) -> impl Future<Output = Result<()>>;
    // fn logout(&self, payload: AuthPayload) -> impl Future<Output = Result<()>>;
    fn logout_all(&self, user_id: &str) -> impl Future<Output = Result<()>>;
    fn find_jwt_by_jti(&self, jti: &str) -> impl Future<Output = Result<RefreshToken>>;
}
