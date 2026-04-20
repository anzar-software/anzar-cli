use crate::error::Error;

pub struct MemCache {}
impl MemCache {
    pub async fn start(conn: &str) -> Result<memcache::Client, Error> {
        let db = memcache::connect(conn)?;
        Ok(db)
    }
}
