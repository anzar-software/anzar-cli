use crate::config::AppState;
use crate::error::Result;

use crate::application::traits::UserRoleServiceTrait;
use crate::domain::model::{
    Account, AccountStatus, CreateUserOutcome, LoginRequest, RegisterRequest, User,
};

use super::support;
use super::tracker::LoginAttemptTracker;
use super::traits::UserServiceTrait;

impl UserServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.authenticate_user", skip(self, body, device_cookie),
        fields(
            attempts.remaining = tracing::field::Empty
        )
    )]
    async fn authenticate_user(
        &self,
        body: &LoginRequest,
        device_cookie: Option<&str>,
    ) -> Result<(User, Account, AccountStatus, u8)> {
        let user_repo = &self.repositories.user_repository;

        // 1. ALWAYS verify password (constant-time even with fake hash)
        let app_state = self;
        let (target_user, target_account) =
            support::resolve_user_with_password(&body.email, app_state).await?;
        let password_valid = self
            .crypto
            .password_hasher
            .verify(&body.password, &target_account.password)?;
        let user_id = target_user.id()?;

        // 3.
        let tracker = LoginAttemptTracker::new(&self.crypto);
        let identity = tracker.resolve_identity(device_cookie, &body.email);
        let lockout_key = tracker.resolve_lockout_key(device_cookie, &body.email);

        // FIXME
        // 4. Use cache_service instead of user_service (more readable)
        if user_repo.is_locked(&lockout_key).await {
            tracing::warn!(
                user.id = %user_id,
                error.code = "ForbiddenReason::AccountSuspended",
                "Login blocked — account is locked"
            );
            return Ok((
                target_user.clone(),
                target_account.clone(),
                AccountStatus::Suspended,
                0,
            ));
        }

        // 5.
        tracing::Span::current().record("user.id", user_id);
        let (account_status, attempts) = match password_valid {
            true => {
                let _ = user_repo.clear_key(&identity).await;
                (AccountStatus::Active, 0)
            }
            _ => {
                let attempts = self
                    .register_failed_attempt(&identity, device_cookie)
                    .await
                    .unwrap_or(1);

                let remaining = &self
                    .configuration
                    .auth
                    .password
                    .security
                    .max_failed_attempts
                    .saturating_sub(attempts);

                tracing::Span::current().record("attempts.remaining", remaining);
                (AccountStatus::InvalidCredentials, attempts)
            }
        };

        Ok((
            target_user.clone(),
            target_account.clone(),
            account_status,
            attempts,
        ))
    }

    async fn register_failed_attempt(
        &self,
        identity: &str,
        device_cookie: Option<&str>,
    ) -> Result<u8> {
        let user_repo = &self.repositories.user_repository;

        let pass_config = &self.configuration.auth.password;
        let expiry = pass_config.security.lockout_duration as u64;

        let attempts = user_repo.increment(identity, expiry).await;

        // max_failed_attempts of authentication within for this specific cookie
        let max_failed_attempts = match device_cookie {
            Some(_) => pass_config.security.max_failed_attempts * 2,
            None => pass_config.security.max_failed_attempts,
        };
        if attempts >= max_failed_attempts {
            user_repo.put_cookie_in_lockout(identity, expiry).await?;
            tracing::warn!("Authentication Failed — update lockout countdown");
        }

        Ok(attempts)
    }

    #[tracing::instrument(
        name = "auth.create_user",
        skip(self, body),
        fields(user.id = tracing::field::Empty)
    )]
    async fn create_user(&self, body: RegisterRequest) -> Result<CreateUserOutcome> {
        // let mut session = self.transaction_repository.start_transactions().await?;

        // 1. Find if user already exist
        let app_state = self;
        let (_, user_exist) = support::resolve_user(&body.email, app_state).await?;

        // 2. Hash user password
        let password = self.crypto.password_hasher.hash(&body.password)?;

        if user_exist {
            tracing::warn!(
                email = %body.email,
                error.code = "ConflictReason::AlreadyExists",
                "Registeration failed — user already exists"
            );
            return Ok(CreateUserOutcome::AlreadyExists);
        }

        let mut user = User::new()
            .with_username(&body.username)
            .with_email(&body.email);

        // let user_id: String = self.user_service.insert(&user, Some(&mut session)).await?;
        let user_id: String = self.repositories.user_repository.insert(&user).await?;
        user.with_id(&user_id);

        let account = Account::user(&user_id).with_password(&password);
        self.repositories.account_repository.insert(account).await?;

        self.insert_user_role(&user_id, "Admin").await?;
        // self.auth_service.user_role_repository.insert(role).await?;

        // self.transaction_repository
        //     .commit_transaction(session)
        //     .await?;

        tracing::info!(user.id = %user_id, "User created successfully");
        Ok(CreateUserOutcome::Created(user))
    }

    #[tracing::instrument(
        name = "auth.find_user_by_email",
        skip(self),
        fields(user.email = email)
    )]
    async fn find_user_by_email(&self, email: &str) -> Result<User> {
        self.repositories.user_repository.find_by_email(email).await
    }

    #[tracing::instrument(
        name = "auth.find_user",
        skip(self),
        fields(user.id = id)
    )]
    async fn find_user(&self, id: &str) -> Result<User> {
        self.repositories.user_repository.find(id).await
    }
}
