use std::sync::Arc;

use sqlx::{Pool, Postgres, Sqlite};

use crate::{
    adapters::database::{
        DatabaseAdapter, mongodb::MongodbAdapter, postgres::PostgreSQLAdapter,
        sqlite::SQLiteAdapter,
    },
    scopes::{auth::model::PasswordResetToken, email::model::EmailVerificationToken, user::User},
    services::{
        account::model::Account,
        jwt::RefreshToken,
        session::model::Session,
        // transaction::adapter::MongodbTransaction,
    },
};

const USER: &str = "users";
const ACCOUNT: &str = "accounts";
const REFRESH_TOKEN: &str = "refresh_tokens";
const PASSWORD_RESET_TOKEN: &str = "password_reset_tokens";
const EMAIL_VERIFICATION_TOKEN: &str = "email_verification_tokens";
const SESSION: &str = "sessions";

pub struct DatabaseAdapters {
    pub user_adapter: Arc<dyn DatabaseAdapter<User>>,
    pub account_adapter: Arc<dyn DatabaseAdapter<Account>>,
    pub jwt_adapter: Arc<dyn DatabaseAdapter<RefreshToken>>,
    pub session_adapter: Arc<dyn DatabaseAdapter<Session>>,
    pub reset_token_adapter: Arc<dyn DatabaseAdapter<PasswordResetToken>>,
    pub email_verification_token: Arc<dyn DatabaseAdapter<EmailVerificationToken>>,
    // pub transaction_adapter: MongodbTransaction,
}

impl DatabaseAdapters {
    pub fn mongodb(client: &mongodb::Client, cnn_string: &str) -> Self {
        Self {
            user_adapter: Arc::new(MongodbAdapter::<User>::new(client, cnn_string, USER)),
            account_adapter: Arc::new(MongodbAdapter::<Account>::new(client, cnn_string, ACCOUNT)),
            jwt_adapter: Arc::new(MongodbAdapter::<RefreshToken>::new(
                client,
                cnn_string,
                REFRESH_TOKEN,
            )),
            session_adapter: Arc::new(MongodbAdapter::<Session>::new(client, cnn_string, SESSION)),
            reset_token_adapter: Arc::new(MongodbAdapter::<PasswordResetToken>::new(
                client,
                cnn_string,
                PASSWORD_RESET_TOKEN,
            )),
            email_verification_token: Arc::new(MongodbAdapter::<EmailVerificationToken>::new(
                client,
                cnn_string,
                EMAIL_VERIFICATION_TOKEN,
            )),
            // transaction_adapter: MongodbTransaction::new(client),
        }
    }

    pub fn sqlite(db: &Pool<Sqlite>) -> Self {
        Self {
            user_adapter: Arc::new(SQLiteAdapter::<User>::new(db, USER)),
            account_adapter: Arc::new(SQLiteAdapter::<Account>::new(db, ACCOUNT)),
            jwt_adapter: Arc::new(SQLiteAdapter::<RefreshToken>::new(db, REFRESH_TOKEN)),
            session_adapter: Arc::new(SQLiteAdapter::<Session>::new(db, SESSION)),
            reset_token_adapter: Arc::new(SQLiteAdapter::<PasswordResetToken>::new(
                db,
                PASSWORD_RESET_TOKEN,
            )),
            email_verification_token: Arc::new(SQLiteAdapter::<EmailVerificationToken>::new(
                db,
                EMAIL_VERIFICATION_TOKEN,
            )),
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
            // transaction_adapter: todo!(),
        }
    }
}
