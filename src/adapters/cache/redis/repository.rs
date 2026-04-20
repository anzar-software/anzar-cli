use redis::aio::ConnectionManager;

use crate::error::Error;

pub struct Redis {}
impl Redis {
    pub async fn start(conn: &str) -> Result<ConnectionManager, Error> {
        let client = redis::Client::open(conn)?;
        let connection = client.get_connection_manager().await?;

        Ok(connection)
    }
}
