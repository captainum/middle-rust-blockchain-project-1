use super::record::errors::{
    ParseRecordFromBinError, ParseRecordFromCsvError, ParseRecordFromTxtError, WriteRecordError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<ParseRecordFromTxtError> for ReadError {
    fn from(e: ParseRecordFromTxtError) -> Self {
        Self::InvalidFormat(e.to_string())
    }
}

impl From<ParseRecordFromCsvError> for ReadError {
    fn from(e: ParseRecordFromCsvError) -> Self {
        Self::InvalidFormat(e.to_string())
    }
}

impl From<ParseRecordFromBinError> for ReadError {
    fn from(e: ParseRecordFromBinError) -> Self {
        Self::InvalidFormat(e.to_string())
    }
}

impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Write record error: {0}")]
    WriteRecordError(WriteRecordError),
    #[error("Write header error: {0}")]
    WriteHeaderError(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<WriteRecordError> for WriteError {
    fn from(e: WriteRecordError) -> Self {
        Self::WriteRecordError(e)
    }
}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}
