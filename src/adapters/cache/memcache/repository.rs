use crate::error::Error;

pub struct MemCache {}
impl MemCache {
    pub async fn start(conn: &str) -> Result<memcache::Client, Error> {
        let db = memcache::connect(conn).map_err(|e| {
            tracing::error!("Failed to open MemCache connection: {e}");
            Error::InternalServerError(e.to_string())
        })?;

        Ok(db)
    }
}
