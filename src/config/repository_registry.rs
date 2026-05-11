use crate::error::Result;

use crate::infrastructure::{
    cache::{CacheAdapters, in_memory::InMemoryAdapter, memcache::MemCache, redis::Redis},
    database::{DatabaseAdapters, mongodb::MongoDB, postgres::PostgreSQL, sqlite::SQLite},
};

use crate::domain::repositories::{
    AccountRepository, EmailVerificationTokenRepository, JWTRepository,
    PasswordResetTokenRepository, PermissionRepository, RolePermissionRepository, RoleRepository,
    SessionRepository, UserRepository, UserRoleRepository,
};

use super::Database;
use super::boot::cache::CacheDriver;
use super::boot::database::DatabaseDriver;
// use crate::services::transaction::repository::TransactionRepository;

#[derive(Clone)]
pub struct RepositoryRegistry {
    pub(crate) user_repository: UserRepository,
    pub(crate) account_repository: AccountRepository,
    pub(crate) jwt_repository: JWTRepository,
    pub(crate) session_repository: SessionRepository,
    pub(crate) password_reset_token_repository: PasswordResetTokenRepository,
    pub(crate) email_verification_token_repository: EmailVerificationTokenRepository,
    pub(crate) role_repository: RoleRepository,
    pub(crate) user_role_repository: UserRoleRepository,
    pub(crate) permission_repository: PermissionRepository,
    pub(crate) role_permission_repository: RolePermissionRepository,
    // pub(crate) transaction_repository: TransactionRepository,
}

impl RepositoryRegistry {
    pub fn new(database_adapters: DatabaseAdapters, cache_adapters: CacheAdapters) -> Self {
        Self {
            user_repository: UserRepository::new(
                database_adapters.user_adapter,
                cache_adapters.cache_adapter,
            ),
            account_repository: AccountRepository::new(database_adapters.account_adapter),
            jwt_repository: JWTRepository::new(database_adapters.jwt_adapter),
            session_repository: SessionRepository::new(database_adapters.session_adapter),
            password_reset_token_repository: PasswordResetTokenRepository::new(
                database_adapters.reset_token_adapter,
            ),
            email_verification_token_repository: EmailVerificationTokenRepository::new(
                database_adapters.email_verification_token,
            ),
            role_repository: RoleRepository::new(database_adapters.role_adapter),
            user_role_repository: UserRoleRepository::new(database_adapters.user_role_adapter),

            permission_repository: PermissionRepository::new(database_adapters.permission_adapter),
            role_permission_repository: RolePermissionRepository::new(
                database_adapters.role_permission_adapter,
            ),
            // transaction_repository: TransactionRepository::new(adapters.transaction_adapter),
        }
    }
    pub async fn from_database(database: &Database) -> Result<Self> {
        let cache_adapter = match database.cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&database.cache.url).await?;
                CacheAdapters::memcached(client)
            }
            CacheDriver::Redis => {
                let connection = Redis::start(&database.cache.url).await?;
                CacheAdapters::redis(connection)
            }
            CacheDriver::InMemory => {
                let store = InMemoryAdapter::default();
                CacheAdapters::in_memory(store)
            }
        };

        let database_adapter = match database.driver {
            DatabaseDriver::SQLite => {
                let sqlite = SQLite::start(&database.connection_string).await?;
                sqlite.run_migrations().await?;

                DatabaseAdapters::sqlite(&sqlite.pool)
            }
            DatabaseDriver::MongoDB => {
                let client = MongoDB::start(&database.connection_string).await?;
                let db_name = database.name().unwrap_or_default();
                DatabaseAdapters::mongodb(&client, db_name)
            }
            DatabaseDriver::PostgreSQL => {
                let postgresql = PostgreSQL::start(&database.connection_string).await?;
                postgresql.run_migrations().await?;

                DatabaseAdapters::postgres(&postgresql.pool)
            }
        };

        Ok(Self::new(database_adapter, cache_adapter))
    }
}
