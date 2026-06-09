use shared::error::CoreError;
use sqlx::migrate::MigrateError;
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for the CLI.
///
/// Add variants here as your app grows. Keep each variant as specific as
/// possible so callers can pattern-match on meaningful categories.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] CoreError),

    // ── I/O ──────────────────────────────────────────────────────────────────
    /// Wraps `std::io::Error` and auto-converts via `?` thanks to `#[from]`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found, with the path included in the message.
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    // ── Configuration ────────────────────────────────────────────────────────
    /// A required config key was missing.
    #[error("missing config key: `{key}`")]
    _MissingConfig { key: String },

    /// A config value was present but invalid.
    #[error("invalid value for `{key}`: {reason}")]
    InvalidConfig { key: String, reason: String },

    /// CLI argument that failed to parse.
    #[error("invalid argument `{arg}`: {reason}")]
    _InvalidArg { arg: String, reason: String },

    #[error("Validation failed")]
    _InvalidAppConfig(Vec<Error>),

    // ── Parsing / Serialization ───────────────────────────────────────────────
    /// JSON parse failure — `#[transparent]` delegates Display and Source
    /// directly to the inner error, so the user sees serde's own message.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Sqlx(#[from] MigrateError),

    // ── Network / HTTP ────────────────────────────────────────────────────────
    /// HTTP request failure (e.g. reqwest). `#[source]` wires up
    /// `Error::source()` without making it transparent.
    #[error("HTTP request failed: {message}")]
    _Http {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// Non-2xx response with status code preserved.
    #[error("server returned {status}: {body}")]
    _HttpStatus { status: u16, body: String },

    // ── Process / Subprocess ─────────────────────────────────────────────────
    /// An external command exited with a non-zero code.
    #[error("command `{cmd}` failed with exit code {code}")]
    _CommandFailed { cmd: String, code: i32 },

    // ── Catch-all ────────────────────────────────────────────────────────────
    /// Escape hatch for one-off errors that don't warrant a dedicated variant.
    /// Prefer adding a real variant over using this in production paths.
    #[error("{0}")]
    Other(String),
}

// ── Convenient constructors ───────────────────────────────────────────────────

// impl Error {
//     pub fn missing_config(key: impl Into<String>) -> Self {
//         Self::MissingConfig { key: key.into() }
//     }
//
//     pub fn invalid_config(key: impl Into<String>, reason: impl Into<String>) -> Self {
//         Self::InvalidConfig {
//             key: key.into(),
//             reason: reason.into(),
//         }
//     }
//
//     pub fn invalid_arg(arg: impl Into<String>, reason: impl Into<String>) -> Self {
//         Self::InvalidArg {
//             arg: arg.into(),
//             reason: reason.into(),
//         }
//     }
//
//     pub fn http(
//         message: impl Into<String>,
//         source: impl std::error::Error + Send + Sync + 'static,
//     ) -> Self {
//         Self::Http {
//             message: message.into(),
//             source: Box::new(source),
//         }
//     }
//
//     pub fn other(msg: impl Into<String>) -> Self {
//         Self::Other(msg.into())
//     }
// }

// ── Result alias ─────────────────────────────────────────────────────────────

/// Shorthand so callers can write `Result<T>` instead of `Result<T, CliError>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;
