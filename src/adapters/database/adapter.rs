use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::{error::Error, utils::query::QueryBuilder};

#[async_trait]
pub trait DatabaseAdapter<T: Send + Sync + Serialize + DeserializeOwned + 'static>:
    Send + Sync
{
    async fn insert(&self, data: T) -> Result<String, Error>;
    async fn find_all(&self, filter: QueryBuilder) -> Result<Vec<T>, Error>;
    async fn find_one(&self, filter: QueryBuilder) -> Result<Option<T>, Error>;
    async fn find_one_and_update(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<Option<T>, Error>;
    async fn update_many(&self, filter: QueryBuilder, update: QueryBuilder) -> Result<(), Error>;
    async fn delete_one(&self, filter: QueryBuilder) -> Result<(), Error>;
    async fn delete_many(&self, filter: QueryBuilder) -> Result<(), Error>;

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
