use super::record::errors::{
    ParseRecordFromBinError, ParseRecordFromCsvError, ParseRecordFromTxtError,
};
use thiserror::Error;

/// Ошибка чтения данных из источника.
#[derive(Debug, Error)]
pub enum ReadError {
    /// Ошибка чтения данных из текстового источника.
    #[error("Text format parsing error: {0}")]
    FromText(#[from] ParseRecordFromTxtError),

    /// Ошибка чтения данных из CSV источника.
    #[error("CSV format parsing error: {0}")]
    FromCsv(#[from] ParseRecordFromCsvError),

    /// Ошибка чтения данных из бинарного источника.
    #[error("Binary format parsing error: {0}")]
    FromBin(#[from] ParseRecordFromBinError),

    /// Ошибка чтения данных, не связанная с его типом.
    #[error("Read data error: {0}")]
    Io(#[from] std::io::Error),
}

/// Ошибка записи данных.
#[derive(Debug, Error)]
pub enum WriteError {
    /// Ошибка записи заголовка в CSV формате.
    #[error("Write header error: {0}")]
    WriteHeaderError(String),

    /// Непредвиденная ошибка записи данных.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    /// Ошибка записи данных, не связанная с его типом.
    #[error("Read data error: {0}")]
    Io(#[from] std::io::Error),
}
