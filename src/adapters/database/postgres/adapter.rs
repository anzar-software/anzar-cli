use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{FromRow, Pool, Postgres};

use crate::adapters::database::DatabaseAdapter;
use crate::adapters::database::bindings::traits::{IdResult, PgInsert};
use crate::error::Error;
use crate::utils::query::{IntoDbFilter, QueryBuilder};

pub struct PostgreSQLAdapter<T: Send + Sync> {
    pool: Pool<Postgres>,
    table: String,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync> PostgreSQLAdapter<T> {
    pub fn new(pool: &Pool<Postgres>, table: &str) -> Self {
        PostgreSQLAdapter {
            pool: pool.clone(),
            table: table.into(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T> DatabaseAdapter<T> for PostgreSQLAdapter<T>
where
    T: Send
        + Sync
        + Serialize
        + DeserializeOwned
        + 'static
        + for<'r> FromRow<'r, sqlx::postgres::PgRow>
        + Unpin
        + PgInsert,
{
    async fn insert(&self, data: T) -> Result<String, Error> {
        let columns: String = T::columns()
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<String>>()
            .join(", ");
        let values = T::columns()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
            self.table, columns, values
        );

        let query = sqlx::query_as::<_, IdResult>(&sql);

        let row: IdResult = data
            .bind_query(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                dbg!(&e);
                Error::DatabaseError(e.to_string())
            })?;

        Ok(row.id)
    }

    async fn find_one(&self, query: QueryBuilder) -> Result<Option<T>, Error> {
        let (where_clause, values) = query.into_postgres_filter(0);

        let sql = format!("SELECT * FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in values {
            query = v.bind_pg(query);
        }

        query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn find_one_and_update(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<Option<T>, Error> {
        let (set_clause, update_values) = update.into_postgres_update();
        let offset = update_values.len();
        let (where_clause, filter_values) = filter.into_postgres_filter(offset);

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, set_clause, where_clause
        );

        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in update_values.into_iter().chain(filter_values) {
            query = v.bind_pg(query);
        }

        query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn update_many(&self, filter: QueryBuilder, update: QueryBuilder) -> Result<(), Error> {
        let (set_clause, update_values) = update.into_postgres_update();
        let offset = update_values.len();
        let (where_clause, filter_values) = filter.into_postgres_filter(offset);

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, set_clause, where_clause
        );

        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in update_values.into_iter().chain(filter_values) {
            query = v.bind_pg(query);
        }

        query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_one(&self, _filter: QueryBuilder) -> Result<(), Error> {
        Ok(())
    }
    async fn delete_many(&self, _filter: QueryBuilder) -> Result<(), Error> {
        Ok(())
    }
}
