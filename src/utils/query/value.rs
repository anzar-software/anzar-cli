use mongodb::bson::Bson;

use crate::scopes::user::Role;

#[derive(Clone)]
pub struct Operation {
    pub field: &'static str,
    pub op: Op,
    pub value: DbValue,
}

#[derive(Clone)]
pub enum Op {
    Eq,
    Ne,
    Gt,
    Lt,
    In,
    Null,
    // Update ops
    Set,
    Inc,
    Unset,
}

#[derive(Clone, Debug)]
pub enum DbValue {
    String(String),
    Role(Role),
    Int(i64),
    Float(f64),
    Uuid(uuid::Uuid),
    Bool(bool),
    Date(chrono::DateTime<chrono::Utc>),
    Null,
    List(Vec<DbValue>),
}

impl From<&str> for DbValue {
    fn from(value: &str) -> Self {
        DbValue::String(value.to_string())
    }
}
impl From<String> for DbValue {
    fn from(value: String) -> Self {
        DbValue::String(value)
    }
}
impl From<Role> for DbValue {
    fn from(value: Role) -> Self {
        DbValue::Role(value)
    }
}
impl From<bool> for DbValue {
    fn from(b: bool) -> Self {
        DbValue::Bool(b)
    }
}

impl From<i64> for DbValue {
    fn from(i: i64) -> Self {
        DbValue::Int(i)
    }
}

impl From<f64> for DbValue {
    fn from(f: f64) -> Self {
        DbValue::Float(f)
    }
}
impl From<uuid::Uuid> for DbValue {
    fn from(u: uuid::Uuid) -> Self {
        DbValue::Uuid(u)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DbValue {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        DbValue::Date(dt)
    }
}
impl From<Option<String>> for DbValue {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(s) => DbValue::String(s),
            None => DbValue::Null,
        }
    }
}

impl DbValue {
    pub fn bind_pg<'q, T>(
        self,
        query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments> {
        match self {
            DbValue::String(s) => query.bind(s),
            DbValue::Int(i) => query.bind(i),
            DbValue::Float(f) => query.bind(f),
            DbValue::Bool(b) => query.bind(b),
            DbValue::Date(dt) => query.bind(dt),
            DbValue::Null => query.bind::<Option<String>>(None),
            DbValue::Role(r) => query.bind(r),
            DbValue::Uuid(uuid) => query.bind(uuid),
            DbValue::List(_) => todo!(),
        }
    }

    pub fn bind_sqlite<'q, T>(
        self,
        query: sqlx::query::QueryAs<'q, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'q>>,
    ) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, T, sqlx::sqlite::SqliteArguments<'q>> {
        match self {
            DbValue::String(s) => query.bind::<String>(s),
            DbValue::Int(i) => query.bind::<i64>(i),
            DbValue::Float(f) => query.bind::<f64>(f),
            DbValue::Bool(b) => query.bind::<bool>(b),
            DbValue::Date(dt) => query.bind(dt),
            DbValue::Null => query.bind::<Option<String>>(None),
            DbValue::Role(r) => query.bind(r),
            DbValue::Uuid(uuid) => query.bind(uuid),
            DbValue::List(_db_values) => todo!(),
        }
    }

    pub fn into_boson(self, field: &'static str) -> Bson {
        const IDS: [&str; 4] = ["id", "_id", "user_id", "userId"];

        match self {
            DbValue::String(s) => {
                if IDS.contains(&field) {
                    match mongodb::bson::oid::ObjectId::parse_str(&s) {
                        Ok(oid) => Bson::ObjectId(oid),
                        Err(_) => Bson::String(s),
                    }
                } else {
                    Bson::String(s)
                }
            }
            DbValue::Role(r) => Bson::String(format!("{:?}", r)),
            DbValue::Int(i) => Bson::Int64(i),
            DbValue::Float(f) => Bson::Double(f),
            DbValue::Uuid(u) => Bson::String(u.to_string()),
            DbValue::Bool(b) => Bson::Boolean(b),
            DbValue::Date(dt) => {
                Bson::DateTime(mongodb::bson::DateTime::from_millis(dt.timestamp_millis()))
            }
            DbValue::Null => Bson::Null,
            DbValue::List(items) => {
                Bson::Array(items.into_iter().map(|v| v.into_boson(field)).collect())
            }
        }
    }
}
