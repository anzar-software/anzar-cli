use sqlx::{Postgres, Sqlite, postgres::PgArguments, query::QueryAs, sqlite::SqliteArguments};

use crate::domain::model::RolePermission;

use super::traits::{IdResult, MongoInsert, PgInsert, SqliteInsert};

impl PgInsert for RolePermission {
    fn columns() -> Vec<&'static str> {
        vec!["permissionId", "roleId", "issuedAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["\"permissionId\"", "\"roleId\""]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Postgres, IdResult, PgArguments>,
    ) -> QueryAs<'q, Postgres, IdResult, PgArguments> {
        query
            .bind(self.permission_id)
            .bind(self.role_id)
            .bind(self.issued_at)
    }
}

impl SqliteInsert for RolePermission {
    fn columns() -> Vec<&'static str> {
        vec!["permissionId", "roleId", "issuedAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["\"permissionId\"", "\"roleId\""]
    }

    fn bind_query<'q>(
        self,
        query: QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>>,
    ) -> QueryAs<'q, Sqlite, IdResult, SqliteArguments<'q>> {
        query
            .bind(self.permission_id)
            .bind(self.role_id)
            .bind(self.issued_at)
    }
}

impl MongoInsert for RolePermission {
    fn columns() -> Vec<&'static str> {
        vec!["permissionId", "roleId", "issuedAt"]
    }
    fn uniques() -> Vec<&'static str> {
        vec!["\"permissionId\"", "\"roleId\""]
    }
}
