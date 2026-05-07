pub mod validate;

use super::database::{cache_driver::CacheDriver, driver::DatabaseDriver};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    config::validate::Validate,
    error::{Error, InternalError},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct AnzarConfiguration {
    pub app: App,           // Required
    pub database: Database, // Required
    #[serde(default)]
    pub server: Server, // [Optional] Uses Default
    #[serde(default)]
    pub auth: Authentication, // [Optional] Uses Default
    pub security: Security, // Required
}

impl AnzarConfiguration {
    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = vec![];

        if let Err(e) = self.auth.validate() {
            errors.extend(e);
        }
        if let Err(e) = self.security.validate() {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Internal(InternalError::InvalidConfig(errors)))
        }
    }
}

impl AnzarConfiguration {
    pub fn new(app_config: super::AppConfig) -> Self {
        Self {
            app: App {
                environment: "dev".into(),
                url: "localhost:3000".to_string(),
            },
            database: Database {
                driver: app_config.database.driver,
                connection_string: app_config.database.connection_string(),
                cache: Cache {
                    driver: app_config.cache.driver,
                    url: app_config.cache.url,
                },
            },
            server: Server::default(),
            auth: Authentication {
                strategy: AuthStrategy::Jwt(JwtConfig {
                    issuer: "localhost:3000".into(),
                    audience: "users".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            security: Security {
                secret_key: String::default(),
                headers: vec![],
            },
        }
    }
    pub fn with_appurl(mut self, url: &str) -> Self {
        self.app.url = url.to_string();
        self
    }
    pub fn with_secret(mut self, key: &str) -> Self {
        self.security.secret_key = key.to_string();
        self
    }
}

// =============================================================================
// API Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct App {
    pub environment: String,
    pub url: String,
}

// =============================================================================
// Database Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct Database {
    pub driver: DatabaseDriver,
    pub connection_string: String,
    pub cache: Cache,
}
impl Database {
    pub fn name(&self) -> Option<&str> {
        self.connection_string
            .rsplit('/')
            .next()
            .and_then(|s| s.split('?').next())
    }
}
// Cache
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct Cache {
    pub driver: CacheDriver,
    pub url: String,
}

// =============================================================================
// Server Configuration - Optional
// =============================================================================
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Server {
    pub https: HttpsConfig,
    pub cors: CorsConfig,
}
// HttpsConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct HttpsConfig {
    pub enabled: bool,
    pub port: u16,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}
impl Default for HttpsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 3000,
            cert_path: None,
            key_path: None,
        }
    }
}
// CorsConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: u64,
}
impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["localhost:3000".into()],
            allowed_methods: vec![
                "GET".into(),
                "POST".into(),
                "PUT".into(),
                "DELETE".into(),
                "OPTIONS".into(),
            ],
            allowed_headers: vec![
                "authorization".into(),
                "content-type".into(),
                "accept".into(),
                "accept-language".into(),
                "Content-Language".into(),
            ],
            allow_credentials: true,
            max_age: 3600,
        }
    }
}

// =============================================================================
// Authentication Configuration - Optional
// =============================================================================
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct Authentication {
    pub strategy: AuthStrategy,
    pub email: EmailConfig,
    pub password: PasswordConfig,
    pub rbac: RbacConfig,
}
impl Authentication {
    pub fn jwt(&self) -> Result<&JwtConfig, Error> {
        match &self.strategy {
            AuthStrategy::Jwt(config) => Ok(config),
            _ => Err(Error::Internal(InternalError::MissingConfiguration(
                "JWT strategy is required, but auth.strategy was not configured correctly".into(),
            ))),
        }
    }
    pub fn session(&self) -> Result<&SessionConfig, Error> {
        match &self.strategy {
            AuthStrategy::Session(config) => Ok(config),
            _ => Err(Error::Internal(InternalError::MissingConfiguration(
                "Session strategy is required, but auth.strategy was not configured correctly"
                    .into(),
            ))),
        }
    }
}
// AuthStrategy
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum AuthStrategy {
    Session(SessionConfig),
    Jwt(JwtConfig),
}
impl Default for AuthStrategy {
    fn default() -> Self {
        Self::Session(SessionConfig::default())
    }
}
// JwtConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct JwtConfig {
    pub algorithm: AlgorithmConfig,
    pub access_token_expires_in: i64,
    pub refresh_token_expires_in: i64,
    pub issuer: String,
    pub audience: String,
}
impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            algorithm: AlgorithmConfig::default(),
            access_token_expires_in: 900,
            refresh_token_expires_in: 604800,
            issuer: String::new(),
            audience: String::new(),
        }
    }
}
//
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum AlgorithmConfig {
    #[default]
    HS256,
    HS384,
    HS512,
    ES256,
    ES384,
    RS256,
    RS384,
    RS512,
    PS256,
    PS384,
    PS512,
    EdDSA,
}
impl From<AlgorithmConfig> for jsonwebtoken::Algorithm {
    fn from(value: AlgorithmConfig) -> Self {
        match value {
            AlgorithmConfig::HS256 => jsonwebtoken::Algorithm::HS256,
            AlgorithmConfig::HS384 => jsonwebtoken::Algorithm::HS384,
            AlgorithmConfig::HS512 => jsonwebtoken::Algorithm::HS512,
            AlgorithmConfig::ES256 => jsonwebtoken::Algorithm::ES256,
            AlgorithmConfig::ES384 => jsonwebtoken::Algorithm::ES384,
            AlgorithmConfig::RS256 => jsonwebtoken::Algorithm::RS256,
            AlgorithmConfig::RS384 => jsonwebtoken::Algorithm::RS384,
            AlgorithmConfig::PS256 => jsonwebtoken::Algorithm::PS256,
            AlgorithmConfig::PS384 => jsonwebtoken::Algorithm::PS384,
            AlgorithmConfig::PS512 => jsonwebtoken::Algorithm::PS512,
            AlgorithmConfig::RS512 => jsonwebtoken::Algorithm::RS512,
            AlgorithmConfig::EdDSA => jsonwebtoken::Algorithm::EdDSA,
        }
    }
}
// SessionConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct SessionConfig {
    pub name: String,
    pub max_age: u64,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSiteConfig,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum SameSiteConfig {
    #[default]
    Strict,
    Lax,
    None,
}
impl From<SameSiteConfig> for actix_web::cookie::SameSite {
    fn from(value: SameSiteConfig) -> Self {
        match value {
            SameSiteConfig::Strict => actix_web::cookie::SameSite::Strict,
            SameSiteConfig::Lax => actix_web::cookie::SameSite::Lax,
            SameSiteConfig::None => actix_web::cookie::SameSite::None,
        }
    }
}
impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: "id".into(),
            max_age: 3600,
            secure: true,
            http_only: true,
            same_site: SameSiteConfig::default(),
        }
    }
}

// EmailConfig
// ------------------------------------------------------------
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct EmailConfig {
    pub verification: EmailVerification,
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct EmailVerification {
    pub required: bool,
    pub token_expires_in: i64, // maybe option
    pub success_redirect: Option<String>,
    pub error_redirect: Option<String>,
}
impl Default for EmailVerification {
    fn default() -> Self {
        Self {
            required: false,
            token_expires_in: 1800,
            success_redirect: None,
            error_redirect: None,
        }
    }
}

// PasswordConfig
// ------------------------------------------------------------
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct PasswordConfig {
    pub algorithm: HashingAlgorithm,
    pub requirements: PasswordRequirements,
    pub reset: PasswordReset,
    pub security: PasswordSecurity,
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum HashingAlgorithm {
    Argon2 {
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
    },
    Bcrypt {
        // const MIN_COST: u32 = 4;
        // const MAX_COST: u32 = 31;
        // pub const DEFAULT_COST: u32 = 12;
        cost: u32,
    },
}
impl Default for HashingAlgorithm {
    fn default() -> Self {
        pub const DEFAULT_M_COST: u32 = 19 * 1024; // ~19 MiB
        pub const DEFAULT_T_COST: u32 = 2;
        pub const DEFAULT_P_COST: u32 = 1;

        Self::Argon2 {
            memory_kib: DEFAULT_M_COST,
            iterations: DEFAULT_T_COST,
            parallelism: DEFAULT_P_COST,
        }
    }
}

// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct PasswordRequirements {
    pub min_length: u16,
    pub max_length: u16,
    pub require_uppercase: bool,
    pub require_number: bool,
    pub require_special_char: bool,
}
impl Default for PasswordRequirements {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: false,
            require_number: false,
            require_special_char: false,
        }
    }
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct PasswordReset {
    pub token_expires_in: i64, // maybe option
    // TODO: remove option and use redirect to root
    pub success_redirect: Option<String>,
    pub error_redirect: Option<String>,
}
impl Default for PasswordReset {
    fn default() -> Self {
        Self {
            token_expires_in: 1800,
            success_redirect: None,
            error_redirect: None,
        }
    }
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct PasswordSecurity {
    pub max_failed_attempts: u8,
    pub lockout_duration: i64,
}
impl Default for PasswordSecurity {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration: 1800,
        }
    }
}

// RbacConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct RbacConfig {
    pub enabled: bool,
    pub default_role: String,
}
impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_role: "User".into(),
        }
    }
}

// =============================================================================
// Security Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct Security {
    #[serde(skip_serializing)]
    pub secret_key: String,
    #[serde(default = "default_headers")]
    pub headers: Vec<(String, String)>,
    // pub headers: std::collections::HashMap<String, String>,
}

fn default_headers() -> Vec<(String, String)> {
    vec![
        ("X-Content-Type-Options".into(), "nosniff".into()),
        ("X-Frame-Options".into(), "DENY".into()),
        ("X-XSS-Protection".into(), "0".into()),
        ("Cache-Control".into(), "no-store".into()),
        ("Pragma".into(), "no-cache".into()),
        (
            "Content-Security-Policy".into(),
            "default-src 'self'".into(),
        ),
        ("Content-Type".into(), "application/json".into()),
        (
            "Strict-Transport-Security".into(),
            "max-age=31536000".into(),
        ),
    ]
}

// humantime-serde is great for this — lets you write "15m" in config files.
// server:
//   rate_limiting:
//     enabled: true
//     window_ms: 60000        # 1 minute
//     max_requests: 100
//     strategy: "ip"          # ip | user | api_key
//
//   request:
//     timeout_ms: 30000
//     max_body_size: "2mb"

// =============================================================================
// Logging Configuration
// =============================================================================
// logging:
//   level: "${LOG_LEVEL:info}"    # debug | info | warn | error
//   format: "json"                # json | text
//   redact: ["password", "token", "secret", "authorization"]
