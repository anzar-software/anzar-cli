use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::SigningKey;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for SigningKey {
    fn columns() -> Vec<&'static str> {
        vec![
            "status",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
            "expiresAt",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["encrypted_private_key", "kid"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.status)
            .bind(self.encrypted_private_key)
            .bind(self.public_key)
            .bind(self.algorithm)
            .bind(self.kid)
            .bind(self.kty)
            .bind(self.created_at)
            .bind(self.rotated_at)
            .bind(self.expires_at)
    }
}

impl SqliteInsert for SigningKey {
    fn columns() -> Vec<&'static str> {
        vec![
            "status",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
            "expiresAt",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["encrypted_private_key", "kid"]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.status)
            .bind(self.encrypted_private_key)
            .bind(self.public_key)
            .bind(self.algorithm)
            .bind(self.kid)
            .bind(self.kty)
            .bind(self.created_at)
            .bind(self.rotated_at)
            .bind(self.expires_at)
    }
}

impl MongoInsert for SigningKey {
    fn columns() -> Vec<&'static str> {
        vec![
            "status",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
            "expiresAt",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["encrypted_private_key", "kid"]
    }
}
