pub mod validate;

use crate::{error::Error, shared::configuration::validate::Validate};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
            Err(Error::InvalidAppConfig(errors))
        }
    }
}

// =============================================================================
// API Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct App {
    pub environment: String,
    pub url: String,
}

// =============================================================================
// Database Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Database {
    pub driver: DatabaseDriver,
    pub connection_string: String,
    pub cache: Cache,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum DatabaseDriver {
    #[default]
    MongoDB,
    PostgreSQL,
    SQLite,
}
impl fmt::Display for DatabaseDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseDriver::MongoDB => write!(f, "MongoDB"),
            DatabaseDriver::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseDriver::SQLite => write!(f, "SQLite"),
        }
    }
}
// Cache
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cache {
    pub driver: CacheDriver,
    pub url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum CacheDriver {
    #[default]
    MemCached,
    Redis,
}
impl fmt::Display for CacheDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheDriver::MemCached => write!(f, "MemCached"),
            CacheDriver::Redis => write!(f, "Redis"),
        }
    }
}

// =============================================================================
// Server Configuration - Optional
// =============================================================================
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Server {
    pub https: HttpsConfig,
    pub cors: CorsConfig,
}
// HttpsConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Authentication {
    pub strategy: AuthStrategy,
    pub email: EmailConfig,
    pub password: PasswordConfig,
}
// AuthStrategy
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AuthStrategy {
    Session(SessionConfig),
    Jwt(JwtConfig),
}
impl fmt::Display for AuthStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthStrategy::Session(_) => write!(f, "Session"),
            AuthStrategy::Jwt(_) => write!(f, "Jwt"),
        }
    }
}
impl Default for AuthStrategy {
    fn default() -> Self {
        Self::Session(SessionConfig::default())
    }
}
// JwtConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
// SessionConfig
// ------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    pub name: String,
    pub max_age: u64,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSiteConfig,
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum SameSiteConfig {
    #[default]
    Strict,
    Lax,
    None,
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct EmailConfig {
    pub verification: EmailVerification,
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct PasswordConfig {
    pub algorithm: HashingAlgorithm,
    pub requirements: PasswordRequirements,
    pub reset: PasswordReset,
    pub security: PasswordSecurity,
}
// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum HashingAlgorithm {
    Argon2 {
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
    },
    Bcrypt {
        cost: u32,
    },
}
impl Default for HashingAlgorithm {
    fn default() -> Self {
        Self::Argon2 {
            memory_kib: 65536, // 64 MiB
            iterations: 3,
            parallelism: 4,
        }
    }
}

// ************************************************************
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

// =============================================================================
// Security Configuration - REQUIRED
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
