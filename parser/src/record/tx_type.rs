use crate::record::status::Status;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Deposit => "DEPOSIT",
            Self::Transfer => "TRANSFER",
            Self::Withdrawal => "WITHDRAWAL",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, Error)]
pub enum TxTypeParseError {
    #[error("Invalid tx type: {0}")]
    InvalidTxType(String),
}

impl TryFrom<&str> for TxType {
    type Error = TxTypeParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "DEPOSIT" => Ok(Self::Deposit),
            "TRANSFER" => Ok(Self::Transfer),
            "WITHDRAWAL" => Ok(Self::Withdrawal),
            _ => Err(TxTypeParseError::InvalidTxType(s.to_string())),
        }
    }
}
