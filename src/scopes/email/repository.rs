use std::sync::Arc;

use crate::utils::query::QueryBuilder;
use crate::{
    adapters::database::DatabaseAdapter,
    error::{Error, Result, TokenErrorType},
    scopes::email::model::EmailVerificationToken,
};

use chrono::Utc;

#[derive(Clone)]
pub struct EmailVerificationTokenRepository {
    adapter: Arc<dyn DatabaseAdapter<EmailVerificationToken>>,
}

impl EmailVerificationTokenRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<EmailVerificationToken>>) -> Self {
        Self { adapter }
    }

    pub async fn insert(&self, otp: EmailVerificationToken) -> Result<()> {
        self.adapter.insert(otp).await.map(|_| ()).map_err(|e| {
            tracing::error!(
                "Failed to insert email verification token to database: {:?}",
                e
            );
            Error::Internal(crate::error::InternalError::TokenCreation {
                token_type: TokenErrorType::PasswordResetToken,
            })
        })
    }

    pub async fn find(&self, hash: &str) -> Result<EmailVerificationToken> {
        let filter = QueryBuilder::default().eq("token", hash);

        match self.adapter.find_one(filter).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::EmailVerificationToken,
                },
            )),
            Err(err) => Err(err),
        }
    }

    pub async fn invalidate(&self, id: &str) -> Result<EmailVerificationToken> {
        let filter = QueryBuilder::default().eq("id", id);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::EmailVerificationToken,
                },
            )),
            Err(err) => Err(err),
        }
    }

    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        // FIXME delete tokens not update
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        self.adapter
            .update_many(filter, update)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke email verification tokens: {:?}", e);
                e
            })?;

        Ok(())
    }
}
