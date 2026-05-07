use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use super::traits::{IdResult, PgInsert, SqliteInsert};
use crate::domain::model::PasswordResetToken;

impl PgInsert for PasswordResetToken {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "issuedAt", "expiresAt", "usedAt", "token"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.user_id)
            .bind(self.issued_at)
            .bind(self.expires_at)
            .bind(self.used_at)
            .bind(self.token)
    }
}

impl SqliteInsert for PasswordResetToken {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "issuedAt", "expiresAt", "usedAt", "token"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.user_id)
            .bind(self.issued_at)
            .bind(self.expires_at)
            .bind(self.used_at)
            .bind(self.token)
    }
}
