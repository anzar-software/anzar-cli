use std::sync::Arc;

use chrono::Utc;

use crate::error::{AuthError, Error, Result, TokenErrorType};

use super::model::RefreshToken;
use super::ports::database::DatabaseAdapter;
use super::ports::query::QueryBuilder;
use crate::domain::model::Claims;

#[derive(Clone)]
pub struct JWTRepository {
    adapter: Arc<dyn DatabaseAdapter<RefreshToken>>,
}
impl JWTRepository {
    pub fn new(adapter: Arc<dyn DatabaseAdapter<RefreshToken>>) -> Self {
        Self { adapter }
    }
}

impl JWTRepository {
    #[tracing::instrument(name = "db.jwt.insert", skip(self, refresh_token))]
    pub async fn insert(&self, refresh_token: RefreshToken) -> Result<()> {
        match self.adapter.insert(refresh_token).await {
            Ok(_id) => Ok(()),
            Err(err) => {
                tracing::error!("Failed to insert account to database - {err}");
                Err(err)
            }
        }
    }

    #[tracing::instrument(
        name = "db.jwt.find_and_consume", skip(self, claims), fields(user.id = claims.sub)
    )]
    pub async fn find_and_consume(&self, claims: &Claims) -> Result<RefreshToken> {
        let filter = QueryBuilder::default()
            .eq("jti", claims.jti.to_string())
            .eq("userId", claims.clone().sub)
            .gt("expiresAt", Utc::now())
            .is_null("usedAt");

        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.jwt.find_by_jti", skip(self, jti))]
    pub async fn find_by_jti(&self, jti: &str) -> Result<RefreshToken> {
        let filter = QueryBuilder::default().eq("jti", jti);

        match self.adapter.find_one(filter).await {
            Ok(Some(token)) => Ok(token),
            Ok(None) => Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.jwt.invalidate", skip(self, jti))]
    pub async fn invalidate(&self, jti: uuid::Uuid) -> Result<RefreshToken> {
        let filter = QueryBuilder::default().eq("jti", jti.to_string());
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        match self.adapter.find_one_and_update(filter, update).await {
            Ok(Some(refresh_token)) => Ok(refresh_token),
            Ok(None) => Err(Error::Unauthenticated(AuthError::TokenInvalid {
                token_type: TokenErrorType::RefreshToken,
            })),
            Err(err) => {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "db.jwt.revoke", skip(self), fields(user.id = user_id))]
    pub async fn revoke(&self, user_id: &str) -> Result<()> {
        let filter = QueryBuilder::default().eq("userId", user_id);
        let update = QueryBuilder::default().set("usedAt", Utc::now());

        self.adapter
            .update_many(filter, update)
            .await
            .inspect_err(|err| {
                tracing::error!(error_code = "InternalError::Database", error = %err, "Database query failed");
            })?;

        Ok(())
    }
}
