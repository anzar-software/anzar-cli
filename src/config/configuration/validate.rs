use crate::config::{
    AuthStrategy, Authentication, PasswordConfig, PasswordRequirements, PasswordSecurity,
    SameSiteConfig, Security, SessionConfig,
};
use crate::error::{Error, InternalError};

pub trait Validate {
    fn validate(&self) -> Result<(), Vec<Error>>;
}

// =============================================================================
// Security Configuration - REQUIRED
// =============================================================================
impl Validate for Security {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.secret_key.len() < 32 {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "security.secret_key".into(),
                reason: "must be at least 32 characters".into(),
            }));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// =============================================================================
// Authentication Configuration - Optional
// =============================================================================

impl Validate for Authentication {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let AuthStrategy::Jwt(jwt) = &self.strategy {
            if jwt.issuer.is_empty() {
                errors.push(Error::Internal(InternalError::MissingField {
                    field: "auth.jwt.issuer".into(),
                    reason: "must be set".into(),
                }));
            }
            if jwt.audience.is_empty() {
                errors.push(Error::Internal(InternalError::MissingField {
                    field: "auth.jwt.audience".into(),
                    reason: "must be set".into(),
                }));
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
impl Validate for SessionConfig {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.name.is_empty() {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.session.name".into(),
                reason: "must not be empty".into(),
            }));
        }
        if self.max_age == 0 {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.session.max_age".into(),
                reason: "must be greater than 0".into(),
            }));
        }
        if let SameSiteConfig::None = self.same_site {
            if !self.secure {
                errors.push(Error::Internal(InternalError::MissingField {
                    field: "auth.session.secure".into(),
                    reason: "must be true when same_site is None".into(),
                }));
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
// ------------------------------------------------------------
impl Validate for PasswordRequirements {
    fn validate(&self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if self.min_length == 0 {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.password.requirements.min_length".into(),
                reason: "must be greater than 0".into(),
            }));
        }
        if self.max_length < self.min_length {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.password.requirements.max_length".into(),
                reason: "must be greater than or equal to min_length".into(),
            }));
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
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.password.security.max_failed_attempts".into(),
                reason: "must be greater than 0".into(),
            }));
        }
        if self.lockout_duration <= 0 {
            errors.push(Error::Internal(InternalError::MissingField {
                field: "auth.password.security.lockout_duration".into(),
                reason: "must be greater than 0".into(),
            }));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
