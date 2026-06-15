use crate::error::Result;

use crate::domain::model::{Account, User};
use crate::intern::auth::AuthService;

use super::fake::FakeUserGenerator;

pub async fn resolve_user_with_password(
    email: &str,
    app_state: &AuthService,
) -> Result<(User, Account)> {
    // FIXME Single query with JOIN
    // Try to fetch real user
    let real = app_state.user_repository.find_by_email(email).await.ok();

    // Extract real password hash or use a fake one
    let (user, account) = match real {
        Some(user) => {
            match app_state
                .account_repository
                .find(&user.clone().id.unwrap_or_default())
                .await
            {
                Ok(account) => (user, account),
                Err(_) => {
                    let fake_gen =
                        FakeUserGenerator::new(&app_state.configuration.security.secret_key);
                    (
                        fake_gen.generate_fake_user(email)?,
                        fake_gen.generate_fake_account(&app_state.crypto.password_hasher),
                    )
                }
            }
        }
        None => {
            let fake_gen = FakeUserGenerator::new(&app_state.configuration.security.secret_key);
            (
                fake_gen.generate_fake_user(email)?,
                fake_gen.generate_fake_account(&app_state.crypto.password_hasher),
            )
        }
    };

    Ok((user, account))
}

pub async fn resolve_user(email: &str, app_state: &AuthService) -> Result<(User, bool)> {
    let real = app_state.user_repository.find_by_email(email).await.ok();

    match real {
        Some(user) => Ok((user, true)),
        None => {
            let fake_gen = FakeUserGenerator::new(&app_state.configuration.security.secret_key);
            Ok((fake_gen.generate_fake_user(email)?, false))
        }
    }
}
