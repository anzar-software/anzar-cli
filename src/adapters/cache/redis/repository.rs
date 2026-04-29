use redis::aio::ConnectionManager;

use crate::error::Error;

pub struct Redis {}
impl Redis {
    pub async fn start(conn: &str) -> Result<ConnectionManager, Error> {
        let client = redis::Client::open(conn).inspect_err(|e| {
            tracing::error!(
                error_code = "InternalError::Database",
                "Failed to connect to database - {e}"
            );
        })?;
        let connection = client.get_connection_manager().await?;

        Ok(connection)
    }
}
