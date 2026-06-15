use sqlx::{Executor, Pool, Sqlite, SqlitePool};

use crate::error::Result;

pub struct SQLite {
    pub pool: Pool<Sqlite>,
}
impl SQLite {
    pub async fn start(conn: &str) -> Result<Self> {
        let pool = SqlitePool::connect(conn).await.inspect_err(|e| {
            tracing::error!(
                error_code = "InternalError::Database",
                "Failed to connect to database - {e}"
            );
        })?;

        pool.execute("PRAGMA foreign_keys = ON;").await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let path = std::path::Path::new("../../migrations/sqlite");
        if path.exists() {
            let migrator = sqlx::migrate::Migrator::new(path).await?;
            migrator
                .run(&self.pool)
                .await
                .inspect_err(|e| tracing::error!("Failed to run migrations - {e}"))?;
        }

        Ok(())
    }
}
