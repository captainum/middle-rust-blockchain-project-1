use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseTxTypeError {
    #[error("Invalid TX_TYPE: {0}")]
    InvalidTxType(String),
}

#[derive(Debug, Error)]
pub enum ParseStatusError {
    #[error("Invalid STATUS: {0}")]
    InvalidStatus(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseKeyError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseValueError {
    #[error("Invalid value: {value} ({description})")]
    InvalidValue { value: String, description: String },
}

impl From<ParseTxTypeError> for ParseValueError {
    fn from(e: ParseTxTypeError) -> Self {
        match e {
            ParseTxTypeError::InvalidTxType(ref value) => ParseValueError::InvalidValue {
                value: value.clone(),
                description: e.to_string(),
            },
        }
    }
}

impl From<ParseStatusError> for ParseValueError {
    fn from(e: ParseStatusError) -> Self {
        match e {
            ParseStatusError::InvalidStatus(ref value) => ParseValueError::InvalidValue {
                value: value.clone(),
                description: e.to_string(),
            },
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromTxtError {
    #[error("Colon after key={0} not found")]
    ColonNotFound(String),
    #[error("Unexpected key found: {0}")]
    InvalidKey(ParseKeyError),
    #[error("Missing key {0}")]
    MissingKey(String),
    #[error("{0}")]
    InvalidValue(ParseValueError),
    #[error("Unexpected error. line={line}, description={description}")]
    UnexpectedError { line: String, description: String },
    #[error("Unexpected error: {0}")]
    UnexpectedError1(String),
}

impl From<ParseKeyError> for ParseRecordFromTxtError {
    fn from(e: ParseKeyError) -> Self {
        ParseRecordFromTxtError::InvalidKey(e)
    }
}

impl From<ParseValueError> for ParseRecordFromTxtError {
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromTxtError::InvalidValue(e)
    }
}

impl From<std::io::Error> for ParseRecordFromTxtError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError1(e.to_string())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromCsvError {
    #[error("Invalid count of columns: {0}")]
    InvalidCountOfColumns(usize),
    #[error("{0}")]
    InvalidValue(ParseValueError),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<ParseValueError> for ParseRecordFromCsvError {
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromCsvError::InvalidValue(e)
    }
}

impl From<std::io::Error> for ParseRecordFromCsvError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromBinError {
    #[error("Invalid magic number")]
    InvalidMagicNumber,
    #[error("Invalid body length: {0}")]
    InvalidBodyLength(u32),
    #[error("{0}")]
    InvalidValue(ParseValueError),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<ParseValueError> for ParseRecordFromBinError {
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromBinError::InvalidValue(e)
    }
}

impl From<std::io::Error> for ParseRecordFromBinError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum WriteRecordError {
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<std::io::Error> for WriteRecordError {
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}
