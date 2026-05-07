use crate::domain::model::Account;
use crate::error::Result;

pub trait AccountServiceTrait {
    fn find_account(&self, user_id: &str) -> impl Future<Output = Result<Account>>;
    fn update_user_password(&self, id: &str, hash: &str) -> impl Future<Output = Result<Account>>;
    fn unlock_account(&self, id: &str) -> impl Future<Output = Result<Account>>;
    fn validate_account(&self, id: &str) -> impl Future<Output = Result<()>>;
}
