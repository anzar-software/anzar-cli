use crate::config::AnzarConfiguration;
use crate::infrastructure::database::DatabaseAdapters;

use crate::domain::repositories::{JWTRepository, SessionRepository};
use crate::utils::crypto::Crypto;

#[derive(Clone)]
pub struct SessionService {
    pub(crate) session_repository: SessionRepository,
    pub(crate) jwt_repository: JWTRepository,
    pub(crate) crypto: Crypto,
    pub(crate) configuration: AnzarConfiguration,
}

impl SessionService {
    pub fn new(
        database_adapters: &DatabaseAdapters,
        crypto: &Crypto,
        configuration: &AnzarConfiguration,
    ) -> Self {
        Self {
            session_repository: SessionRepository::new(database_adapters.session_adapter.clone()),
            jwt_repository: JWTRepository::new(database_adapters.jwt_adapter.clone()),
            crypto: crypto.clone(),
            configuration: configuration.clone(),
        }
    }
}
