use crate::error::Result;
use crate::scopes::auth::service::AuthService;
use crate::scopes::user::{User, UserRepository};
use crate::{config::AnzarConfiguration, services::fake::service::FakeUserGenerator};

pub async fn resolve_user_with_password(
    auth_service: &AuthService,
    email: &str,
    configuration: &AnzarConfiguration,
) -> Result<(User, String)> {
    // FIXME Single query with JOIN
    // Try to fetch real user
    let real = auth_service.user_repository.find_by_email(email).await.ok();

    // Extract real password hash or use a fake one
    let (user, password) = match real {
        Some(user) => {
            match auth_service
                .account_repository
                .find(&user.clone().id.unwrap_or_default())
                .await
            {
                Ok(account) => (user, account.password),
                Err(_) => {
                    let fake_gen = FakeUserGenerator::new(&configuration.security.secret_key);
                    (
                        fake_gen.generate_fake_user(email),
                        fake_gen.generate_fake_hash(),
                    )
                }
            }
        }
        None => {
            let fake_gen = FakeUserGenerator::new(&configuration.security.secret_key);
            (
                fake_gen.generate_fake_user(email),
                fake_gen.generate_fake_hash(),
            )
        }
    };

    Ok((user, password))
}

pub async fn resolve_user(
    user_repository: &UserRepository,
    email: &str,
    configuration: &AnzarConfiguration,
) -> Result<(User, bool)> {
    let real = user_repository.find_by_email(email).await.ok();

    match real {
        Some(user) => Ok((user, true)),
        None => {
            let fake_gen = FakeUserGenerator::new(&configuration.security.secret_key);
            Ok((fake_gen.generate_fake_user(email), false))
        }
    }
}
