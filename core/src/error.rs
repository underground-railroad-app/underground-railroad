//! Error types for the Underground Railroad core library

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types
#[derive(Error, Debug)]
pub enum Error {
    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Veilid network error
    #[error("Network error: {0}")]
    Network(String),

    /// Identity/authentication error
    #[error("Identity error: {0}")]
    Identity(String),

    /// Trust verification failed
    #[error("Trust error: {0}")]
    Trust(String),

    /// Invalid input or configuration
    #[error("Invalid: {0}")]
    Invalid(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation timed out
    #[error("Timeout: {0}")]
    Timeout(String),

    /// General I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Internal error (should not happen)
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<veilid_core::VeilidAPIError> for Error {
    fn from(e: veilid_core::VeilidAPIError) -> Self {
        Error::Network(format!("Veilid error: {}", e))
    }
}
