use thiserror::Error;

#[derive(Error, Debug)]
pub enum UndergroundError {
    #[error("Veilid error: {0}")]
    Veilid(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid key")]
    InvalidKey,

    #[error("Not initialized")]
    NotInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, UndergroundError>;
