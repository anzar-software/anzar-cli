use crate::config::AppState;
use crate::error::Result;
use crate::scopes::user::User;
use crate::services::fake::service::FakeUserGenerator;

pub async fn resolve_user_with_password(
    email: &str,
    app_state: &AppState,
) -> Result<(User, String)> {
    // FIXME Single query with JOIN
    // Try to fetch real user
    let real = app_state
        .auth_service
        .user_repository
        .find_by_email(email)
        .await
        .ok();

    // Extract real password hash or use a fake one
    let (user, password) = match real {
        Some(user) => {
            match app_state
                .auth_service
                .account_repository
                .find(&user.clone().id.unwrap_or_default())
                .await
            {
                Ok(account) => (user, account.password),
                Err(_) => {
                    let fake_gen =
                        FakeUserGenerator::new(&app_state.configuration.security.secret_key);
                    (
                        fake_gen.generate_fake_user(email),
                        fake_gen.generate_fake_hash(&app_state.crypto.password_hasher),
                    )
                }
            }
        }
        None => {
            let fake_gen = FakeUserGenerator::new(&app_state.configuration.security.secret_key);
            (
                fake_gen.generate_fake_user(email),
                fake_gen.generate_fake_hash(&app_state.crypto.password_hasher),
            )
        }
    };

    Ok((user, password))
}

pub async fn resolve_user(email: &str, app_state: &AppState) -> Result<(User, bool)> {
    let real = app_state
        .auth_service
        .user_repository
        .find_by_email(email)
        .await
        .ok();

    match real {
        Some(user) => Ok((user, true)),
        None => {
            let fake_gen = FakeUserGenerator::new(&app_state.configuration.security.secret_key);
            Ok((fake_gen.generate_fake_user(email), false))
        }
    }
}
