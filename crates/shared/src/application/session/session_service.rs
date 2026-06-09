use crate::error::Result;

use crate::domain::model::Session;
use crate::intern::session::SessionService;

impl SessionService {
    #[tracing::instrument(
        name = "auth.issue_session", skip(self), fields(user.id = user_id)
    )]
    pub async fn issue_session(
        &self,
        user_id: &str,
        full_permissions: Vec<String>,
    ) -> Result<String> {
        self.session_repository.revoke(user_id).await?;

        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        let session = Session::default()
            .with_user_id(user_id)
            .with_token(&hashed_token)
            .with_role(&self.configuration.auth.rbac.default_role)
            .with_permissions(full_permissions);
        self.session_repository.insert(session).await?;

        Ok(token)
    }

    #[tracing::instrument(name = "auth.find_session", skip(self, token))]
    pub async fn find_session(&self, token: &str) -> Result<Session> {
        let hash = self.crypto.token.hash(token);
        self.session_repository.find(&hash).await
    }

    #[tracing::instrument(name = "auth.invalidate_session", skip(self, token))]
    pub async fn invalidate_session(&self, token: &str) -> Result<()> {
        self.session_repository.invalidate(token).await?;
        Ok(())
    }

    #[tracing::instrument(name = "auth.extend_timeout", skip(self, session_id))]
    pub async fn extend_timeout(&self, session_id: &str) -> Result<Session> {
        self.session_repository.extend_timeout(session_id).await
    }
}
