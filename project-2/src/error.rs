use thiserror::Error;

/// Custom error type
#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Some error occurred")]
    Io(#[from] std::io::Error),
    #[error("Key not found")]
    KeyNotFound,
}

/// Type alias
pub type Result<T> = std::result::Result<T, CustomError>
;