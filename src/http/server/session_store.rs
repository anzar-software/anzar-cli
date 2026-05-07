use std::collections::HashMap;
use actix_session::storage::{LoadError, SaveError, UpdateError, DeleteError, SessionStore, SessionKey};
use actix_web::cookie::time::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct CacheSessionStore {
    cache: Arc<dyn CacheAdapter>,
    key_prefix: String,
}

impl CacheSessionStore {
    pub fn new(cache: Arc<dyn CacheAdapter>) -> Self {
        Self {
            cache,
            key_prefix: "session:".to_string(),
        }
    }

    fn full_key(&self, session_key: &SessionKey) -> String {
        format!("{}{}", self.key_prefix, session_key.as_ref())
    }
}

#[async_trait]
impl SessionStore for CacheSessionStore {
    async fn load(
        &self,
        session_key: &SessionKey,
    ) -> Result<Option<HashMap<String, String>>, LoadError> {
        let key = self.full_key(session_key);
        match self.cache.find_one(&key).await {
            Ok(Some(value)) => {
                let state = serde_json::from_str(&value)
                    .map_err(|e| LoadError::Deserialization(anyhow::anyhow!(e)))?;
                Ok(Some(state))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(LoadError::Other(anyhow::anyhow!(e))),
        }
    }

    async fn save(
        &self,
        session_state: HashMap<String, String>,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        // Generate a cryptographically random session key
        let session_key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let value = serde_json::to_string(&session_state)
            .map_err(|e| SaveError::Serialization(anyhow::anyhow!(e)))?;

        let expiry = ttl.whole_seconds().max(0) as u64;
        let key = format!("{}{}", self.key_prefix, session_key);

        self.cache
            .insert(&key, value, expiry)
            .await
            .map_err(|e| SaveError::Other(anyhow::anyhow!(e)))?;

        session_key
            .try_into()
            .map_err(|e| SaveError::Other(anyhow::anyhow!(e)))
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: HashMap<String, String>,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let key = self.full_key(&session_key);
        let value = serde_json::to_string(&session_state)
            .map_err(|e| UpdateError::Serialization(anyhow::anyhow!(e)))?;

        let expiry = ttl.whole_seconds().max(0) as u64;

        // Check if key still exists — it may have expired between load and update
        match self.cache.find_one(&key).await {
            Ok(Some(_)) => {
                self.cache
                    .update(&key, value, expiry)
                    .await
                    .map_err(|e| UpdateError::Other(anyhow::anyhow!(e)))?;
                Ok(session_key)
            }
            // Session expired between request start and now — save as new
            Ok(None) => {
                self.cache
                    .insert(&key, value, expiry)
                    .await
                    .map_err(|e| UpdateError::Other(anyhow::anyhow!(e)))?;
                Ok(session_key)
            }
            Err(e) => Err(UpdateError::Other(anyhow::anyhow!(e))),
        }
    }

    async fn delete(&self, session_key: &SessionKey) -> Result<(), DeleteError> {
        let key = self.full_key(session_key);
        self.cache
            .delete_one(&key)
            .await
            .map_err(|e| DeleteError::Other(anyhow::anyhow!(e)))?;
        Ok(())
    }
}
