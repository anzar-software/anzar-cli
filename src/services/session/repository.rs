use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::error::{Error, InternalError, Result, TokenErrorType};
use crate::utils::query::QueryBuilder;
use crate::utils::{SecureToken, TokenHasher};
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
    pub async fn insert(&self, session: Session) -> Result<()> {
        self.adapter.insert(session).await.map_err(|e| {
            tracing::error!("Failed to insert SessionId to database: {:?}", e);
            Error::Internal(InternalError::TokenCreation {
                token_type: TokenErrorType::SessionToken,
            })
        })?;

        Ok(())
    }

    pub async fn find(&self, token: &str) -> Result<Session> {
        let filter = QueryBuilder::default().eq("token", SecureToken::hash(token));

        match self.adapter.find_one(filter).await {
            Ok(Some(session)) => Ok(session),
            Ok(None) => Err(Error::Unauthenticated(
                crate::error::AuthError::TokenInvalid {
                    token_type: TokenErrorType::SessionToken,
                },
            )),
            Err(err) => Err(err),
        }
    }

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
            Err(err) => Err(err),
        }
    }

    pub async fn invalidate(&self, token: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("token", token);

        self.adapter.delete_one(filter).await
    }

    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);

        self.adapter.delete_many(filter).await.map_err(|e| {
            tracing::error!("Failed to revoke session after security breach: {:?}", e);
            Error::Internal(InternalError::Hashing)
        })
    }
}
