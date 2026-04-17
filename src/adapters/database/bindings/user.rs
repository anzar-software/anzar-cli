use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use super::traits::{IdResult, PgInsert, SqliteInsert};
use crate::scopes::user::User;

impl PgInsert for User {
    fn columns() -> Vec<&'static str> {
        vec!["username", "email", "role", "verified", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.username)
            .bind(self.email)
            .bind(self.role)
            .bind(self.verified)
            .bind(self.created_at)
    }
}

impl SqliteInsert for User {
    fn columns() -> Vec<&'static str> {
        vec!["username", "email", "role", "verified", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.username)
            .bind(self.email)
            .bind(self.role)
            .bind(self.verified)
            .bind(self.created_at)
    }
}
