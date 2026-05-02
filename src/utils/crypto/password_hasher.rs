use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::error::{Error, InternalError, Result};

pub trait Hashable: Send + Sync {
    fn hash(&self, value: &str) -> Result<String>;
    fn verify(&self, a: &str, b: &str) -> Result<bool>;
}

// pub struct Argon2Password {
//     memory_cost: u32,
//     iterations: u32,
// }

// =============================================================================
// Password Hasher - Argon
// =============================================================================
#[derive(Clone)]
pub struct Argon2Password;

impl Hashable for Argon2Password {
    fn hash(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                tracing::error!("Failed to hash user password: {:?}", e);
                Error::Internal(InternalError::Hashing)
            })?;

        Ok(hash.to_string())
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        static DUMMY_HASH: &str = "$argon2id$v=19$m=65536,t=3,p=4$\
     Lm1Jk9XQ2E1o8XxZMZ1jPQ$\
     8vBxrT9uC1NQb3lQfa2RyEBJxK2Sr6ELrRvsGqIzJxA";

        let parsed = PasswordHash::new(hash)
            .or_else(|_| PasswordHash::new(DUMMY_HASH))
            .map_err(|_| Error::Internal(InternalError::Hashing))?;

        let isvalid = Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();

        Ok(isvalid)
    }
}

// =============================================================================
// Password Hasher - BCrypt
// =============================================================================
#[derive(Clone)]
pub struct BcryptPassword;

impl Hashable for BcryptPassword {
    fn hash(&self, _value: &str) -> Result<String> {
        unimplemented!()
    }

    fn verify(&self, _a: &str, _b: &str) -> Result<bool> {
        unimplemented!()
    }
}
