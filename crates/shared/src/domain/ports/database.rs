use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use super::query::QueryBuilder;
use crate::error::CoreError;

#[async_trait]
pub trait DatabaseAdapter<T: Send + Sync + Serialize + DeserializeOwned + 'static>:
    Send + Sync
{
    // -- INSERTS
    async fn insert(&self, data: T) -> Result<String, CoreError>;
    async fn insert_many(&self, data: Vec<T>) -> Result<Vec<String>, CoreError>;
    async fn upsert(&self, data: T) -> Result<String, CoreError>;
    async fn upsert_many(&self, data: Vec<T>) -> Result<Vec<String>, CoreError>;

    // -- FINDS
    async fn find_one(&self, filter: QueryBuilder) -> Result<Option<T>, CoreError>;
    async fn find_all(&self, filter: QueryBuilder) -> Result<Vec<T>, CoreError>;

    // -- UPDATES
    async fn find_one_and_update(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<Option<T>, CoreError>;
    async fn update_many(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<(), CoreError>;

    // -- DELETES
    async fn delete_one(&self, filter: QueryBuilder) -> Result<(), CoreError>;
    async fn delete_many(&self, filter: QueryBuilder) -> Result<(), CoreError>;

    // transactions as a separate concern
    // async fn begin_transaction(&self) -> Result<Box<dyn Transaction>, Error>;
}

// pub trait Transaction: Send + Sync {
//     fn commit(self) -> impl std::future::Future<Output = Result<(), Error>> + Send;
//     fn rollback(self) -> impl std::future::Future<Output = Result<(), Error>> + Send;
// }

// #[async_trait]
// pub trait Transaction: Send + Sync {
//     async fn commit(self) -> Result<(), Error>;
//     async fn rollback(self) -> Result<(), Error>;
// }
