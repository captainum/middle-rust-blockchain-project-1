use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxTypeParseError {
    #[error("Invalid tx type: {0}")]
    InvalidTxType(String),
}

#[derive(Debug, Error)]
pub enum StatusParseError {
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum FromLinesError {
    #[error("Colon after key={0} not found")]
    ColonNotFound(String),
    #[error("Incorrect value format ({description})")]
    IncorrectValueFormat { description: String },
    #[error("Unexpected key found: {0}")]
    UnexpectedKeyFound(String),
    #[error("Missing key {0}")]
    MissingKey(String),
    #[error("Unexpected error. line={line}, description={description}")]
    UnexpectedError { line: String, description: String },
}

impl From<TxTypeParseError> for FromLinesError {
    fn from(e: TxTypeParseError) -> Self {
        FromLinesError::IncorrectValueFormat {
            description: e.to_string(),
        }
    }
}

impl From<StatusParseError> for FromLinesError {
    fn from(e: StatusParseError) -> Self {
        FromLinesError::IncorrectValueFormat {
            description: e.to_string(),
        }
    }
}
