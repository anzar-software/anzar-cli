use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::Account;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for Account {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "password", "locked", "verified", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["password"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.user_id)
            .bind(self.password)
            .bind(self.locked)
            .bind(self.verified)
            .bind(self.created_at)
    }
}

impl SqliteInsert for Account {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "password", "locked", "verified", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["password"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.user_id)
            .bind(self.password)
            .bind(self.locked)
            .bind(self.verified)
            .bind(self.created_at)
    }
}

impl MongoInsert for Account {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "password", "locked", "verified", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["password"]
    }
}
