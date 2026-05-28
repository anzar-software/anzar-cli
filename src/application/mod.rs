mod account;
mod email;
mod password_reset;
mod permission;
mod refresh_token;
mod role;
mod role_permission;
mod session;
mod signing_keys;
mod user;
mod user_role;

pub mod model {
    pub use crate::domain::model::*;
}

pub mod traits {
    pub use crate::application::account::traits::AccountServiceTrait;
    pub use crate::application::email::traits::EmailVerificationTokenServiceTrait;
    pub use crate::application::password_reset::traits::PasswordResetTokenServiceTrait;
    pub use crate::application::permission::traits::PermissionServiceTrait;
    pub use crate::application::refresh_token::traits::JwtServiceTrait;
    pub use crate::application::role::traits::RoleServiceTrait;
    pub use crate::application::role_permission::traits::RolePermissionServiceTrait;
    pub use crate::application::session::traits::SessionServiceTrait;
    pub use crate::application::signing_keys::traits::SigningKeysServiceTrait;
    pub use crate::application::user::traits::UserServiceTrait;
    pub use crate::application::user_role::traits::UserRoleServiceTrait;
}
