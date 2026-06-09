use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::Permission;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for Permission {
    fn columns() -> Vec<&'static str> {
        vec!["name", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["name"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query.bind(self.name).bind(self.created_at)
    }
}

impl SqliteInsert for Permission {
    fn columns() -> Vec<&'static str> {
        vec!["name", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["name"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query.bind(self.name).bind(self.created_at)
    }
}

impl MongoInsert for Permission {
    fn columns() -> Vec<&'static str> {
        vec!["name", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["name"]
    }
}
