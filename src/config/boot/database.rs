use secrecy::{ExposeSecret, SecretString};
use utoipa::ToSchema;

#[derive(
    Debug, Default, Clone, Copy, serde::Deserialize, serde::Serialize, Eq, PartialEq, ToSchema,
)]
pub enum DatabaseDriver {
    #[default]
    SQLite,
    PostgreSQL,
    MongoDB,
}

impl DatabaseDriver {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseDriver::SQLite => "sqlite",
            DatabaseDriver::MongoDB => "mongodb",
            DatabaseDriver::PostgreSQL => "postgresql",
        }
    }
    pub fn _is_sql(&self) -> bool {
        matches!(self, DatabaseDriver::SQLite | DatabaseDriver::PostgreSQL)
    }

    pub fn _is_nosql(&self) -> bool {
        matches!(self, DatabaseDriver::MongoDB)
    }
}
impl std::fmt::Display for DatabaseDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl TryFrom<String> for DatabaseDriver {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(Self::SQLite),
            "mongodb" => Ok(Self::MongoDB),
            "postgresql" => Ok(Self::PostgreSQL),
            other => Err(format!(
                "{} is not supported database. Use either `sqlite`, `postgresql` or `mongodb`",
                other
            )),
        }
    }
}

#[derive(Default, Debug, serde::Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretString,
    pub port: String,
    pub host: String,
    pub name: String,
    pub driver: DatabaseDriver,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        match self.driver {
            DatabaseDriver::MongoDB => {
                // mongodb+srv://<username>:<password>@<host>:<port>/<db_name>
                // test: mongodb://localhost:27017/dev
                // prod: mongodb://db:27017/production
                format!(
                    "mongodb://{}:{}@{}:{}/{}?retryWrites=false&authSource=admin",
                    self.username,
                    self.password.expose_secret(),
                    self.host,
                    self.port,
                    self.name
                )
            }
            DatabaseDriver::SQLite => self.name.to_string(),
            DatabaseDriver::PostgreSQL => {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username,
                    self.password.expose_secret(),
                    self.host,
                    self.port,
                    self.name
                )
            }
        }
    }

    pub fn is_sql(&self) -> bool {
        matches!(
            self.driver,
            DatabaseDriver::SQLite | DatabaseDriver::PostgreSQL
        )
    }

    pub fn is_nosql(&self) -> bool {
        matches!(self.driver, DatabaseDriver::MongoDB)
    }
}
