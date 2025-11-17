use std::fmt;
use thiserror::Error;

use super::tx_type;

#[derive(Debug)]
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

#[derive(Debug, Error)]
pub enum StatusParseError {
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
}

impl TryFrom<&str> for Status {
    type Error = StatusParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "SUCCESS" => Ok(Self::Success),
            "FAILURE" => Ok(Self::Failure),
            "PENDING" => Ok(Self::Pending),
            _ => Err(StatusParseError::InvalidStatus(s.to_string())),
        }
    }
}
