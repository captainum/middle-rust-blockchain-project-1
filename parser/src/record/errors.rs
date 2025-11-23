//! Модуль описания ошибок парсинга ключа поля, значения поля (включая возможный тип, состояние),
//! а также записи целиком о транзакции.

use thiserror::Error;

/// Ошибка парсинга типа транзакции.
#[derive(Debug, Error)]
pub enum ParseTxTypeError {
    /// Некорректное значение поля типа транзакции TX_TYPE.
    #[error("Invalid TX_TYPE: {0}")]
    InvalidTxType(String),
}

/// Ошибка парсинга состояния транзакции.
#[derive(Debug, Error)]
pub enum ParseStatusError {
    /// Некорректное значение поля состояния транзакции STATUS.
    #[error("Invalid STATUS: {0}")]
    InvalidStatus(String),
}

/// Ошибка парсинга ключа поля транзакции.
#[derive(Debug, Error, PartialEq)]
pub enum ParseKeyError {
    /// Некорректное значение ключа транзакции.
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

/// Ошибка парсинга значения поля транзакции.
#[derive(Debug, Error, PartialEq)]
pub enum ParseValueError {
    /// Некорректное значение поля транзакции.
    #[error("Invalid value: {value} ({description})")]
    InvalidValue {
        /// Полученное значение.
        value: String,

        /// Описание ошибки.
        description: String,
    },
}

/// Реализация трейта [`From<ParseTxTypeError>`] для [`ParseValueError`].
impl From<ParseTxTypeError> for ParseValueError {
    /// Реализация метода [`From<ParseTxTypeError>::from`] для [`ParseValueError`].
    fn from(e: ParseTxTypeError) -> Self {
        match e {
            ParseTxTypeError::InvalidTxType(ref value) => ParseValueError::InvalidValue {
                value: value.clone(),
                description: e.to_string(),
            },
        }
    }
}

/// Реализация трейта [`From<ParseStatusError>`] для [`ParseValueError`].
impl From<ParseStatusError> for ParseValueError {
    /// Реализация метода [`From<ParseStatusError>::from`] для [`ParseValueError`].
    fn from(e: ParseStatusError) -> Self {
        match e {
            ParseStatusError::InvalidStatus(ref value) => ParseValueError::InvalidValue {
                value: value.clone(),
                description: e.to_string(),
            },
        }
    }
}

/// Ошибка парсинга текстового представления операции.
#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromTxtError {
    /// Не найден символ ':', разделяющий ключ и значение поля.
    #[error("Colon after key={0} not found")]
    ColonNotFound(String),

    /// Некорректное значение ключа записи.
    #[error("{0}")]
    InvalidKey(ParseKeyError),

    /// Не найден ожидаемый ключ записи.
    #[error("Missing key: {0}")]
    MissingKey(String),

    /// Некорректное значение поля записи.
    #[error("{0}")]
    InvalidValue(ParseValueError),

    /// Неожиданная ошибка парсинга данных.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Реализация трейта [`From<ParseKeyError>`] для [`ParseRecordFromTxtError`].
impl From<ParseKeyError> for ParseRecordFromTxtError {
    /// Реализация метода [`From<ParseKeyError>::from`] для [`ParseRecordFromTxtError`].
    fn from(e: ParseKeyError) -> Self {
        ParseRecordFromTxtError::InvalidKey(e)
    }
}

/// Реализация трейта [`From<ParseValueError>`] для [`ParseRecordFromTxtError`].
impl From<ParseValueError> for ParseRecordFromTxtError {
    /// Реализация метода [`From<ParseValueError>::from`] для [`ParseRecordFromTxtError`].
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromTxtError::InvalidValue(e)
    }
}

/// Реализация трейта [`From<std::io::Error>`] для [`ParseRecordFromTxtError`].
impl From<std::io::Error> for ParseRecordFromTxtError {
    /// Реализация метода [`From<std::io::Error>::from`] для [`ParseRecordFromTxtError`].
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}

/// Ошибка парсинга табличного представления операции.
#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromCsvError {
    /// Некорректное количество ожидаемых полей в записи.
    #[error("Invalid count of columns: {0}")]
    InvalidCountOfColumns(usize),

    /// Некорректное значение поля записи.
    #[error("{0}")]
    InvalidValue(ParseValueError),

    /// Неожиданная ошибка парсинга данных.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Реализация трейта [`From<ParseValueError>`] для [`ParseRecordFromCsvError`].
impl From<ParseValueError> for ParseRecordFromCsvError {
    /// Реализация метода [`From<ParseValueError>::from`] для [`ParseRecordFromCsvError`].
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromCsvError::InvalidValue(e)
    }
}

/// Реализация трейта [`From<std::io::Error>`] для [`ParseRecordFromCsvError`].
impl From<std::io::Error> for ParseRecordFromCsvError {
    /// Реализация метода [`From<std::io::Error>::from`] для [`ParseRecordFromCsvError`].
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}

/// Ошибка парсинга бинарного представления операции.
#[derive(Debug, Error, PartialEq)]
pub enum ParseRecordFromBinError {
    /// Некорректное значение MAGIC_NUMBER, идентифицирующего заголовок записи.
    #[error("Invalid magic number")]
    InvalidMagicNumber,

    /// Некорректное значение размера тела записи.
    #[error("Invalid record size: {0}")]
    InvalidRecordSize(u32),

    /// Некорректное значение поля записи.
    #[error("{0}")]
    InvalidValue(ParseValueError),

    /// Неожиданная ошибка парсинга данных.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Реализация трейта [`From<ParseValueError>`] для [`ParseRecordFromBinError`].
impl From<ParseValueError> for ParseRecordFromBinError {
    /// Реализация метода [`From<ParseValueError>::from`] для [`ParseRecordFromBinError`].
    fn from(e: ParseValueError) -> Self {
        ParseRecordFromBinError::InvalidValue(e)
    }
}
/// Реализация трейта [`From<std::io::Error>`] для [`ParseRecordFromBinError`].
impl From<std::io::Error> for ParseRecordFromBinError {
    /// Реализация метода [`From<std::io::Error>::from`] для [`ParseRecordFromBinError`].
    fn from(e: std::io::Error) -> Self {
        Self::UnexpectedError(e.to_string())
    }
}
