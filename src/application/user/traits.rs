use crate::domain::model::{
    Account, AccountStatus, CreateUserOutcome, LoginRequest, RegisterRequest, User,
};
use crate::error::Result;

pub trait UserServiceTrait {
    fn authenticate_user(
        &self,
        body: &LoginRequest,
        device_cookie: Option<&str>,
    ) -> impl Future<Output = Result<(User, Account, AccountStatus, u8)>>;
    fn register_failed_attempt(
        &self,
        user: &str,
        device_cookie: Option<&str>,
    ) -> impl Future<Output = Result<u8>>;
    fn create_user(&self, body: RegisterRequest)
    -> impl Future<Output = Result<CreateUserOutcome>>;
    fn find_user_by_email(&self, email: &str) -> impl Future<Output = Result<User>>;
    fn find_user(&self, id: &str) -> impl Future<Output = Result<User>>;
}
