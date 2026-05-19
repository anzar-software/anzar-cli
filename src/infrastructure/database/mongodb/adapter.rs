use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{Collection, options::ReturnDocument};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::error::Error;

use crate::domain::database::DatabaseAdapter;
use crate::domain::query::{IntoBsonDocument, IntoDbFilter, QueryBuilder};
use crate::infrastructure::database::bindings::traits::MongoInsert;

#[derive(Debug, Clone)]
pub struct MongodbAdapter<T: Send + Sync + Debug> {
    // client: mongodb::Client,
    collection: Collection<T>,
}

impl<T: Send + Sync + Debug> MongodbAdapter<T> {
    pub fn new(client: &mongodb::Client, cnx: &str, name: &str) -> Self {
        MongodbAdapter {
            // client: client.clone(),
            collection: client.database(cnx).collection::<T>(name),
        }
    }
}

#[async_trait]
impl<T> DatabaseAdapter<T> for MongodbAdapter<T>
where
    T: Debug
        + Send
        + Sync
        + Serialize
        + DeserializeOwned
        + 'static
        + MongoInsert
        + IntoBsonDocument,
{
    async fn insert(&self, data: T) -> Result<String, Error> {
        let collection = self.collection.clone_with_type::<mongodb::bson::Document>();
        let doc = data.into_bson_document().unwrap();

        let operation = collection.insert_one(doc);

        let doc = operation.await?;

        let id = doc.inserted_id.as_object_id().ok_or_else(|| {
            Error::Validation(crate::error::ValidationError::Malformed {
                field: crate::error::CredentialField::ObjectId,
            })
        })?;
        Ok(id.to_string())
    }
    async fn insert_many(&self, data: Vec<T>) -> Result<Vec<String>, Error> {
        let collection = self.collection.clone_with_type::<mongodb::bson::Document>();
        let docs: Vec<mongodb::bson::Document> = data
            .into_iter()
            .map(|d| d.into_bson_document().unwrap())
            .collect();
        let operation = collection.insert_many(docs);

        let doc = operation.await?;

        let ids = doc
            .inserted_ids
            .into_values()
            .map(|v| {
                v.as_object_id().map(|id| id.to_string()).ok_or_else(|| {
                    Error::Validation(crate::error::ValidationError::Malformed {
                        field: crate::error::CredentialField::ObjectId,
                    })
                })
            })
            .collect::<Result<Vec<String>, _>>()?;

        Ok(ids)
    }

    async fn upsert(&self, data: T) -> Result<String, Error> {
        let doc = data.into_bson_document().unwrap();

        let filter: mongodb::bson::Document = T::uniques()
            .iter()
            .filter_map(|&key| {
                let clean_key = key.trim_matches('"');
                doc.get(clean_key)
                    .map(|val| (clean_key.to_string(), val.clone()))
            })
            .collect();
        let update = mongodb::bson::doc! { "$set": doc };

        let collection = self.collection.clone_with_type::<mongodb::bson::Document>();
        let result = collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::After)
            .upsert(true)
            .await?
            .ok_or(Error::Internal(crate::error::InternalError::Database(
                "upsert failed".into(),
            )))?;

        let id = result
            .get("_id")
            .and_then(|id| id.as_object_id())
            .ok_or(Error::Internal(crate::error::InternalError::Database(
                "missing id".into(),
            )))?;

        Ok(id.to_string())
    }

    async fn upsert_many(&self, data: Vec<T>) -> Result<Vec<String>, Error> {
        let mut ids = Vec::with_capacity(data.len());
        for item in data {
            let id = self.upsert(item).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    // NOTE
    // if mongodb is upgraded to version 8.x
    // use this method
    //
    // async fn upsert_many(&self, data: Vec<T>) -> Result<Vec<String>, Error> {
    //     let mut operations = vec![];
    //
    //     for item in &data {
    //         let doc = mongodb::bson::to_document(item)?;
    //
    //         let filter: mongodb::bson::Document = T::uniques()
    //             .iter()
    //             .filter_map(|&key| doc.get(key).map(|val| (key.to_string(), val.clone())))
    //             .collect();
    //         let update = mongodb::bson::doc! { "$set": doc };
    //
    //         let update_model = mongodb::options::UpdateOneModel::builder()
    //             .namespace(self.collection.namespace())
    //             .filter(filter)
    //             .update(update)
    //             .upsert(true)
    //             .build();
    //         operations.push(mongodb::options::WriteModel::UpdateOne(update_model));
    //     }
    //
    //     let result = self.client.bulk_write(operations).verbose_results().await?;
    //
    //     let ids: Vec<String> = result
    //         .update_results
    //         .into_values()
    //         .map(|v| {
    //             v.upserted_id
    //                 .and_then(|id| id.as_object_id())
    //                 .map(|id| id.to_string())
    //                 .ok_or_else(|| {
    //                     Error::Validation(crate::error::ValidationError::Malformed {
    //                         field: crate::error::CredentialField::ObjectId,
    //                     })
    //                 })
    //         })
    //         .collect::<Result<Vec<String>, _>>()?;
    //     Ok(ids)
    // }

    // -- FINDS
    async fn find_all(&self, query: QueryBuilder) -> Result<Vec<T>, Error> {
        let doc = query.into_mongo_filter();

        self.collection
            .find(doc)
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
