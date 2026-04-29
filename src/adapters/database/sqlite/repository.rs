use sqlx::{Executor, Pool, Sqlite, SqlitePool};

use crate::error::Error;

pub struct SQLite {}
impl SQLite {
    pub async fn start(conn: &str) -> Result<Pool<Sqlite>, Error> {
        let pool = SqlitePool::connect(conn).await.inspect_err(|e| {
            tracing::error!(
                error_code = "InternalError::Database",
                "Failed to connect to database - {e}"
            );
        })?;

        pool.execute("PRAGMA foreign_keys = ON;").await?;
        Ok(pool)
    }
}
