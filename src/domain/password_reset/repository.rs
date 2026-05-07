use chrono::Utc;
use std::sync::Arc;

use crate::error::{AuthError, Error, InternalError, Result, TokenErrorType};

use super::model::PasswordResetToken;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct PasswordResetTokenRepository {
    adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>,
}

impl PasswordResetTokenRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>) -> Self {
        Self { adapter }
    }

    #[tracing::instrument(name = "db.password_reset_token.insert", skip(self, otp))]
    pub async fn insert(&self, otp: PasswordResetToken) -> Result<String> {
        match self.adapter.insert(otp).await {
            Ok(id) => Ok(id),
            Err(err) => {
                tracing::error!("Failed to insert password reset token to database");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(
        name = "db.password_reset_token.revoke", skip(self), fields(user.id = user_id)
    )]
    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        // FIXME delete instead
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        self.adapter
            .update_many(filter, update)
            .await
            .inspect_err(|err| {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
            })?;

        Ok(())
    }

    #[tracing::instrument(name = "db.password_reset_token.find", skip(self, token))]
    pub async fn find(&self, token: &str) -> Result<PasswordResetToken> {
        // "expiresAt": {
        //     "$lt": Utc::now().to_string()
        // },
        let filter = QueryBuilder::default().eq("token", token);

        match self.adapter.find_one(filter).await {
            Ok(Some(password_reset_token)) => Ok(password_reset_token),
            Ok(None) => Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::PasswordResetToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.password_reset_token.invalidate", skip(self, id))]
    pub async fn invalidate(&self, id: &str) -> Result<PasswordResetToken> {
        let filter = QueryBuilder::default().eq("id", id);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(password_reset_token)) => Ok(password_reset_token),
            Ok(None) => Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::PasswordResetToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
