use dialoguer::Select;

use crate::theme::theme;
use shared::config::{AuthStrategy, JwtConfig, SessionConfig};

pub fn select_strategy() -> AuthStrategy {
    let strategies: Vec<AuthStrategy> = vec![
        AuthStrategy::Session(SessionConfig {
            ..Default::default()
        }),
        AuthStrategy::Jwt(JwtConfig {
            ..Default::default()
        }),
    ];

    let choice = Select::with_theme(&theme())
        .with_prompt("Select authentication strategy")
        .items(&strategies)
        .default(0)
        .interact()
        .unwrap();

    strategies[choice].clone()
}
