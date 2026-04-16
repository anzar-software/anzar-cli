use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use sqlx::{FromRow, Pool, Postgres};

use crate::adapters::database::DatabaseAdapter;
use crate::error::Error;
use crate::scopes::user::Role;

#[derive(sqlx::FromRow)]
struct IdResult {
    id: String,
}

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
        + Unpin,
{
    async fn insert(&self, data: T) -> Result<String, Error> {
        let value = serde_json::to_value(data)?;
        let obj = value.as_object().unwrap();

        let columns: String = obj
            .keys()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<String>>()
            .join(", ");
        let values = obj
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
            self.table, columns, values
        );

        let mut query = sqlx::query_as::<_, IdResult>(&sql);

        for (_, v) in obj.iter() {
            query = _bind_value(query, v.to_owned());
        }

        let row: IdResult = query.fetch_one(&self.pool).await.map_err(|e| {
            dbg!(&e);
            Error::DatabaseError(e.to_string())
        })?;

        Ok(row.id)
    }

    async fn find_one(&self, filter: Value) -> Result<Option<T>, Error> {
        let obj = _parse_to_map(filter)?;
        let where_clause = _parse_to_sql(&obj, " AND ", 0);

        let sql = format!("SELECT * FROM {} WHERE {}", self.table, where_clause);
        let mut query = sqlx::query_as::<_, T>(&sql);

        for (_, v) in obj.iter() {
            query = _bind_value(query, v.to_owned());
        }

        query.fetch_optional(&self.pool).await.map_err(|e| {
            dbg!(&e);
            Error::DatabaseError(e.to_string())
        })
    }

    async fn find_one_and_update(&self, filter: Value, update: Value) -> Result<Option<T>, Error> {
        let obj_update = _parse_to_map(update)?;
        let clause_update = _parse_to_sql(&obj_update, ", ", 0);

        let obj_filter = _parse_to_map(filter)?;
        let clause_filter = _parse_to_sql(&obj_filter, " AND ", obj_update.len());

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, clause_update, clause_filter
        );
        let mut query = sqlx::query_as::<_, T>(&sql);

        for (_, v) in obj_update.iter() {
            query = _bind_value(query, v.to_owned());
        }
        for (_, v) in obj_filter.iter() {
            query = _bind_value(query, v.to_owned());
        }

        query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn update_many(&self, filter: Value, update: Value) -> Result<(), Error> {
        let obj_update = _parse_to_map(update)?;
        let clause_update = _parse_to_sql(&obj_update, ", ", 0);

        let obj_filter = _parse_to_map(filter)?;
        let clause_filter = _parse_to_sql(&obj_filter, " AND ", obj_update.len());

        let sql = format!(
            "UPDATE {} SET {} WHERE {} RETURNING *",
            self.table, clause_update, clause_filter
        );
        let mut query = sqlx::query_as::<_, T>(&sql);

        for (_, v) in obj_update.iter() {
            query = _bind_value(query, v.to_owned());
        }
        for (_, v) in obj_filter.iter() {
            query = _bind_value(query, v.to_owned());
        }

        query
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_one(&self, _filter: Value) -> Result<(), Error> {
        Ok(())
    }
    async fn delete_many(&self, _filter: Value) -> Result<(), Error> {
        Ok(())
    }
}

fn _parse_to_map(data: Value) -> Result<Map<String, Value>, Error> {
    let value = serde_json::to_value(data)?;
    let obj = value.as_object().unwrap();
    if obj.is_empty() {
        return Err(Error::InternalServerError("parsing error".into()));
    }

    Ok(obj.to_owned())
}

fn _parse_to_sql(obj: &Map<String, Value>, join: &str, offset: usize) -> String {
    let conditions: Vec<String> = obj
        .keys()
        .enumerate()
        .map(|(i, k)| format!("\"{}\" = ${}", k, i + 1 + offset))
        .collect();
    conditions.join(join)
}

fn _bind_value<T>(
    query: QueryAs<Postgres, T, PgArguments>,
    v: Value,
) -> QueryAs<Postgres, T, PgArguments> {
    match v {
        serde_json::Value::String(s) => {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                query.bind(dt.with_timezone(&chrono::Utc))
            } else if let Ok(role) = serde_json::from_value::<Role>(Value::String(s.clone())) {
                query.bind(role)
            } else {
                query.bind(s)
            }
        }
        serde_json::Value::Number(n) if n.is_i64() => query.bind(n.as_i64()),
        serde_json::Value::Number(n) if n.is_f64() => query.bind(n.as_f64()),
        serde_json::Value::Bool(b) => query.bind(b),
        // serde_json::Value::Null => query.bind::<Option<String>>(None),
        serde_json::Value::Null => query.bind::<Option<chrono::DateTime<chrono::Utc>>>(None),
        _ => query.bind(v.to_string()),
    }
}
