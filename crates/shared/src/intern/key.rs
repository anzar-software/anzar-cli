use crate::config::AnzarConfiguration;
use crate::infrastructure::database::DatabaseAdapters;

use crate::domain::repositories::SigningKeysRepository;
use crate::utils::crypto::Crypto;

#[derive(Clone)]
pub struct KeyService {
    pub(crate) signing_key_repository: SigningKeysRepository,
    pub(crate) crypto: Crypto,
    pub(crate) configuration: AnzarConfiguration,
}

impl KeyService {
    pub fn new(
        database_adapters: &DatabaseAdapters,
        crypto: &Crypto,
        configuration: &AnzarConfiguration,
    ) -> Self {
        Self {
            signing_key_repository: SigningKeysRepository::new(
                database_adapters.signing_keys_adapter.clone(),
            ),
            crypto: crypto.clone(),
            configuration: configuration.clone(),
        }
    }
}
