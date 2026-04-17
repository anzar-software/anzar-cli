use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use super::traits::{IdResult, PgInsert, SqliteInsert};
use crate::scopes::email::model::EmailVerificationToken;

impl PgInsert for EmailVerificationToken {
    fn columns() -> Vec<&'static str> {
        vec![
            "userId",
            "issuedAt",
            "expiresAt",
            "usedAt",
            "token",
            "valid",
        ]
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
            .bind(self.valid)
    }
}

impl SqliteInsert for EmailVerificationToken {
    fn columns() -> Vec<&'static str> {
        vec![
            "userId",
            "issuedAt",
            "expiresAt",
            "usedAt",
            "token",
            "valid",
        ]
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
            .bind(self.valid)
    }
}
