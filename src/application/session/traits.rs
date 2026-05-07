use crate::domain::model::Session;
use crate::error::Result;

// [ SessionTrait ]
pub trait SessionServiceTrait {
    fn issue_session(&self, user_id: &str) -> impl Future<Output = Result<String>>;
    fn find_session(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
    fn invalidate_session(&self, session_id: &str) -> impl Future<Output = Result<()>>;
    fn extend_timeout(&self, session_id: &str) -> impl Future<Output = Result<Session>>;
}
