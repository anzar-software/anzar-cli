use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use super::traits::{IdResult, PgInsert, SqliteInsert};
use crate::scopes::user::UserRole;

impl PgInsert for UserRole {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "roleId"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query.bind(self.user_id).bind(self.role_id)
    }
}

impl SqliteInsert for UserRole {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "roleId"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query.bind(self.user_id).bind(self.role_id)
    }
}
