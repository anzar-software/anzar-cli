use chrono::Utc;
use std::sync::Arc;

use crate::error::InternalError;
use crate::error::{Error, Result, TokenErrorType};

use super::model::EmailVerificationToken;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;

#[derive(Clone)]
pub struct EmailVerificationTokenRepository {
    adapter: Arc<dyn DatabaseAdapter<EmailVerificationToken>>,
}

impl EmailVerificationTokenRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<EmailVerificationToken>>) -> Self {
        Self { adapter }
    }

    #[tracing::instrument(name = "db.email.insert", skip(self, otp))]
    pub async fn insert(&self, otp: EmailVerificationToken) -> Result<()> {
        match self.adapter.insert(otp).await {
            Ok(_id) => Ok(()),
            Err(e) => {
                tracing::error!("Failed to insert user to database - {e}");
                Err(Error::Internal(InternalError::Database(e.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.email.consume", skip(self, hash))]
    pub async fn consume(&self, hash: &str) -> Result<EmailVerificationToken> {
        let filter = QueryBuilder::default()
            .eq("hash", hash)
            .gt("expiresAt", Utc::now())
            .is_null("usedAt");
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::EmailVerificationToken,
                },
            )),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.email.invalidate", skip(self, id))]
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
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.email.revoke", skip(self), fields(user.id = user_id))]
    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        // FIXME delete tokens not update
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.update_many(filter, update).await {
            Ok(_) => Ok(()),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }
}
