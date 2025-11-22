use super::errors::ParseTxTypeError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    type Error = ParseTxTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DEPOSIT" => Ok(Self::Deposit),
            "TRANSFER" => Ok(Self::Transfer),
            "WITHDRAWAL" => Ok(Self::Withdrawal),
            _ => Err(ParseTxTypeError::InvalidTxType(value.to_string())),
        }
    }
}

impl TryFrom<u8> for TxType {
    type Error = ParseTxTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Deposit),
            1 => Ok(Self::Transfer),
            2 => Ok(Self::Withdrawal),
            _ => Err(ParseTxTypeError::InvalidTxType(value.to_string())),
        }
    }
}

impl From<TxType> for u8 {
    fn from(value: TxType) -> Self {
        match value {
            TxType::Deposit => 0,
            TxType::Transfer => 1,
            TxType::Withdrawal => 2,
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
    fn test_try_from_string() {
        assert_eq!(TxType::try_from("DEPOSIT").unwrap(), TxType::Deposit);
        assert_eq!(TxType::try_from("TRANSFER").unwrap(), TxType::Transfer);
        assert_eq!(TxType::try_from("WITHDRAWAL").unwrap(), TxType::Withdrawal);

        assert!(TxType::try_from("").is_err_and(|e| e.to_string() == "Invalid TX_TYPE: "));
        assert!(
            TxType::try_from("INVALID").is_err_and(|e| e.to_string() == "Invalid TX_TYPE: INVALID")
        );
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(TxType::try_from(0).unwrap(), TxType::Deposit);
        assert_eq!(TxType::try_from(1).unwrap(), TxType::Transfer);
        assert_eq!(TxType::try_from(2).unwrap(), TxType::Transfer);

        assert!(TxType::try_from(3).is_err_and(|e| e.to_string() == "Invalid TX_TYPE: 3"));
    }

    #[test]
    fn test_into_u8() {
        assert_eq!(u8::from(TxType::Deposit), 0);
        assert_eq!(u8::from(TxType::Transfer), 1);
        assert_eq!(u8::from(TxType::Withdrawal), 2);
    }
}
