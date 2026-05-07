mod models;
mod repository;
mod scope;
mod support;
mod user_role;

// #[cfg(test)]
// mod tests;

pub mod service;

pub use models::{CreateUserOutcome, User};
pub use repository::UserRepository;
pub use scope::{__path_find_user, user_scope};
pub use user_role::{UserRoleRepository, model::UserRole, service::UserRoleServiceTrait};
