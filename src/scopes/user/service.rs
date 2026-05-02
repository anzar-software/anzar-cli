use crate::config::AppState;
use crate::error::Result;

use crate::scopes::auth::{LoginRequest, RegisterRequest, support};
use crate::scopes::email::{
    model::EmailVerificationToken, service::EmailVerificationTokenServiceTrait,
};
use crate::scopes::user::models::CreateUserOutcome;

use crate::services::account::model::{Account, AccountStatus};
use crate::services::lockout::service::LoginAttemptTracker;

use super::User;
use super::support as user_support;

pub trait UserServiceTrait {
    fn authenticate_user(
        &self,
        body: &LoginRequest,
        session: &actix_session::Session,
    ) -> impl Future<Output = Result<(User, AccountStatus, u8)>>;
    fn register_failed_attempt(
        &self,
        user: &str,
        device_cookie: Option<&str>,
    ) -> impl Future<Output = Result<u8>>;
    fn create_user(&self, body: RegisterRequest)
    -> impl Future<Output = Result<CreateUserOutcome>>;
    fn create_verification_email(&self, user: &User) -> impl Future<Output = Result<String>>;
    fn find_user_by_email(&self, email: &str) -> impl Future<Output = Result<User>>;
    fn find_user(&self, id: &str) -> impl Future<Output = Result<User>>;
    fn validate_account(&self, id: &str) -> impl Future<Output = Result<User>>;
}
impl UserServiceTrait for AppState {
    #[tracing::instrument(
        name = "auth.authenticate_user",
        skip(self, body, session),
        fields(user.id = tracing::field::Empty)
    )]
    async fn authenticate_user(
        &self,
        body: &LoginRequest,
        session: &actix_session::Session,
    ) -> Result<(User, AccountStatus, u8)> {
        let user_repo = &self.auth_service.user_repository;

        // 1. ALWAYS verify password (constant-time even with fake hash)
        let app_state = self;
        let (target_user, target_hash) =
            user_support::resolve_user_with_password(&body.email, app_state).await?;
        let password_valid = self
            .crypto
            .password_hasher
            .verify(&body.password, &target_hash)?;
        let user_id = target_user.id()?;

        // 2. Fetch device cookie
        let raw = session.get::<String>(support::DEVICE_COOKIE).ok().flatten();
        let device_cookie = raw.as_deref();

        // 3.
        let tracker = LoginAttemptTracker::new(&self.crypto);
        let identity = tracker.resolve_identity(device_cookie, &body.email);
        let lockout_key = tracker.resolve_lockout_key(device_cookie, &body.email);

        // FIXME
        // 4. Use cache_service instead of user_service (more readable)
        if user_repo.is_locked(&lockout_key).await {
            let _ = user_repo.reset_attempts(&identity).await;

            tracing::warn!(
                user.id = %user_id,
                error.code = "ForbiddenReason::AccountSuspended",
                "Login blocked — account is locked"
            );
            return Ok((target_user.clone(), AccountStatus::Suspended, 0));
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

                let password_config = &self.configuration.auth.password;
                tracing::Span::current().record(
                    "attempts.remaining",
                    password_config.security.max_failed_attempts - attempts,
                );
                (AccountStatus::InvalidCredentials, attempts)
            }
        };

        Ok((target_user.clone(), account_status, attempts))
    }

    async fn register_failed_attempt(
        &self,
        identity: &str,
        device_cookie: Option<&str>,
    ) -> Result<u8> {
        let user_repo = &self.auth_service.user_repository;
        let attempts = user_repo.increment(identity).await;

        let pass_config = &self.configuration.auth.password;
        // max_failed_attempts of authentication within for this specific cookie
        let max_failed_attempts = match device_cookie {
            Some(_) => pass_config.security.max_failed_attempts * 2,
            None => pass_config.security.max_failed_attempts,
        };
        if attempts >= max_failed_attempts {
            user_repo
                .put_cookie_in_lockout(identity, pass_config.security.lockout_duration as u64)
                .await?;
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
        let (_, user_exist) = user_support::resolve_user(&body.email, app_state).await?;

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
        let user_id: String = self.auth_service.user_repository.insert(&user).await?;
        user.with_id(&user_id);

        let account = Account::user(&user_id).with_password(&password);
        self.auth_service.account_repository.insert(account).await?;

        // self.transaction_repository
        //     .commit_transaction(session)
        //     .await?;

        tracing::info!(user.id = %user_id, "User created successfully");
        Ok(CreateUserOutcome::Created(user))
    }

    #[tracing::instrument(
        name = "auth.create_verification_email",
        skip(self, user),
        fields(user.id = user.id, user.email = user.email)
    )]
    async fn create_verification_email(&self, user: &User) -> Result<String> {
        let user_id = user.id()?;

        let token = self.crypto.token.generate()?;
        let hashed_token = self.crypto.token.hash(&token);

        let expiry = self.configuration.auth.email.verification.token_expires_in;
        let otp = EmailVerificationToken::default()
            .with_user_id(user_id)
            .with_token_hash(&hashed_token)
            .with_expiray(chrono::Duration::seconds(expiry));
        self.insert_email_verification_token(otp).await?;

        Ok(token)
    }

    #[tracing::instrument(
        name = "auth.find_user_by_email",
        skip(self),
        fields(user.email = email)
    )]
    async fn find_user_by_email(&self, email: &str) -> Result<User> {
        self.auth_service.user_repository.find_by_email(email).await
    }

    #[tracing::instrument(
        name = "auth.find_user",
        skip(self),
        fields(user.id = id)
    )]
    async fn find_user(&self, id: &str) -> Result<User> {
        self.auth_service.user_repository.find(id).await
    }

    #[tracing::instrument(
        name = "auth.validate_account",
        skip(self),
        fields(user.id = id)
    )]
    async fn validate_account(&self, id: &str) -> Result<User> {
        self.auth_service.user_repository.validate_account(id).await
    }
}
