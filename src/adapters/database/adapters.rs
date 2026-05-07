use std::sync::Arc;

use sqlx::{Pool, Postgres, Sqlite};

use crate::adapters::database::{
    DatabaseAdapter, mongodb::MongodbAdapter, postgres::PostgreSQLAdapter, sqlite::SQLiteAdapter,
};
use crate::scopes::role::model::Role;
use crate::scopes::{
    auth::model::PasswordResetToken, email::model::EmailVerificationToken, user::User,
    user::UserRole,
};
use crate::services::{
    account::model::Account,
    jwt::RefreshToken,
    session::model::Session, // transaction::adapter::MongodbTransaction,
};

const USER: &str = "users";
const ACCOUNT: &str = "accounts";
const REFRESH_TOKEN: &str = "refresh_tokens";
const PASSWORD_RESET_TOKEN: &str = "password_reset_tokens";
const EMAIL_VERIFICATION_TOKEN: &str = "email_verification_tokens";
const SESSION: &str = "sessions";

const ROLE: &str = "roles";
const USER_ROLE: &str = "user_roles";

pub struct DatabaseAdapters {
    pub user_adapter: Arc<dyn DatabaseAdapter<User>>,
    pub account_adapter: Arc<dyn DatabaseAdapter<Account>>,
    pub jwt_adapter: Arc<dyn DatabaseAdapter<RefreshToken>>,
    pub session_adapter: Arc<dyn DatabaseAdapter<Session>>,
    pub reset_token_adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>,
    pub email_verification_token: Arc<dyn DatabaseAdapter<EmailVerificationToken>>,
    pub role_adapter: Arc<dyn DatabaseAdapter<Role>>,
    pub user_role_adapter: Arc<dyn DatabaseAdapter<UserRole>>,
    // pub transaction_adapter: MongodbTransaction,
}

impl DatabaseAdapters {
    pub fn mongodb(client: &mongodb::Client, conn_string: &str) -> Self {
        Self {
            user_adapter: Arc::new(MongodbAdapter::<User>::new(client, conn_string, USER)),
            account_adapter: Arc::new(MongodbAdapter::<Account>::new(client, conn_string, ACCOUNT)),
            jwt_adapter: Arc::new(MongodbAdapter::<RefreshToken>::new(
                client,
                conn_string,
                REFRESH_TOKEN,
            )),
            session_adapter: Arc::new(MongodbAdapter::<Session>::new(client, conn_string, SESSION)),
            reset_token_adapter: Arc::new(MongodbAdapter::<PasswordResetToken>::new(
                client,
                conn_string,
                PASSWORD_RESET_TOKEN,
            )),
            email_verification_token: Arc::new(MongodbAdapter::<EmailVerificationToken>::new(
                client,
                conn_string,
                EMAIL_VERIFICATION_TOKEN,
            )),
            role_adapter: Arc::new(MongodbAdapter::<Role>::new(client, conn_string, ROLE)),
            user_role_adapter: Arc::new(MongodbAdapter::<UserRole>::new(
                client,
                conn_string,
                USER_ROLE,
            )),
            // transaction_adapter: MongodbTransaction::new(client),
        }
    }

    pub fn sqlite(pool: &Pool<Sqlite>) -> Self {
        Self {
            user_adapter: Arc::new(SQLiteAdapter::<User>::new(pool, USER)),
            account_adapter: Arc::new(SQLiteAdapter::<Account>::new(pool, ACCOUNT)),
            jwt_adapter: Arc::new(SQLiteAdapter::<RefreshToken>::new(pool, REFRESH_TOKEN)),
            session_adapter: Arc::new(SQLiteAdapter::<Session>::new(pool, SESSION)),
            reset_token_adapter: Arc::new(SQLiteAdapter::<PasswordResetToken>::new(
                pool,
                PASSWORD_RESET_TOKEN,
            )),
            email_verification_token: Arc::new(SQLiteAdapter::<EmailVerificationToken>::new(
                pool,
                EMAIL_VERIFICATION_TOKEN,
            )),
            role_adapter: Arc::new(SQLiteAdapter::<Role>::new(pool, ROLE)),
            user_role_adapter: Arc::new(SQLiteAdapter::<UserRole>::new(pool, USER_ROLE)),
            // transaction_adapter: todo!(),
        }
    }

    pub fn postgres(pool: &Pool<Postgres>) -> Self {
        Self {
            user_adapter: Arc::new(PostgreSQLAdapter::<User>::new(pool, USER)),
            account_adapter: Arc::new(PostgreSQLAdapter::<Account>::new(pool, ACCOUNT)),
            jwt_adapter: Arc::new(PostgreSQLAdapter::<RefreshToken>::new(pool, REFRESH_TOKEN)),
            session_adapter: Arc::new(PostgreSQLAdapter::<Session>::new(pool, SESSION)),
            reset_token_adapter: Arc::new(PostgreSQLAdapter::<PasswordResetToken>::new(
                pool,
                PASSWORD_RESET_TOKEN,
            )),
            email_verification_token: Arc::new(PostgreSQLAdapter::<EmailVerificationToken>::new(
                pool,
                EMAIL_VERIFICATION_TOKEN,
            )),
            role_adapter: Arc::new(PostgreSQLAdapter::<Role>::new(pool, ROLE)),
            user_role_adapter: Arc::new(PostgreSQLAdapter::<UserRole>::new(pool, USER_ROLE)),
            // transaction_adapter: todo!(),
        }
    }
}
