use crate::config::AppState;
use crate::domain::model::Session;
use crate::error::Result;

use super::traits::SessionServiceTrait;

impl SessionServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.issue_session", skip(self), fields(user.id = user_id)
    )]
    async fn issue_session(&self, user_id: &str) -> Result<String> {
        self.repositories.session_repository.revoke(user_id).await?;

        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        let session = Session::default()
            .with_user_id(user_id)
            .with_token(&hashed_token);
        self.repositories.session_repository.insert(session).await?;

        Ok(token)
    }

    #[tracing::instrument(name = "auth.find_session", skip(self, token))]
    async fn find_session(&self, token: &str) -> Result<Session> {
        let hash = self.crypto.token.hash(token);
        self.repositories.session_repository.find(&hash).await
    }

    #[tracing::instrument(name = "auth.invalidate_session", skip(self, token))]
    async fn invalidate_session(&self, token: &str) -> Result<()> {
        self.repositories
            .session_repository
            .invalidate(token)
            .await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.extend_timeout", skip(self, session_id))]
    async fn extend_timeout(&self, session_id: &str) -> Result<Session> {
        self.repositories
            .session_repository
            .extend_timeout(session_id)
            .await
    }
}
