use crate::config::{AnzarConfiguration, PasswordConfig};
use crate::error::Result;
use crate::scopes::auth::service::AuthService;
use crate::scopes::email::model::EmailVerificationToken;
use crate::scopes::email::service::EmailVerificationTokenServiceTrait;

use crate::scopes::user::models::CreateUserOutcome;
use crate::services::account::model::{Account, AccountStatus};
use crate::services::lockout::service::LoginAttemptTracker;
use crate::utils::{CustomPasswordHasher, HmacSigner, Password, SecureToken, TokenHasher};

use super::User;
use crate::scopes::auth::{RegisterRequest, support};
use crate::scopes::user::support as user_support;

pub trait UserServiceTrait {
    fn authenticate_user(
        &self,
        email: &str,
        password: &str,
        device_cookie: &HmacSigner,
        session: &actix_session::Session,
        configuration: &AnzarConfiguration,
    ) -> impl Future<Output = Result<(User, AccountStatus, u8)>>;
    fn register_failed_attempt(
        &self,
        user: &str,
        device_cookie: Option<&str>,
        pass_config: &PasswordConfig,
    ) -> impl Future<Output = Result<u8>>;
    fn create_user(
        &self,
        req: RegisterRequest,
        configuration: &AnzarConfiguration,
    ) -> impl Future<Output = Result<CreateUserOutcome>>;
    fn create_verification_email(
        &self,
        user: &User,
        expiray: i64,
    ) -> impl Future<Output = Result<String>>;
    fn find_user_by_email(&self, email: &str) -> impl Future<Output = Result<User>>;
    fn find_user(&self, id: &str) -> impl Future<Output = Result<User>>;
    fn validate_account(&self, id: &str) -> impl Future<Output = Result<User>>;
}
impl UserServiceTrait for AuthService {
    #[tracing::instrument(
        name = "auth.authenticate_user",
        skip(self, email, password, hmac_signer, session, configuration),
        fields(user.id = tracing::field::Empty)
    )]
    async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
        hmac_signer: &HmacSigner,
        session: &actix_session::Session,
        configuration: &AnzarConfiguration,
    ) -> Result<(User, AccountStatus, u8)> {
        // 1. ALWAYS verify password (constant-time even with fake hash)
        let (target_user, target_hash) = user_support::resolve_user_with_password(
            &self.user_repository,
            &self.account_repository,
            email,
            configuration,
        )
        .await?;
        let password_valid = Password::verify(password, &target_hash)?;
        let user_id = target_user.id()?;

        // 2. Fetch device cookie
        let raw = session.get::<String>(support::DEVICE_COOKIE).ok().flatten();
        let device_cookie = raw.as_deref();

        // 3.
        let tracker = LoginAttemptTracker::new(hmac_signer);
        let identity = tracker.resolve_identity(device_cookie, email);
        let lockout_key = tracker.resolve_lockout_key(device_cookie, email);

        // FIXME
        // 4. Use cache_service instead of user_service (more readable)
        if self.user_repository.is_locked(&lockout_key).await {
            let _ = self.user_repository.reset_attempts(&identity).await;

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
                let _ = self.user_repository.clear_key(&identity).await;
                (AccountStatus::Active, 0)
            }
            _ => {
                let attempts = self
                    .register_failed_attempt(&identity, device_cookie, &configuration.auth.password)
                    .await
                    .unwrap_or(1);

                tracing::Span::current().record(
                    "attempts.remaining",
                    configuration.auth.password.security.max_failed_attempts - attempts,
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
        pass_config: &PasswordConfig,
    ) -> Result<u8> {
        let attempts = self.user_repository.increment(identity).await;

        // max_failed_attempts of authentication within for this specific cookie
        let max_failed_attempts = match device_cookie {
            Some(_) => pass_config.security.max_failed_attempts * 2,
            None => pass_config.security.max_failed_attempts,
        };
        if attempts >= max_failed_attempts {
            self.user_repository
                .put_cookie_in_lockout(identity, pass_config.security.lockout_duration as u64)
                .await?;
            tracing::warn!("Authentication Failed — update lockout countdown");
        }

        Ok(attempts)
    }

    #[tracing::instrument(
        name = "auth.create_user",
        skip(self, req, configuration),
        fields(user.id = tracing::field::Empty)
    )]
    async fn create_user(
        &self,
        req: RegisterRequest,
        configuration: &AnzarConfiguration,
    ) -> Result<CreateUserOutcome> {
        // let mut session = self.transaction_repository.start_transactions().await?;

        // 1. Find if user already exist
        let (_, user_exist) =
            user_support::resolve_user(&self.user_repository, &req.email, configuration).await?;

        // 2. Hash user password
        let password = Password::hash(&req.password)?;

        if user_exist {
            tracing::warn!(
                email = %req.email,
                error.code = "ConflictReason::AlreadyExists",
                "Registeration failed — user already exists"
            );
            return Ok(CreateUserOutcome::AlreadyExists);
        }

        let mut user = User::new()
            .with_username(&req.username)
            .with_email(&req.email);

        // let user_id: String = self.user_service.insert(&user, Some(&mut session)).await?;
        let user_id: String = self.user_repository.insert(&user).await?;
        user.with_id(&user_id);

        let account = Account::user(&user_id).with_password(&password);
        self.account_repository.insert(account).await?;

        // self.transaction_repository
        //     .commit_transaction(session)
        //     .await?;

        tracing::info!(user.id = %user_id, "User created successfully");
        Ok(CreateUserOutcome::Created(user))
    }

    #[tracing::instrument(
        name = "auth.create_verification_email",
        skip(self, user, expiry),
        fields(user.id = user.id, user.email = user.email)
    )]
    async fn create_verification_email(&self, user: &User, expiry: i64) -> Result<String> {
        let user_id = user.id()?;

        let token = SecureToken::with_size32().generate();
        let hashed_token = SecureToken::hash(&token);

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
        self.user_repository.find_by_email(email).await
    }

    #[tracing::instrument(
        name = "auth.find_user",
        skip(self),
        fields(user.id = id)
    )]
    async fn find_user(&self, id: &str) -> Result<User> {
        self.user_repository.find(id).await
    }

    #[tracing::instrument(
        name = "auth.validate_account",
        skip(self),
        fields(user.id = id)
    )]
    async fn validate_account(&self, id: &str) -> Result<User> {
        self.user_repository.validate_account(id).await
    }
}
