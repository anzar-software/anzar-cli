use std::str::FromStr;

use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::error::{CoreError, InternalError, Result};

pub trait Hashable: Send + Sync {
    fn hash(&self, value: &str) -> Result<String>;
    fn verify(&self, a: &str, b: &str) -> Result<bool>;
}

// =============================================================================
// Password Hasher - Argon
// =============================================================================
#[derive(Clone)]
pub struct Argon2Password {
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
}
impl Default for Argon2Password {
    fn default() -> Self {
        pub const DEFAULT_M_COST: u32 = 19 * 1024; // ~19 MiB
        pub const DEFAULT_T_COST: u32 = 2;
        pub const DEFAULT_P_COST: u32 = 1;

        Self {
            memory_kib: DEFAULT_M_COST,
            iterations: DEFAULT_T_COST,
            parallelism: DEFAULT_P_COST,
        }
    }
}
impl Argon2Password {
    pub fn new(m_cost: u32, t_cost: u32, p_cost: u32) -> Self {
        Self {
            memory_kib: m_cost,
            iterations: t_cost,
            parallelism: p_cost,
        }
    }
}

impl Hashable for Argon2Password {
    fn hash(&self, password: &str) -> Result<String> {
        let params = Params::new(
            self.memory_kib,  // m_cost: memory size in KiB (19 MB)
            self.iterations,  // t_cost: number of iterations
            self.parallelism, // p_cost: parallelism (threads)
            None,             // output length (None = default 32 bytes)
        )
        .unwrap();

        let salt = SaltString::generate(&mut OsRng);

        let hash = Argon2::new(
            argon2::Algorithm::default(),
            argon2::Version::default(),
            params,
        )
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Failed to hash user password: {:?}", e);
            CoreError::Internal(InternalError::Hashing)
        })?;

        Ok(hash.to_string())
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        static DUMMY_HASH: &str = "$argon2id$v=19$m=65536,t=3,p=4$\
     Lm1Jk9XQ2E1o8XxZMZ1jPQ$\
     8vBxrT9uC1NQb3lQfa2RyEBJxK2Sr6ELrRvsGqIzJxA";

        let parsed = PasswordHash::new(hash)
            .or_else(|_| PasswordHash::new(DUMMY_HASH))
            .map_err(|_| CoreError::Internal(InternalError::Hashing))?;

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
pub struct BcryptPassword {
    cost: u32,
}
impl BcryptPassword {
    pub fn new(cost: u32) -> Self {
        Self { cost }
    }
}

impl Hashable for BcryptPassword {
    fn hash(&self, value: &str) -> Result<String> {
        let hash = bcrypt::hash(value, self.cost).map_err(|e| {
            tracing::error!("Failed to hash user password: {:?}", e);
            CoreError::Internal(InternalError::Hashing)
        })?;

        Ok(hash.to_string())
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        static DUMMY_HASH: &str = "$2b$12$kgQ2Rl.I22hVIJVklF9OceDIv8EBoMXtlw6U15pVLlmleTLfRUMRe";

        let hash_to_verify = if bcrypt::HashParts::from_str(hash).is_ok() {
            hash.to_string()
        } else {
            DUMMY_HASH.to_string()
        };

        let is_valid = bcrypt::verify(password, &hash_to_verify)
            .map_err(|_| CoreError::Internal(InternalError::Hashing))?;

        Ok(is_valid)
    }
}
