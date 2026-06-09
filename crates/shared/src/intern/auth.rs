use crate::config::AnzarConfiguration;
use crate::infrastructure::{cache::CacheAdapters, database::DatabaseAdapters};

use crate::domain::repositories::{
    AccountRepository, EmailVerificationTokenRepository, PasswordResetTokenRepository,
    UserRepository,
};

use crate::utils::crypto::Crypto;

#[derive(Clone)]
pub struct AuthService {
    pub(crate) user_repository: UserRepository,
    pub(crate) account_repository: AccountRepository,
    pub(crate) password_reset_token_repository: PasswordResetTokenRepository,
    pub(crate) email_verification_token_repository: EmailVerificationTokenRepository,
    pub(crate) crypto: Crypto,
    pub(crate) configuration: AnzarConfiguration,
}

pub struct AuthContext<'a> {
    pub service: &'a AuthService,
    pub crypto: &'a Crypto,
    pub configuration: &'a AnzarConfiguration,
}

impl AuthService {
    pub fn new(
        database_adapters: &DatabaseAdapters,
        cache_adapters: &CacheAdapters,
        crypto: &Crypto,
        configuration: &AnzarConfiguration,
    ) -> Self {
        Self {
            user_repository: UserRepository::new(
                database_adapters.user_adapter.clone(),
                cache_adapters.cache_adapter.clone(),
            ),
            account_repository: AccountRepository::new(database_adapters.account_adapter.clone()),
            password_reset_token_repository: PasswordResetTokenRepository::new(
                database_adapters.reset_token_adapter.clone(),
            ),
            email_verification_token_repository: EmailVerificationTokenRepository::new(
                database_adapters.email_verification_token.clone(),
            ),
            crypto: crypto.clone(),
            configuration: configuration.clone(),
        }
    }
}
