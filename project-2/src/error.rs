use thiserror::Error;

/// Custom error type
#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Some error occurred")]
    Io(#[from] std::io::Error),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Serde error")]
    Serde(#[from] serde_json::Error),
}

/// Type alias
pub type Result<T> = std::result::Result<T, CustomError>;