use crate::config::{AlgorithmConfig, AppState};
use crate::domain::model::SigningKeys;
use crate::error::Result;

use super::traits::SigningKeysServiceTrait;

impl SigningKeysServiceTrait for AppState {
    #[tracing::instrument(name = "crypto.insert_signing_keys", skip(self, public, private))]
    async fn insert_signing_keys(&self, private: &str, public: &str) -> Result<String> {
        let algorithm = &self.configuration.auth.jwt()?.algorithm;
        let kty = match algorithm {
            AlgorithmConfig::EdDSA => "OKP",
            AlgorithmConfig::ES256 | AlgorithmConfig::ES384 => "EC",
            AlgorithmConfig::RS256
            | AlgorithmConfig::RS384
            | AlgorithmConfig::RS512
            | AlgorithmConfig::PS256
            | AlgorithmConfig::PS384
            | AlgorithmConfig::PS512 => "RSA",
        };

        let encrypted_private_key = self.crypto.clone().aes.encrypt(private)?;
        let signing_keys = SigningKeys::new(&encrypted_private_key, public)
            .with_algorithm(algorithm.as_str())
            .with_kid("k1")
            .with_kty(kty);

        self.repositories
            .signing_keys_repository
            .insert(signing_keys)
            .await
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    async fn find_signing_key(&self) -> Result<(String, SigningKeys)> {
        let signing_key = self.repositories.signing_keys_repository.find().await?;
        let private_key = self
            .crypto
            .clone()
            .aes
            .decrypt(&signing_key.encrypted_private_key)?;

        Ok((private_key, signing_key))
    }
}
