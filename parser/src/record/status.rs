//! Модуль описания возможных состояний транзакции.

use super::errors::ParseStatusError;
use std::fmt;

/// Состояние транзакции.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    /// Успех.
    Success,

    /// Неудача.
    Failure,

    /// В процессе.
    Pending,
}

/// Реализация трейта [`fmt::Display`] для [`Status`].
impl fmt::Display for Status {
    /// Реализация метода [`fmt::Display::fmt`] для [`Status`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Success => "SUCCESS",
            Self::Failure => "FAILURE",
            Self::Pending => "PENDING",
        };

        write!(f, "{s}")
    }
}

/// Реализация трейта [`TryFrom<&str>`] для [`Status`].
impl TryFrom<&str> for Status {
    /// Ошибка парсинга состояния транзакции.
    type Error = ParseStatusError;

    /// Реализация метода [`TryFrom<&str>::try_from`] для [`Status`].
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "SUCCESS" => Ok(Self::Success),
            "FAILURE" => Ok(Self::Failure),
            "PENDING" => Ok(Self::Pending),
            _ => Err(ParseStatusError::InvalidStatus(s.to_string())),
        }
    }
}

/// Реализация трейта [`TryFrom<u8>`] для [`Status`].
impl TryFrom<u8> for Status {
    /// Ошибка парсинга состояния транзакции.
    type Error = ParseStatusError;

    /// Реализация метода [`TryFrom<u8>::try_from`] для [`Status`].
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(ParseStatusError::InvalidStatus(value.to_string())),
        }
    }
}

/// Реализация трейта [`From<Status>`] для [`u8`].
impl From<Status> for u8 {
    /// Реализация метода [`From<Status>::from`] для [`u8`].
    fn from(value: Status) -> Self {
        match value {
            Status::Success => 0,
            Status::Failure => 1,
            Status::Pending => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Status::Success.to_string(), "SUCCESS");
        assert_eq!(Status::Failure.to_string(), "FAILURE");
        assert_eq!(Status::Pending.to_string(), "PENDING");
    }

    #[test]
    fn test_try_from_string() {
        assert_eq!(Status::try_from("SUCCESS").unwrap(), Status::Success);
        assert_eq!(Status::try_from("FAILURE").unwrap(), Status::Failure);
        assert_eq!(Status::try_from("PENDING").unwrap(), Status::Pending);
        assert!(Status::try_from("").is_err_and(|e| e.to_string() == "Invalid STATUS: "));
        assert!(
            Status::try_from("INVALID").is_err_and(|e| e.to_string() == "Invalid STATUS: INVALID")
        );
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(Status::try_from(0).unwrap(), Status::Success);
        assert_eq!(Status::try_from(1).unwrap(), Status::Failure);
        assert_eq!(Status::try_from(2).unwrap(), Status::Pending);

        assert!(Status::try_from(3).is_err_and(|e| e.to_string() == "Invalid STATUS: 3"));
    }

    #[test]
    fn test_into_u8() {
        assert_eq!(u8::from(Status::Success), 0);
        assert_eq!(u8::from(Status::Failure), 1);
        assert_eq!(u8::from(Status::Pending), 2);
    }
}
