use std::sync::Arc;

use crate::adapters::database::DatabaseAdapter;
use crate::error::{CredentialField, Error, ResourceKind, Result, ValidationError};
use crate::utils::query::QueryBuilder;

use super::model::Account;

#[derive(Clone)]
pub struct AccountRepository {
    adapter: Arc<dyn DatabaseAdapter<Account>>,
}

impl AccountRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<Account>>) -> Self {
        Self { adapter }
    }
}

impl AccountRepository {
    pub async fn insert(&self, account: Account) -> Result<()> {
        match self.adapter.insert(account).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to insert Account to database");
                Err(err)
            }
        }
    }

    pub async fn find(&self, user_id: &str) -> Result<Account> {
        let filter = QueryBuilder::default().eq("userId", user_id);

        match self.adapter.find_one(filter).await {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(Error::NotFound(ResourceKind::User {
                id: Some(user_id.into()),
                email: None,
            })),
            Err(err) => Err(err),
        }
    }

    pub async fn update_password(&self, user_id: &str, password: &str) -> Result<Account> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        let update = QueryBuilder::default().set("password", password);

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(account)) => Ok(account),
            Ok(None) => Err(Error::Validation(ValidationError::Missing {
                field: CredentialField::Password,
            })),
            Err(err) => Err(err),
        }
    }

    pub async fn lock_account(&self, user_id: &str) -> Result<Account> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        let update = QueryBuilder::default().set("locked", true);

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(account)) => Ok(account),
            Ok(None) => Err(Error::Validation(ValidationError::Missing {
                field: CredentialField::Username,
            })),
            Err(err) => Err(err),
        }
    }
    pub async fn unlock_account(&self, user_id: &str) -> Result<Account> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        let update = QueryBuilder::default().set("locked", false);

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(account)) => Ok(account),
            Ok(None) => Err(Error::Validation(ValidationError::Missing {
                field: CredentialField::Username,
            })),
            Err(err) => Err(err),
        }
    }
}
