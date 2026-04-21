use crate::config::{AnzarConfiguration, PasswordConfig};
use crate::error::{CredentialField, Error, Result, ValidationError};
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
            return Ok((target_user.clone(), AccountStatus::Suspended, 0));
        }

        // 5.
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
        }

        Ok(attempts)
    }

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

        Ok(CreateUserOutcome::Created(user))
    }
    async fn create_verification_email(&self, user: &User, expiry: i64) -> Result<String> {
        let user_id = user.id.as_ref().ok_or_else(|| {
            Error::Validation(ValidationError::Malformed {
                field: CredentialField::ObjectId,
            })
        })?;

        let token = SecureToken::with_size32().generate();
        let hashed_token = SecureToken::hash(&token);

        let otp = EmailVerificationToken::default()
            .with_user_id(user_id)
            .with_token_hash(&hashed_token)
            .with_expiray(chrono::Duration::seconds(expiry));
        self.insert_email_verification_token(otp).await?;

        Ok(token)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<User> {
        self.user_repository.find_by_email(email).await
    }
    async fn find_user(&self, id: &str) -> Result<User> {
        self.user_repository.find(id).await
    }
    async fn validate_account(&self, id: &str) -> Result<User> {
        self.user_repository.validate_account(id).await
    }
}
