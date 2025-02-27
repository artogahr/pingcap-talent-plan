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
    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),
    #[error("Box<ErrorKind>")]
    BoxedError(#[from] Box<dyn std::error::Error>),
}

/// Type alias
pub type Result<T> = std::result::Result<T, CustomError>;
