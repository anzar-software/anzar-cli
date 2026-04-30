use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::error::Result;

pub struct PostgreSQL {
    pub pool: Pool<Postgres>,
}
impl PostgreSQL {
    pub async fn start(conn: &str) -> Result<Self> {
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

        Ok(Self { pool })
    }

    pub async fn create_database(&self, name: &str) -> Result<()> {
        sqlx::query(&format!("CREATE DATABASE \"{}\"", name))
            .execute(&self.pool)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error_code = "InternalError::Database",
                    "Failed to connect to database - {e}"
                );
            })?;

        self.pool.close().await;

        Ok(())
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let path = std::path::Path::new("migrations/postgres");
        if path.exists() {
            let migrator = sqlx::migrate::Migrator::new(path).await?;
            migrator.run(&self.pool).await.expect("migrations to run");
        }

        Ok(())
    }
}
