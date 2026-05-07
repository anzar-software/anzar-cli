use super::MongodbIndexes;
use crate::error::{Error, InternalError};

const NO_CLIENT: &str = "No available server found for connection string '{}'. Please verify that the connection string is valid and the server is reachable.";
const NO_DB: &str = "Failed to get default database: MongoDB client has no default database configured. Ensure 'default_database' is set in the connection string or client options.";

pub struct MongoDB {}
impl MongoDB {
    pub async fn start(connection_string: &str) -> Result<mongodb::Client, Error> {
        let client = mongodb::Client::with_uri_str(&connection_string)
            .await
            .map_err(|e| {
                tracing::error!(
                    error_code = "InternalError::Database",
                    "Failed to connect to database - {e}"
                );
                Error::Internal(InternalError::Database(
                    NO_CLIENT.replace("{}", connection_string),
                ))
            })?;

        let db = client
            .default_database()
            .ok_or_else(|| Error::Internal(InternalError::Database(NO_DB.into())))?;

        let mongodb_indexes = MongodbIndexes { db: db.clone() };
        mongodb_indexes
            .create_unique_email_index()
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error_code = "InternalError::Database",
                    "Failed to create index for email attribute in users Collection - {e}"
                );
            })?;

        mongodb_indexes
            .create_token_hash_index()
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error_code = "InternalError::Database",
                    "Failed to create index for email attribute in password_reset_tokens Collection - {e}"
                );
            })?;

        Ok(client)
    }
}
