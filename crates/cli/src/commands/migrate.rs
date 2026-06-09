use owo_colors::OwoColorize;
use sqlx::{Executor, Pool, migrate::Migrator, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};

use crate::{
    error::{Error, Result},
    shared::support,
};
use shared::config::{AnzarConfiguration, database::DatabaseDriver};

enum DatabasePool {
    Sqlite(Pool<sqlx::Sqlite>),
    Postgres(Pool<sqlx::Postgres>),
}

pub async fn run(path: Option<String>) -> Result<()> {
    let config = support::load_config()?;

    let migration_path = std::path::Path::new("migrations");
    if !migration_path.exists() {
        support::print_result(
            "No migrations directory found",
            false,
            Some(&format!(
                "create a ./migrations directory by running ( {} )",
                "anzar generate".bold().white()
            )),
        );
        return Ok(());
    }

    let database_pool: DatabasePool = match connect(config, path).await {
        Ok(pool) => pool,
        Err(e) => {
            support::print_result(
                "Failed to connect",
                false,
                // Some(&format!("check connection_string in anzar.yml — {}", e)),
                Some(&e.to_string()),
            );
            return Ok(());
        }
    };

    support::print_result("Running migrations", true, None);
    let migrator = Migrator::new(migration_path).await?;
    let response = match database_pool {
        DatabasePool::Sqlite(pool) => migrator.run(&pool).await,
        DatabasePool::Postgres(pool) => migrator.run(&pool).await,
    };

    match response {
        Ok(_) => {
            support::print_result("Migrations applied", true, None);
            println!();
            println!("  {} your database is up to date", "→".cyan().bold());
        }
        Err(e) => {
            let hint = match &e {
                sqlx::migrate::MigrateError::VersionMissing(v) => {
                    format!("migration version {} is missing from ./migrations", v)
                }
                sqlx::migrate::MigrateError::Dirty(v) => {
                    format!(
                        "migration version {} failed previously — fix or revert it first",
                        v
                    )
                }
                _ => e.to_string(),
            };
            support::print_result("Migrations failed", false, Some(&hint));
            return Ok(());
        }
    }

    Ok(())
}

async fn connect(config: AnzarConfiguration, path: Option<String>) -> Result<DatabasePool, Error> {
    match config.database.driver {
        DatabaseDriver::MongoDB => {
            support::print_result(
                "MongoDB is not supported",
                false,
                Some("switch to SQLite or PostgreSQL in anzar.yml"),
            );
            Err(Error::InvalidConfig {
                key: "database.driver".to_string(),
                reason: "MongoDB is not supported".to_string(),
            })
        }
        DatabaseDriver::SQLite => {
            support::print_result("Connecting to database", true, None);

            let path = match path {
                Some(p) => p,
                None => {
                    return Err(Error::InvalidConfig {
                        key: "path".to_string(),
                        reason: format!(
                            "SQLite requires a file path — run: {}",
                            "anzar migrate --path data/data.db".bold().white()
                        ),
                    });
                }
            };

            let pool = SqlitePoolOptions::new()
                .after_connect(|conn, _| {
                    Box::pin(async move {
                        conn.execute("PRAGMA foreign_keys = ON;").await?;
                        Ok(())
                    })
                })
                .connect(&path)
                .await
                .map_err(|e| Error::InvalidConfig {
                    key: "path".to_string(),
                    reason: format!("check {path} is a valid path — {e}"),
                })?;

            Ok(DatabasePool::Sqlite(pool))
        }
        DatabaseDriver::PostgreSQL => {
            // println!();
            support::print_result("Connecting to database", true, None);

            let conn = config
                .database
                .connection_string
                .replace("@db", "@localhost");

            // Probe TCP before handing off to sqlx so we can distinguish
            let is_reachable = std::net::TcpStream::connect("localhost:5432").is_ok();

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&conn)
                .await
                .map_err(|_| {
                    if !is_reachable {
                        Error::Other(format!(
                            "PostgreSQL is not running — start it with {}",
                            "docker compose up -d".bold().white(),
                        ))
                    } else {
                        Error::Other(format!(
                            "invalid connection string — check {} in anzar.yml",
                            "database.connection_string".bold().white(),
                        ))
                    }
                })?;

            Ok(DatabasePool::Postgres(pool))
        }
    }
}
