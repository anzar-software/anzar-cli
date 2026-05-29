use crate::domain::model::{SigningKey, SigningKeys};
use crate::error::Result;

pub trait SigningKeysServiceTrait {
    fn insert_signing_keys(
        &self,
        private: &str,
        public: &str,
    ) -> impl Future<Output = Result<String>>;

    fn load_active_key(&self) -> impl Future<Output = Result<(String, SigningKey)>>;

    fn find_signing_keys(&self) -> impl Future<Output = Result<Vec<SigningKeys>>>;
    fn rotate_signing_key(&self) -> impl Future<Output = Result<()>>;
}
