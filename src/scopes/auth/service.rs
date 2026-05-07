use crate::adapters::cache::in_memory::InMemoryAdapter;
use crate::adapters::cache::redis::Redis;
use crate::adapters::cache::{CacheAdapters, memcache::MemCache};
use crate::adapters::database::postgres::PostgreSQL;
use crate::adapters::database::{DatabaseAdapters, mongodb::MongoDB, sqlite::SQLite};

use crate::config::database::cache_driver::CacheDriver;
use crate::config::{Database, database::driver::DatabaseDriver};
use crate::error::Result;

use crate::scopes::auth::PasswordResetTokenRepository;
use crate::scopes::email::EmailVerificationTokenRepository;
use crate::scopes::role::RoleRepository;
use crate::scopes::user::{UserRepository, UserRoleRepository};
use crate::services::account::AccountRepository;
use crate::services::jwt::JWTRepository;
use crate::services::session::SessionRepository;
// use crate::services::transaction::repository::TransactionRepository;

#[derive(Clone)]
pub struct AuthService {
    pub(crate) user_repository: UserRepository,
    pub(crate) account_repository: AccountRepository,
    pub(crate) jwt_repository: JWTRepository,
    pub(crate) session_repository: SessionRepository,
    pub(crate) password_reset_token_repository: PasswordResetTokenRepository,
    pub(crate) email_verification_token_repository: EmailVerificationTokenRepository,
    pub(crate) role_repository: RoleRepository,
    pub(crate) user_role_repository: UserRoleRepository,
    // pub(crate) transaction_repository: TransactionRepository,
}

impl AuthService {
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
                DatabaseAdapters::postgres(&postgresql.pool)
            }
        };

        Ok(Self::new(database_adapter, cache_adapter))
    }
}
