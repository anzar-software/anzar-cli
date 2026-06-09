use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::Session;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for Session {
    fn columns() -> Vec<&'static str> {
        vec![
            "userId",
            "issuedAt",
            "expiresAt",
            "usedAt",
            "token",
            "roles",
            "permissions",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["token"]
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
            .bind(self.roles)
            .bind(self.permissions)
    }
}

impl SqliteInsert for Session {
    fn columns() -> Vec<&'static str> {
        vec![
            "userId",
            "issuedAt",
            "expiresAt",
            "usedAt",
            "token",
            "roles",
            "permissions",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["token"]
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
            .bind(serde_json::to_string(&self.roles).unwrap())
            .bind(serde_json::to_string(&self.permissions).unwrap())
    }
}

impl MongoInsert for Session {
    fn columns() -> Vec<&'static str> {
        vec![
            "userId",
            "issuedAt",
            "expiresAt",
            "usedAt",
            "token",
            "roles",
            "permissions",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["token"]
    }
}
