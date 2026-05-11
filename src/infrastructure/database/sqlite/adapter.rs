use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{FromRow, Pool, Sqlite};
use std::marker::PhantomData;

use crate::error::Error;

use crate::domain::database::DatabaseAdapter;
use crate::domain::query::{IntoDbFilter, QueryBuilder};

// FIXME simplify the import
use crate::infrastructure::database::bindings::traits::{IdResult, SqliteInsert};

pub struct SQLiteAdapter<T: Send + Sync> {
    pool: Pool<Sqlite>,
    table: String,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync> SQLiteAdapter<T> {
    pub fn new(pool: &Pool<Sqlite>, table: &str) -> Self {
        SQLiteAdapter {
            pool: pool.clone(),
            table: table.into(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<T> DatabaseAdapter<T> for SQLiteAdapter<T>
where
    T: Send
        + Sync
        + Serialize
        + DeserializeOwned
        + 'static
        + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>
        + Unpin
        + SqliteInsert,
{
    async fn insert_many(&self, data: Vec<T>) -> Result<Vec<String>, Error> {
        let columns: String = T::columns()
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<String>>()
            .join(", ");

        let mut count = 0;
        let mut rows: Vec<String> = Vec::new();
        for _ in 0..data.len() {
            let values = T::columns()
                .iter()
                .map(|_| {
                    count += 1;
                    format!("${}", count)
                })
                .collect::<Vec<String>>()
                .join(", ");
            rows.push(format!("({values})"));
        }
        let placeholders = rows.join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} RETURNING id",
            self.table, columns, placeholders
        );

        let query = sqlx::query_as::<_, IdResult>(&sql);
        let query = data.into_iter().fold(query, |q, d| d.bind_query(q));

        let rows: Vec<IdResult> = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }
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

        let row: IdResult = data.bind_query(query).fetch_one(&self.pool).await?;
        Ok(row.id)
    }

    async fn upsert(&self, data: T) -> Result<String, Error> {
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
        let conflict_vals = T::uniques().join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET id = excluded.id RETURNING id",
            self.table, columns, values, conflict_vals
        );

        let query = sqlx::query_as::<_, IdResult>(&sql);

        let row: IdResult = data.bind_query(query).fetch_one(&self.pool).await?;
        Ok(row.id)
    }
    async fn upsert_many(&self, data: Vec<T>) -> Result<Vec<String>, Error> {
        let columns: String = T::columns()
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<String>>()
            .join(", ");
        let conflict_vals = T::uniques().join(", ");

        let mut count = 0;
        let mut rows: Vec<String> = Vec::new();
        for _ in 0..data.len() {
            let row = T::columns()
                .iter()
                .map(|_| {
                    count += 1;
                    format!("${}", count)
                })
                .collect::<Vec<String>>()
                .join(", ");
            rows.push(format!("({row})"));
        }
        let placeholders = rows.join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} ON CONFLICT ({}) DO UPDATE SET id = excluded.id RETURNING id",
            self.table, columns, placeholders, conflict_vals
        );

        let query = sqlx::query_as::<_, IdResult>(&sql);
        let query = data.into_iter().fold(query, |q, d| d.bind_query(q));

        let rows: Vec<IdResult> = query.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.id).collect())
    }

    async fn find_all(&self, filter: QueryBuilder) -> Result<Vec<T>, Error> {
        let (where_clause, values) = filter.into_sqlite_filter();

        let sql = format!("SELECT * FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in values {
            query = v.bind_sqlite(query);
        }

        query.fetch_all(&self.pool).await.map_err(Into::into)
    }

    async fn find_one(&self, query: QueryBuilder) -> Result<Option<T>, Error> {
        let (where_clause, values) = query.into_sqlite_filter();

        let sql = format!("SELECT * FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in values {
            query = v.bind_sqlite(query);
        }

        query.fetch_optional(&self.pool).await.map_err(Into::into)
    }

    async fn find_one_and_update(
        &self,
        filter: QueryBuilder,
        update: QueryBuilder,
    ) -> Result<Option<T>, Error> {
        let (where_clause, filter_values) = filter.into_sqlite_filter();
        let (set_clause, update_values) = update.into_sqlite_update();

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, set_clause, where_clause
        );
        let mut query = sqlx::query_as::<_, T>(&sql);

        for v in update_values.into_iter().chain(filter_values) {
            query = v.bind_sqlite(query);
        }

        query.fetch_optional(&self.pool).await.map_err(Into::into)
    }

    async fn update_many(&self, filter: QueryBuilder, update: QueryBuilder) -> Result<(), Error> {
        let (set_clause, update_values) = update.into_sqlite_update();
        let (where_clause, filter_values) = filter.into_sqlite_filter();

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, set_clause, where_clause
        );
        let mut query = sqlx::query_as::<_, T>(&sql);

        for v in update_values.into_iter().chain(filter_values) {
            query = v.bind_sqlite(query);
        }

        query.fetch_optional(&self.pool).await?;
        Ok(())
    }

    async fn delete_one(&self, filter: QueryBuilder) -> Result<(), Error> {
        let (where_clause, values) = filter.into_sqlite_filter();

        let sql = format!("DELETE FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in values {
            query = v.bind_sqlite(query);
        }

        query.fetch_optional(&self.pool).await?;
        Ok(())
    }
    async fn delete_many(&self, filter: QueryBuilder) -> Result<(), Error> {
        let (where_clause, values) = filter.into_sqlite_filter();

        let sql = format!("DELETE FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);
        for v in values {
            query = v.bind_sqlite(query);
        }

        query.fetch_optional(&self.pool).await?;
        Ok(())
    }
}
