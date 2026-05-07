use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::Role;

use super::traits::{IdResult, PgInsert, SqliteInsert};

impl PgInsert for Role {
    fn columns() -> Vec<&'static str> {
        vec!["name", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query.bind(self.name).bind(self.created_at)
    }
}

impl SqliteInsert for Role {
    fn columns() -> Vec<&'static str> {
        vec!["name", "createdAt"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query.bind(self.name).bind(self.created_at)
    }
}
