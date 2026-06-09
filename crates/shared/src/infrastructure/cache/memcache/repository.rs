use crate::error::CoreError;

pub struct MemCache {}
impl MemCache {
    pub async fn start(conn: &str) -> Result<memcache::Client, CoreError> {
        let db = memcache::connect(conn).inspect_err(|e| {
            tracing::error!(
                error_code = "InternalError::Database",
                "Failed to connect to database - {e}"
            );
        })?;

        Ok(db)
    }
}
