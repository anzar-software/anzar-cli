use crate::config::AppState;
use crate::error::Result;
use crate::{scopes::user::User, services::session::model::Session};

// [ SessionTrait ]
pub trait SessionServiceTrait {
    fn issue_session(&self, user_id: &User) -> impl Future<Output = Result<String>>;
    fn find_session(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
    fn invalidate_session(&self, session_id: &str) -> impl Future<Output = Result<()>>;
    fn extend_timeout(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
}
impl SessionServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.issue_session", skip(self), fields(user.id = user.id)
    )]
    async fn issue_session(&self, user: &User) -> Result<String> {
        let user_id = user.id()?;

        self.auth_service.session_repository.revoke(user_id).await?;

        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        let session = Session::default()
            .with_user_id(user_id)
            .with_token(&hashed_token);
        self.auth_service.session_repository.insert(session).await?;

        Ok(token)
    }

    #[tracing::instrument(name = "auth.find_session", skip(self, token))]
    async fn find_session(&self, token: &str) -> Result<Session> {
        let hash = self.crypto.token.hash(token);
        self.auth_service.session_repository.find(&hash).await
    }

    #[tracing::instrument(name = "auth.invalidate_session", skip(self, token))]
    async fn invalidate_session(&self, token: &str) -> Result<()> {
        self.auth_service
            .session_repository
            .invalidate(token)
            .await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.extend_timeout", skip(self, session_id))]
    async fn extend_timeout(&self, session_id: &str) -> Result<Session> {
        self.auth_service
            .session_repository
            .extend_timeout(session_id)
            .await
    }
}
