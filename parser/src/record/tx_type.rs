use super::errors::TxTypeParseError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(TxType::Deposit.to_string(), "DEPOSIT");
        assert_eq!(TxType::Transfer.to_string(), "TRANSFER");
        assert_eq!(TxType::Withdrawal.to_string(), "WITHDRAWAL");
    }

    #[test]
    fn test_try_from() {
        assert_eq!(TxType::try_from("DEPOSIT").unwrap(), TxType::Deposit);
        assert_eq!(TxType::try_from("TRANSFER").unwrap(), TxType::Transfer);
        assert_eq!(TxType::try_from("WITHDRAWAL").unwrap(), TxType::Withdrawal);

        assert!(TxType::try_from("").is_err_and(|e| e.to_string() == "Invalid tx type: "));
        assert!(
            TxType::try_from("INVALID").is_err_and(|e| e.to_string() == "Invalid tx type: INVALID")
        );
    }
}
