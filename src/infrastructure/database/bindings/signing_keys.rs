use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::SigningKeys;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for SigningKeys {
    fn columns() -> Vec<&'static str> {
        vec![
            "active",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
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
            .bind(self.active)
            .bind(self.encrypted_private_key)
            .bind(self.public_key)
            .bind(self.algorithm)
            .bind(self.kid)
            .bind(self.kty)
            .bind(self.created_at)
            .bind(self.rotated_at)
    }
}

impl SqliteInsert for SigningKeys {
    fn columns() -> Vec<&'static str> {
        vec![
            "active",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
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
            .bind(self.active)
            .bind(self.encrypted_private_key)
            .bind(self.public_key)
            .bind(self.algorithm)
            .bind(self.kid)
            .bind(self.kty)
            .bind(self.created_at)
            .bind(self.rotated_at)
    }
}

impl MongoInsert for SigningKeys {
    fn columns() -> Vec<&'static str> {
        vec![
            "active",
            "encrypted_private_key",
            "public_key",
            "algorithm",
            "kid",
            "kty",
            "createdAt",
            "rotatedAt",
        ]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["encrypted_private_key", "kid"]
    }
}
