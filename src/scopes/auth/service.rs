use crate::adapters::cache::{CacheAdapters, memcache::MemCache};
use crate::adapters::database::{DatabaseAdapters, mongodb::MongoDB, sqlite::SQLite};

use crate::config::database::cache_driver::CacheDriver;
use crate::config::{Database, database::driver::DatabaseDriver};
use crate::error::Result;

use crate::scopes::auth::PasswordResetTokenRepository;
use crate::scopes::email::EmailVerificationTokenRepository;
use crate::scopes::user::UserRepository;
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
    // pub(crate) transaction_repository: TransactionRepository,
}

impl AuthService {
    pub fn new(
        database_adapters: DatabaseAdapters,
        driver: DatabaseDriver,
        cache_adapters: CacheAdapters,
    ) -> Self {
        Self {
            user_repository: UserRepository::new(
                database_adapters.user_adapter,
                driver,
                cache_adapters.cache_adapter,
            ),
            account_repository: AccountRepository::new(database_adapters.account_adapter, driver),
            jwt_repository: JWTRepository::new(database_adapters.jwt_adapter, driver),
            session_repository: SessionRepository::new(database_adapters.session_adapter, driver),
            password_reset_token_repository: PasswordResetTokenRepository::new(
                database_adapters.reset_token_adapter,
                driver,
            ),
            email_verification_token_repository: EmailVerificationTokenRepository::new(
                database_adapters.email_verification_token,
                driver,
            ),
            // transaction_repository: TransactionRepository::new(adapters.transaction_adapter),
        }
    }
    // TODO add from_cache()
    pub async fn from_database(database: &Database) -> Result<Self> {
        let cache_adapter = match database.cache.driver {
            CacheDriver::MemCached => {
                let client = MemCache::start(&database.cache.url).await?;
                CacheAdapters::memcached(client)
            }
            CacheDriver::Redis => todo!(),
        };

        let database_adapter = match database.driver {
            // DatabaseDriver::SQLite => Ok(Self::from_sqlite("/app/test.db".into()).await?),
            DatabaseDriver::SQLite => {
                let db = SQLite::start(&database.connection_string).await?;
                DatabaseAdapters::sqlite(&db)
            }
            DatabaseDriver::MongoDB => {
                let client = MongoDB::start(&database.connection_string).await?;
                let db_name = database.name().unwrap_or_default();
                DatabaseAdapters::mongodb(&client, db_name)
            }
            DatabaseDriver::PostgreSQL => todo!(),
        };

        Ok(Self::new(database_adapter, database.driver, cache_adapter))
    }
}
