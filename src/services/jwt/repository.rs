use std::sync::Arc;

use chrono::Utc;

use super::RefreshToken;
use crate::utils::query::QueryBuilder;
use crate::{
    adapters::database::DatabaseAdapter,
    error::{Error, Reason, Result, TokenErrorType},
    extractors::Claims,
};

#[derive(Clone)]
pub struct JWTRepository {
    adapter: Arc<dyn DatabaseAdapter<RefreshToken>>,
}

impl JWTRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<RefreshToken>>) -> Self {
        Self { adapter }
    }

    pub async fn insert(&self, refresh_token: RefreshToken) -> Result<()> {
        self.adapter.insert(refresh_token).await.map_err(|e| {
            tracing::error!("Failed to insert refreshToken to database: {:?}", e);
            Error::TokenCreationFailed {
                token_type: TokenErrorType::RefreshToken,
            }
        })?;

        Ok(())
    }

    pub async fn find_and_consume(&self, claims: &Claims) -> Result<RefreshToken> {
        let filter = QueryBuilder::default()
            .eq("jti", claims.jti)
            .eq("userId", claims.clone().sub);

        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            // Token was already consumed — potential replay attack
            // Consider revoking ALL refresh tokens for this user
            Ok(Some(token)) if token.used_at.is_some() => Err(Error::TokenAlreadyUsed {
                token_id: token.id.unwrap_or_default(),
            }),
            Ok(Some(token)) if Utc::now() > token.expires_at => Err(Error::TokenExpired {
                token_type: TokenErrorType::RefreshToken,
                expired_at: token.expires_at,
            }),

            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::TokenNotFound {
                token_id: "".into(),
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn find_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        let filter = QueryBuilder::default().eq("jti", jti);

        match self.adapter.find_one(filter).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::InvalidToken {
                token_type: TokenErrorType::RefreshToken,
                reason: Reason::NotFound,
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn invalidate(&self, jti: uuid::Uuid) -> Result<RefreshToken> {
        let filter = QueryBuilder::default().eq("jti", jti);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(refresh_token)) => Ok(refresh_token),
            Ok(None) => Err(Error::InvalidToken {
                token_type: TokenErrorType::RefreshToken,
                reason: crate::error::Reason::Malformed,
            }),
            Err(err) => Err(err),
        }
    }
    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        self.adapter
            .update_many(filter, update)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke tokens after security breach: {:?}", e);
                Error::TokenRevocationFailed {
                    token_id: "".into(),
                }
            })?;

        Ok(())
    }
}
