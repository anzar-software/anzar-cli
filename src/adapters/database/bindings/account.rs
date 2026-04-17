use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use super::traits::{IdResult, PgInsert, SqliteInsert};
use crate::services::account::model::Account;

impl PgInsert for Account {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "password", "locked", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.user_id)
            .bind(self.password)
            .bind(self.locked)
            .bind(self.created_at)
    }
}

impl SqliteInsert for Account {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "password", "locked", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.user_id)
            .bind(self.password)
            .bind(self.locked)
            .bind(self.created_at)
    }
}
