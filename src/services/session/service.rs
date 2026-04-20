use crate::error::{CredentialField, Error, Result, ValidationError};
use crate::utils::{SecureToken, TokenHasher};
use crate::{
    scopes::{auth::service::AuthService, user::User},
    services::session::model::Session,
};

// [ SessionTrait ]
pub trait SessionServiceTrait {
    fn issue_session(&self, user_id: &User) -> impl Future<Output = Result<String>>;
    fn find_session(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
    fn invalidate_session(&self, session_id: &str) -> impl Future<Output = Result<()>>;
    fn extend_timeout(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
}
impl SessionServiceTrait for AuthService {
    async fn issue_session(&self, user: &User) -> Result<String> {
        let user_id = user.id.as_ref().ok_or_else(|| {
            Error::Validation(ValidationError::Malformed {
                field: CredentialField::ObjectId,
            })
        })?;

        self.session_repository.revoke(user_id).await?;

        let token = SecureToken::with_size32().generate();
        let hashed_token = SecureToken::hash(&token);

        let session = Session::default()
            .with_user_id(user_id)
            .with_token(&hashed_token);
        self.session_repository.insert(session).await?;

        Ok(token)
    }
    async fn find_session(&self, token: &str) -> Result<Session> {
        self.session_repository.find(token).await
    }

    async fn invalidate_session(&self, token: &str) -> Result<()> {
        self.session_repository.invalidate(token).await?;
        Ok(())
    }
    async fn extend_timeout(&self, session_id: &str) -> Result<Session> {
        self.session_repository.extend_timeout(session_id).await
    }
}
