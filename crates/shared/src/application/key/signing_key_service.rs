use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::Utc;

use crate::error::Result;

use crate::config::AlgorithmConfig;
use crate::domain::model::{SigningKey, SigningKeys};
use crate::intern::key::KeyService;
use crate::utils::crypto::JwtSigner;

impl KeyService {
    fn build_signingkey(&self, private: &str, public: &str) -> Result<SigningKey> {
        let encrypted_private_key = self.crypto.clone().aes.encrypt(private)?;
        let public_key_pem: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(public).unwrap();

        let algorithm = &self.configuration.auth.jwt()?.algorithm;
        let (kty, kid) = match algorithm {
            AlgorithmConfig::EdDSA => self.crypto.openssl.build_okp(public_key_pem)?,
            AlgorithmConfig::ES256 | AlgorithmConfig::ES384 => {
                self.crypto.openssl.build_ec(public_key_pem)?
            }
            _ => self.crypto.openssl.build_rsa(public_key_pem)?,
        };

        Ok(SigningKey::new(&encrypted_private_key, public)
            .with_algorithm(algorithm.as_str())
            .with_kid(&kid)
            .with_kty(&kty))
    }

    #[tracing::instrument(name = "crypto.insert_signing_keys", skip(self))]
    async fn new_key(&self) -> Result<String> {
        let (private, public) = self.crypto.openssl.gen_prv_pub_key()?;

        let key = self.build_signingkey(&private, &public)?;

        self.signing_key_repository.insert(key).await
    }
}

impl KeyService {
    pub async fn load_or_create(&self) -> Result<Vec<SigningKeys>> {
        let keys = match self.get_jwks().await {
            Ok(response) => {
                if response.is_empty()
                    || response
                        .iter()
                        .find(|sk| sk.key.status == "active")
                        .is_none()
                {
                    self.new_key().await?;
                    self.get_jwks().await?
                } else {
                    response
                }
            }
            Err(_) => {
                self.new_key().await?;
                self.get_jwks().await?
            }
        };

        Ok(keys)
    }

    #[tracing::instrument(name = "crypto.find_all_keys", skip(self))]
    pub async fn list(&self) -> Result<Vec<SigningKey>> {
        self.signing_key_repository.find_all().await
    }
    #[tracing::instrument(name = "crypto.rotate_signing_key", skip(self))]
    pub async fn rotate(&self) -> Result<(SigningKey, SigningKey)> {
        // 1.
        let max_token_ttl = self.configuration.auth.jwt()?.refresh_token_expires_in;
        let retired_key = self
            .signing_key_repository
            .retire_oldkey(max_token_ttl)
            .await?;

        // 2.
        self.new_key().await?;

        //3.
        let keys = self.get_jwks().await?;
        let active_key = keys.iter().find(|k| k.key.status == "active").unwrap();

        // 4.
        let signer = JwtSigner::new(keys.clone(), self.configuration.auth.jwt()?);
        self.crypto.rotate_jwt(signer);

        Ok((retired_key, active_key.key.clone()))
    }
    #[tracing::instrument(name = "signing_key.prune", skip(self))]
    pub async fn prune(&self) -> Result<Vec<SigningKey>> {
        let keys = self.list().await?;

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
    pub async fn revoke(&self, kid: &str) -> Result<SigningKey> {
        self.signing_key_repository.revoke_key(kid).await
    }

    #[tracing::instrument(name = "crypto.find_signing_key", skip(self))]
    pub async fn get_jwks(&self) -> Result<Vec<SigningKeys>> {
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
}
