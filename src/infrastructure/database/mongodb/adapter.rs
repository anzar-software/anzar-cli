use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{Collection, options::ReturnDocument};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::domain::database::DatabaseAdapter;
use crate::error::{CredentialField, Error, ValidationError};

use crate::domain::query::{IntoDbFilter, QueryBuilder};

#[derive(Debug, Clone)]
pub struct MongodbAdapter<T: Send + Sync + Debug> {
    collection: Collection<T>,
}

impl<T: Send + Sync + Debug> MongodbAdapter<T> {
    pub fn new(client: &mongodb::Client, cnx: &str, name: &str) -> Self {
        MongodbAdapter {
            collection: client.database(cnx).collection::<T>(name),
        }
    }
}

#[async_trait]
impl<T> DatabaseAdapter<T> for MongodbAdapter<T>
where
    T: Debug + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    async fn insert(&self, data: T) -> Result<String, Error> {
        let operation = self.collection.insert_one(data);

        // if let Some(s) = session {
        //     operation = operation.session(s);
        // }

        let doc = operation.await?;

        let id = doc.inserted_id.as_object_id().ok_or_else(|| {
            Error::Validation(ValidationError::Malformed {
                field: CredentialField::ObjectId,
            })
        })?;

        Ok(id.to_string())
    }

    async fn find_all(&self, _query: QueryBuilder) -> Result<Vec<T>, Error> {
        self.collection
            .find(mongodb::bson::doc! {})
            .await?
            .try_collect()
            .await
            .map_err(Into::into)
    }

    async fn find_one(&self, query: QueryBuilder) -> Result<Option<T>, Error> {
        let doc = query.into_mongo_filter();

        self.collection.find_one(doc).await.map_err(Into::into)
    }

    async fn find_one_and_update(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<Option<T>, Error> {
        let filter = filter.into_mongo_filter();
        let update = update.into_mongo_update();

        self.collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::Before)
            .await
            .map_err(Into::into)
    }

    async fn update_many(&self, filter: QueryBuilder, update: QueryBuilder) -> Result<(), Error> {
        let filter = filter.into_mongo_filter();
        let update = update.into_mongo_update();

        self.collection.update_many(filter, update).await?;
        Ok(())
    }

    async fn delete_one(&self, query: QueryBuilder) -> Result<(), Error> {
        let filter = query.into_mongo_filter();

        self.collection.delete_one(filter).await?;
        Ok(())
    }

    async fn delete_many(&self, query: QueryBuilder) -> Result<(), Error> {
        let filter = query.into_mongo_filter();

        self.collection.delete_many(filter).await?;
        Ok(())
    }
}
