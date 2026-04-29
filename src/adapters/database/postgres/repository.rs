use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::error::Error;

pub struct PostgreSQL {}
impl PostgreSQL {
    pub async fn start(conn: &str) -> Result<Pool<Postgres>, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(conn)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error_code = "InternalError::Database",
                    "Failed to connect to database - {e}"
                );
            })?;

        Ok(pool)
    }
}
