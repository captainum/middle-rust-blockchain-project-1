use super::errors::{ParseStatusError, ParseTxTypeError};
use crate::record::tx_type::TxType;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Success,
    Failure,
    Pending,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Success => "SUCCESS",
            Self::Failure => "FAILURE",
            Self::Pending => "PENDING",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for Status {
    type Error = ParseStatusError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "SUCCESS" => Ok(Self::Success),
            "FAILURE" => Ok(Self::Failure),
            "PENDING" => Ok(Self::Pending),
            _ => Err(ParseStatusError::InvalidStatus(s.to_string())),
        }
    }
}

impl TryFrom<u8> for Status {
    type Error = ParseStatusError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            2 => Ok(Self::Pending),
            _ => Err(ParseStatusError::InvalidStatus(value.to_string())),
        }
    }
}

impl From<Status> for u8 {
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
