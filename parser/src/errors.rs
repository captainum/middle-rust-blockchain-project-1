use thiserror::Error;
use super::record::errors::FromLinesError;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<FromLinesError> for ReadError {
    fn from(e: FromLinesError) -> Self {
        Self::InvalidFormat(e.to_string())
    }
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}