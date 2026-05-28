use crate::{domain::model::SigningKeys, error::Result};

pub trait SigningKeysServiceTrait {
    fn insert_signing_keys(
        &self,
        private: &str,
        public: &str,
    ) -> impl Future<Output = Result<String>>;

    fn find_signing_key(&self) -> impl Future<Output = Result<(String, SigningKeys)>>;
}
