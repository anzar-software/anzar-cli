mod account;
mod auth;
mod email;
mod password_reset;
mod refresh_token;
mod role;
mod session;
mod user;
mod user_role;

mod serde;

mod ports;
pub use ports::cache;
pub use ports::database;
pub use ports::query;

pub mod model {
    pub use super::account::model::{Account, AccountStatus};
    pub use super::auth::claims::{Claims, TokenType};
    pub use super::auth::model::{LoginRequest, RegisterRequest, ResetPasswordRequest, TokenQuery};
    pub use super::email::model::EmailVerificationToken;
    pub use super::password_reset::model::{ExpiringLink, PasswordResetToken};
    pub use super::refresh_token::model::{IssuedTokens, RefreshToken};
    pub use super::role::model::Role;
    pub use super::session::model::Session;
    pub use super::user::model::{CreateUserOutcome, User};
    pub use super::user_role::model::UserRole;
}

pub mod repositories {
    pub use super::account::repository::AccountRepository;
    pub use super::email::repository::EmailVerificationTokenRepository;
    pub use super::password_reset::repository::PasswordResetTokenRepository;
    pub use super::refresh_token::repository::JWTRepository;
    pub use super::role::repository::RoleRepository;
    pub use super::session::repository::SessionRepository;
    pub use super::user::repository::UserRepository;
    pub use super::user_role::repository::UserRoleRepository;
}
