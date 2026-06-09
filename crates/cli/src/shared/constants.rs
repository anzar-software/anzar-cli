use shared::config::database::DatabaseDriver;

// Authentication
pub const JWT_AUTH: &str = include_str!("../templates/auth/jwt.yml");
pub const SESSION_AUTH: &str = include_str!("../templates/auth/session.yml");
pub const CONFIG_TEMPLATE: &str = include_str!("../templates/configuration.yml");

// Compose
pub const COMPOSE: &str = include_str!("../templates/compose.conf.yml");
pub const POSTGRES_COMPOSE: &str = include_str!("../templates/compose/postgres.yml");
pub const MONGO_COMPOSE: &str = include_str!("../templates/compose/mongo.yml");
pub const REDIS: &str = include_str!("../templates/compose/redis.yml");
pub const MEMCACHED: &str = include_str!("../templates/compose/memcached.yml");

// Databases
// -- Sqlite
pub const SQLITE_CREATE_USERS: &str = include_str!("../templates/db/sqlite/create_users.sql");
pub const SQLITE_CREATE_ACCOUNTS: &str = include_str!("../templates/db/sqlite/create_accounts.sql");
pub const SQLITE_CREATE_SESSIONS: &str = include_str!("../templates/db/sqlite/create_sessions.sql");
pub const SQLITE_CREATE_REFRESH_TOKENS: &str =
    include_str!("../templates/db/sqlite/create_refresh_tokens.sql");
pub const SQLITE_CREATE_PASSWORD_RESET_TOKENS: &str =
    include_str!("../templates/db/sqlite/create_password_reset_tokens.sql");
pub const SQLITE_CREATE_EMAIL_VERIFICATION_TOKENS: &str =
    include_str!("../templates/db/sqlite/create_email_verification_tokens.sql");

pub const SQLITE_CREATE_ROLES: &str = include_str!("../templates/db/sqlite/create_roles.sql");
pub const SQLITE_CREATE_USER_ROLES: &str =
    include_str!("../templates/db/sqlite/create_user_roles.sql");
pub const SQLITE_CREATE_PERMISSIONS: &str =
    include_str!("../templates/db/sqlite/create_permissions.sql");
pub const SQLITE_CREATE_ROLE_PERMISSIONS: &str =
    include_str!("../templates/db/sqlite/create_role_permissions.sql");
pub const SQLITE_CREATE_SIGNING_KEYS: &str =
    include_str!("../templates/db/sqlite/create_signing_keys.sql");

// -- PostgreSQL
pub const PG_CREATE_USERS: &str = include_str!("../templates/db/postgres/create_users.sql");
pub const PG_CREATE_ACCOUNTS: &str = include_str!("../templates/db/postgres/create_accounts.sql");
pub const PG_CREATE_SESSIONS: &str = include_str!("../templates/db/postgres/create_sessions.sql");
pub const PG_CREATE_REFRESH_TOKENS: &str =
    include_str!("../templates/db/postgres/create_refresh_tokens.sql");
pub const PG_CREATE_PASSWORD_RESET_TOKENS: &str =
    include_str!("../templates/db/postgres/create_password_reset_tokens.sql");
pub const PG_CREATE_EMAIL_VERIFICATION_TOKENS: &str =
    include_str!("../templates/db/postgres/create_email_verification_tokens.sql");

pub const PG_CREATE_ROLES: &str = include_str!("../templates/db/postgres/create_roles.sql");
pub const PG_CREATE_USER_ROLES: &str =
    include_str!("../templates/db/postgres/create_user_roles.sql");
pub const PG_CREATE_PERMISSIONS: &str =
    include_str!("../templates/db/postgres/create_permissions.sql");
pub const PG_CREATE_ROLE_PERMISSIONS: &str =
    include_str!("../templates/db/postgres/create_role_permissions.sql");
pub const PG_CREATE_SIGNING_KEYS: &str =
    include_str!("../templates/db/postgres/create_signing_keys.sql");

pub fn jwt_tables(db: DatabaseDriver) -> Vec<(&'static str, &'static str)> {
    let mut tables = db_tables(db);
    match db {
        DatabaseDriver::SQLite => {
            tables.push((SQLITE_CREATE_REFRESH_TOKENS, "anzar_create_refresh_tokens"))
        }
        DatabaseDriver::PostgreSQL => {
            tables.push((PG_CREATE_REFRESH_TOKENS, "anzar_create_refresh_tokens"))
        }
        DatabaseDriver::MongoDB => unreachable!(),
    }

    tables
}
pub fn session_tables(db: DatabaseDriver) -> Vec<(&'static str, &'static str)> {
    let mut tables = db_tables(db);
    match db {
        DatabaseDriver::SQLite => tables.push((SQLITE_CREATE_SESSIONS, "anzar_create_sessions")),
        DatabaseDriver::PostgreSQL => tables.push((PG_CREATE_SESSIONS, "anzar_create_sessions")),
        DatabaseDriver::MongoDB => unreachable!(),
    }

    tables
}

pub fn db_tables(db: DatabaseDriver) -> Vec<(&'static str, &'static str)> {
    match db {
        DatabaseDriver::PostgreSQL => vec![
            (PG_CREATE_USERS, "anzar_create_users"),
            (PG_CREATE_ACCOUNTS, "anzar_create_accounts"),
            (
                PG_CREATE_PASSWORD_RESET_TOKENS,
                "anzar_create_password_reset_tokens",
            ),
            (
                PG_CREATE_EMAIL_VERIFICATION_TOKENS,
                "anzar_create_email_verification_tokens",
            ),
            (PG_CREATE_ROLES, "anzar_create_roles"),
            (PG_CREATE_USER_ROLES, "anzar_create_user_roles"),
            (PG_CREATE_PERMISSIONS, "anzar_create_permissions"),
            (PG_CREATE_ROLE_PERMISSIONS, "anzar_create_role_permissions"),
            (PG_CREATE_SIGNING_KEYS, "anzar_create_signing_keys"),
        ],
        DatabaseDriver::SQLite => vec![
            (SQLITE_CREATE_USERS, "anzar_create_users"),
            (SQLITE_CREATE_ACCOUNTS, "anzar_create_accounts"),
            (
                SQLITE_CREATE_PASSWORD_RESET_TOKENS,
                "anzar_create_password_reset_tokens",
            ),
            (
                SQLITE_CREATE_EMAIL_VERIFICATION_TOKENS,
                "anzar_create_email_verification_tokens",
            ),
            (SQLITE_CREATE_ROLES, "anzar_create_roles"),
            (SQLITE_CREATE_USER_ROLES, "anzar_create_user_roles"),
            (SQLITE_CREATE_PERMISSIONS, "anzar_create_permissions"),
            (
                SQLITE_CREATE_ROLE_PERMISSIONS,
                "anzar_create_role_permissions",
            ),
            (SQLITE_CREATE_SIGNING_KEYS, "anzar_create_signing_keys"),
        ],
        DatabaseDriver::MongoDB => unreachable!(),
    }
}
