use std::sync::Arc;

use crate::utils::query::QueryBuilder;
use crate::{
    adapters::database::DatabaseAdapter,
    error::{Error, Reason, Result, TokenErrorType},
};

use super::model::PasswordResetToken;
use chrono::Utc;

#[derive(Clone)]
pub struct PasswordResetTokenRepository {
    adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>,
}

impl PasswordResetTokenRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>) -> Self {
        Self { adapter }
    }

    pub async fn insert(&self, otp: PasswordResetToken) -> Result<String> {
        self.adapter.insert(otp).await.map_err(|e| {
            tracing::error!("Failed to insert password reset token to database: {:?}", e);
            Error::TokenCreationFailed {
                token_type: crate::error::TokenErrorType::PasswordResetToken,
            }
        })
    }

    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        // FIXME delete instead
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        self.adapter
            .update_many(filter, update)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke password tokens: {:?}", e);
                Error::TokenRevocationFailed {
                    token_id: "".into(),
                }
            })?;

        Ok(())
    }

    pub async fn find(&self, token: &str) -> Result<PasswordResetToken> {
        // "expiresAt": {
        //     "$lt": Utc::now().to_string()
        // },
        let filter = QueryBuilder::default().eq("token", token);

        match self.adapter.find_one(filter).await {
            Ok(Some(password_reset_token)) => Ok(password_reset_token),
            Ok(None) => Err(Error::InvalidToken {
                token_type: TokenErrorType::PasswordResetToken,
                reason: Reason::NotFound,
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn invalidate(&self, id: &str) -> Result<PasswordResetToken> {
        let filter = QueryBuilder::default().eq("id", id);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(password_reset_token)) => Ok(password_reset_token),
            Ok(None) => Err(Error::InvalidToken {
                token_type: TokenErrorType::PasswordResetToken,
                reason: Reason::NotFound,
            }),
            Err(err) => Err(err),
        }
    }
}
