use super::{
    App, AuthStrategy, Authentication, HttpsConfig, PasswordConfig, PasswordRequirements,
    PasswordSecurity, SameSiteConfig, Security, Server, SessionConfig,
};
use crate::error::Error;

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<Error>>;
}
// =============================================================================
// App Configuration - REQUIRED
// =============================================================================
impl Validate for App {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        let url_ok = self.url.starts_with("http://") || self.url.starts_with("https://");
        if !url_ok {
            errors.push(Error::InvalidConfig {
                key: "app.url".into(),
                reason: "must start with http:// or https://".into(),
            });
        }

        // app.environment should be one of the known values
        let valid_envs = ["development", "staging", "production"];
        if !valid_envs.contains(&self.environment.as_str()) {
            errors.push(Error::InvalidConfig {
                key: "app.environment".into(),
                reason: "must be development, staging, or production".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =============================================================================
// Security Configuration - REQUIRED
// =============================================================================
impl Validate for Security {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.secret_key.len() < 32 {
            errors.push(Error::InvalidConfig {
                key: "security.secret_key".into(),
                reason: "must be at least 32 characters".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =============================================================================
// Authentication Configuration - strategy, password
// =============================================================================

impl Validate for Authentication {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let AuthStrategy::Jwt(jwt) = &self.strategy {
            if jwt.issuer.is_empty() {
                errors.push(Error::InvalidConfig {
                    key: "auth.jwt.issuer".into(),
                    reason: "must be set (e.g. \"https://your-app.com\")".into(),
                });
            }
            if jwt.audience.is_empty() {
                errors.push(Error::InvalidConfig {
                    key: "auth.jwt.audience".into(),
                    reason: "must be set (e.g. \"your-app-api\")".into(),
                });
            }
        }

        if let AuthStrategy::Session(session) = &self.strategy {
            if let Err(e) = session.validate() {
                errors.extend(e);
            }
        }

        // Collect child errors with context
        if let Err(e) = self.password.validate() {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
// SessionConfig
// ------------------------------------------------------------
impl Validate for SessionConfig {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.name.is_empty() {
            errors.push(Error::InvalidConfig {
                key: "auth.session.name".into(),
                reason: "must not be empty".into(),
            });
        }
        if self.max_age == 0 {
            errors.push(Error::InvalidConfig {
                key: "auth.session.max_age".into(),
                reason: "must be a positive number of seconds (e.g. 3600 for 1 hour)".into(),
            });
        }
        if let SameSiteConfig::None = self.same_site {
            if !self.secure {
                errors.push(Error::InvalidConfig {
                    key: "auth.session.secure".into(),
                    reason: "SameSite=None requires Secure=true, otherwise browsers will reject the cookie".into(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
// PasswordConfiguration
// ------------------------------------------------------------
impl Validate for PasswordConfig {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.requirements.validate() {
            errors.extend(e);
        }
        if let Err(e) = self.security.validate() {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
impl Validate for PasswordRequirements {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.min_length == 0 {
            errors.push(Error::InvalidConfig {
                key: "auth.password.requirements.min_length".into(),
                reason: "must be at least 1 character".into(),
            });
        }
        if self.max_length < self.min_length {
            errors.push(Error::InvalidConfig {
                key: "auth.password.requirements.max_length".into(),
                reason: format!(
                    "min_length ({}) must be less than max_length",
                    self.min_length
                ),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
impl Validate for PasswordSecurity {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.max_failed_attempts == 0 {
            errors.push(Error::InvalidConfig {
                key: "auth.password.security.max_failed_attempts".into(),
                reason: "must be at least 1 (e.g. 5 to lock after 5 failed attempts)".into(),
            });
        }
        if self.lockout_duration <= 0 {
            errors.push(Error::InvalidConfig {
                key: "auth.password.security.lockout_duration".into(),
                reason: "must be a positive number of seconds (e.g. 1800 for 30 minutes)".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =============================================================================
// Server Configuration - Https
// =============================================================================
impl Validate for Server {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.https.validate() {
            errors.extend(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
// ------------------------------------------------------------
impl Validate for HttpsConfig {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.enabled && self.cert_path.is_none() {
            errors.push(Error::InvalidConfig {
                key: "server.https.cert_path".into(),
                reason: "required when https is enabled".into(),
            });
        }
        if self.enabled && self.key_path.is_none() {
            errors.push(Error::InvalidConfig {
                key: "server.https.key_path".into(),
                reason: "required when https is enabled".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
