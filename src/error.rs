use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShieldSyncError {
    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid key or password")]
    InvalidKey,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Sync conflict: {0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, ShieldSyncError>;