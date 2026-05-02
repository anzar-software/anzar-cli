use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::error::{Error, InternalError, Result, TokenErrorType};
use crate::utils::query::QueryBuilder;
use crate::{adapters::database::DatabaseAdapter, services::session::model::Session};

#[derive(Clone)]
pub struct SessionRepository {
    adapter: Arc<dyn DatabaseAdapter<Session>>,
}

impl SessionRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<Session>>) -> Self {
        Self { adapter }
    }
}

impl SessionRepository {
    #[tracing::instrument(name = "db.account.insert", skip(self, session))]
    pub async fn insert(&self, session: Session) -> Result<()> {
        match self.adapter.insert(session).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to insert account to database - {err}");
                Err(Error::Internal(InternalError::Database(err.to_string())))
            }
        }
    }

    #[tracing::instrument(name = "db.account.find", skip(self, token))]
    pub async fn find(&self, token: &str) -> Result<Session> {
        let filter = QueryBuilder::default().eq("token", token);

        match self.adapter.find_one(filter).await {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(Error::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::SessionToken,
                },
            )),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.account.extend_timeout", skip(self, id))]
    pub async fn extend_timeout(&self, id: &str) -> Result<Session> {
        let filter = QueryBuilder::default().eq("id", id);
        let update = QueryBuilder::default()
            .set("usedAt", Utc::now())
            .set("expiresAt", Utc::now() + Duration::hours(24));

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(Error::NotFound(crate::error::ResourceKind::Token {
                token_type: TokenErrorType::SessionToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.account.invalidate", skip(self, token))]
    pub async fn invalidate(&self, token: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("token", token);

        self.adapter.delete_one(filter).await
    }

    #[tracing::instrument(name = "db.account.revoke", skip(self), fields(user.id = user_id))]
    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);

        self.adapter.delete_many(filter).await.map_err(|err| {
            tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
            Error::Internal(InternalError::Hashing)
        })
    }
}
