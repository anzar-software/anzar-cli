use redis::aio::ConnectionManager;

use crate::error::Error;

pub struct Redis {}
impl Redis {
    pub async fn start(conn: &str) -> Result<ConnectionManager, Error> {
        let client = redis::Client::open(conn).map_err(|e| {
            tracing::error!("Failed to open Redis connection: {e}");
            Error::InternalServerError(e.to_string())
        })?;

        let connection = client.get_connection_manager().await.map_err(|e| {
            tracing::error!("Failed to get Redis connection manager: {e}");
            Error::InternalServerError(e.to_string())
        })?;

        Ok(connection)
    }
}
