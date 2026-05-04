use crate::{shared::configuration::AuthStrategy, theme::theme};
use dialoguer::Select;

pub fn select_strategy() -> AuthStrategy {
    let strategies: Vec<AuthStrategy> = vec![
        AuthStrategy::Session(crate::shared::configuration::SessionConfig {
            ..Default::default()
        }),
        AuthStrategy::Jwt(crate::shared::configuration::JwtConfig {
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
