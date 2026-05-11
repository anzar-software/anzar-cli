use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::User;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for User {
    fn columns() -> Vec<&'static str> {
        vec!["username", "email", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["email"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.username)
            .bind(self.email)
            .bind(self.created_at)
    }
}

impl SqliteInsert for User {
    fn columns() -> Vec<&'static str> {
        vec!["username", "email", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["email"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.username)
            .bind(self.email)
            .bind(self.created_at)
    }
}

impl MongoInsert for User {
    fn columns() -> Vec<&'static str> {
        vec!["username", "email", "createdAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["email"]
    }
}
