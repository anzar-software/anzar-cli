use crate::config::{AlgorithmConfig, AppState};
use crate::domain::model::{SigningKey, SigningKeys};
use crate::error::Result;
use crate::utils::crypto::JwtSigner;

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
        let signing_keys = SigningKey::new(&encrypted_private_key, public)
            .with_algorithm(algorithm.as_str())
            .with_kid("k1")
            .with_kty(kty);

        self.repositories
            .signing_keys_repository
            .insert(signing_keys)
            .await
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    async fn load_active_key(&self) -> Result<(String, SigningKey)> {
        let key = self.repositories.signing_keys_repository.find().await?;

        let private_key = self
            .crypto
            .clone()
            .aes
            .decrypt(&key.encrypted_private_key)?;

        Ok((private_key, key))
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    async fn find_signing_keys(&self) -> Result<Vec<SigningKeys>> {
        let signing_keys = self.repositories.signing_keys_repository.find_all().await?;
        let mut data: Vec<SigningKeys> = Vec::new();

        for key in signing_keys {
            let private_key = self
                .crypto
                .clone()
                .aes
                .decrypt(&key.encrypted_private_key)?;
            data.push(SigningKeys { private_key, key })
        }

        Ok(data)
    }

    #[tracing::instrument(name = "crypto.rotate_signing_key", skip(self))]
    async fn rotate_signing_key(&self) -> Result<()> {
        let max_token_ttl = 2000;
        let _ = self
            .repositories
            .signing_keys_repository
            .retire_oldkey(max_token_ttl)
            .await?;

        let (private, public) = self.crypto.openssl.gen_prv_pub_key();
        self.insert_signing_keys(&private, &public).await?;

        let (_, key) = self.load_active_key().await?;
        let jwt = JwtSigner::new(&private, &key, self.configuration.auth.jwt()?);

        self.crypto.rotate_jwt(jwt);

        Ok(())
    }
}
