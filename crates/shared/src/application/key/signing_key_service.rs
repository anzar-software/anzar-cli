use chrono::Utc;

use crate::error::Result;

use crate::config::AlgorithmConfig;
use crate::domain::model::{SigningKey, SigningKeys};
use crate::intern::key::KeyService;
use crate::utils::crypto::JwtSigner;

impl KeyService {
    fn generate_keypair(&self) -> (String, String) {
        self.crypto.openssl.gen_prv_pub_key()
    }
    fn build_signingkey(&self, private: &str, public: &str) -> Result<SigningKey> {
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
        let kid = super::support::jwk_thumbprint_rsa(public);

        Ok(SigningKey::new(&encrypted_private_key, public)
            .with_algorithm(algorithm.as_str())
            .with_kid(&kid)
            .with_kty(kty))
    }

    #[tracing::instrument(name = "crypto.insert_signing_keys", skip(self))]
    pub async fn save_new_key(&self) -> Result<String> {
        let (private, public) = self.generate_keypair();
        let key = self.build_signingkey(&private, &public)?;

        self.signing_key_repository.insert(key).await
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    pub async fn load_active_key(&self) -> Result<(String, SigningKey)> {
        let key = self.signing_key_repository.find().await?;

        let private_key = self
            .crypto
            .clone()
            .aes
            .decrypt(&key.encrypted_private_key)?;

        Ok((private_key, key))
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    pub async fn find_signing_keys(&self) -> Result<Vec<SigningKeys>> {
        let signing_keys = self.signing_key_repository.find_valid_keys().await?;
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

    #[tracing::instrument(name = "crypto.find_all_keys", skip(self))]
    pub async fn list_keys(&self) -> Result<Vec<SigningKey>> {
        self.signing_key_repository.find_all().await
    }

    #[tracing::instrument(name = "crypto.rotate_signing_key", skip(self))]
    pub async fn rotate_signing_key(&self) -> Result<(SigningKey, SigningKey)> {
        // 1.
        let max_token_ttl = self.configuration.auth.jwt()?.refresh_token_expires_in;
        let retired_key = self
            .signing_key_repository
            .retire_oldkey(max_token_ttl)
            .await?;

        // 2. 3.
        self.save_new_key().await?;
        let (private, active_key) = self.load_active_key().await?;

        // 4.
        let jwt = JwtSigner::new(&private, &active_key, self.configuration.auth.jwt()?);
        self.crypto.rotate_jwt(jwt);

        Ok((retired_key, active_key))
    }

    #[tracing::instrument(name = "signing_key.prune", skip(self))]
    pub async fn prune(&self) -> Result<Vec<SigningKey>> {
        let keys = self.list_keys().await?;

        let mut removed_keys: Vec<SigningKey> = Vec::new();
        for key in keys {
            let expired = key.expires_at.map(|dt| Utc::now() > dt).unwrap_or(false);
            if key.status == "revoked" || expired {
                let id = key.id()?;
                self.signing_key_repository.delete(id).await?;

                removed_keys.push(key);
            }
        }

        Ok(removed_keys)
    }

    #[tracing::instrument(name = "signing_key.revoke", skip(self))]
    pub async fn revoke(&self, id: &str) -> Result<SigningKey> {
        self.signing_key_repository.revoke_key(id).await
    }
}
