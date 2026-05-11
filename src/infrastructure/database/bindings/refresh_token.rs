use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::RefreshToken;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for RefreshToken {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "issuedAt", "expiresAt", "usedAt", "jti"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["jti"]
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
            .bind(self.jti)
    }
}

impl SqliteInsert for RefreshToken {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "issuedAt", "expiresAt", "usedAt", "jti"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["jti"]
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
            .bind(self.jti)
    }
}

impl MongoInsert for RefreshToken {
    fn columns() -> Vec<&'static str> {
        vec!["userId", "issuedAt", "expiresAt", "usedAt", "jti"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["jti"]
    }
}
