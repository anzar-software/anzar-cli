use crate::config::AppState;
use crate::error::Result;

use crate::domain::model::Account;

use super::traits::AccountServiceTrait;

impl AccountServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.find_account", skip(self), fields(user.id = user_id)
    )]
    async fn find_account(&self, user_id: &str) -> Result<Account> {
        self.repositories.account_repository.find(user_id).await
    }

    #[tracing::instrument(
        name = "auth.update_user_password", skip(self, hash), fields(user.id = id)
    )]
    async fn update_user_password(&self, id: &str, hash: &str) -> Result<Account> {
        self.repositories
            .account_repository
            .update_password(id, hash)
            .await
    }

    #[tracing::instrument(
        name = "auth.unlock_account", skip(self), fields(user.id = id)
    )]
    async fn unlock_account(&self, id: &str) -> Result<Account> {
        self.repositories
            .account_repository
            .unlock_account(id)
            .await
    }

    #[tracing::instrument(
        name = "auth.validate_account",
        skip(self),
        fields(user.id = id)
    )]
    async fn validate_account(&self, id: &str) -> Result<()> {
        self.repositories
            .account_repository
            .validate_account(id)
            .await
    }
}
