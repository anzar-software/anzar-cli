use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

#[derive(sqlx::FromRow)]
pub struct IdResult {
    pub id: String,
}

pub trait PgInsert: Send + Sync {
    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments>;

    fn columns() -> Vec<&'static str>;
    fn uniques() -> Vec<&'static str>;
}

pub trait SqliteInsert: Send + Sync {
    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>;

    fn columns() -> Vec<&'static str>;
    fn uniques() -> Vec<&'static str>;
}

pub trait MongoInsert: Send + Sync {
    fn columns() -> Vec<&'static str>;
    fn uniques() -> Vec<&'static str>;
}
