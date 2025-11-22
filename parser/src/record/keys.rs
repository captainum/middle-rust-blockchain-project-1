use super::errors::ParseKeyError;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RecordKey {
    TxId,
    TxType,
    FromUserId,
    ToUserId,
    Amount,
    Timestamp,
    Status,
    Description,
}

impl fmt::Display for RecordKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::TxId => "TX_ID",
            Self::TxType => "TX_TYPE",
            Self::FromUserId => "FROM_USER_ID",
            Self::ToUserId => "TO_USER_ID",
            Self::Amount => "AMOUNT",
            Self::Timestamp => "TIMESTAMP",
            Self::Status => "STATUS",
            Self::Description => "DESCRIPTION",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for RecordKey {
    type Error = ParseKeyError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "TX_ID" => Ok(Self::TxId),
            "TX_TYPE" => Ok(Self::TxType),
            "FROM_USER_ID" => Ok(Self::FromUserId),
            "TO_USER_ID" => Ok(Self::ToUserId),
            "AMOUNT" => Ok(Self::Amount),
            "TIMESTAMP" => Ok(Self::Timestamp),
            "STATUS" => Ok(Self::Status),
            "DESCRIPTION" => Ok(Self::Description),
            _ => Err(ParseKeyError::InvalidKey(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(RecordKey::TxId.to_string(), "TX_ID");
        assert_eq!(RecordKey::TxId.to_string(), "TX_ID");
        assert_eq!(RecordKey::TxType.to_string(), "TX_TYPE");
        assert_eq!(RecordKey::FromUserId.to_string(), "FROM_USER_ID");
        assert_eq!(RecordKey::ToUserId.to_string(), "TO_USER_ID");
        assert_eq!(RecordKey::Amount.to_string(), "AMOUNT");
        assert_eq!(RecordKey::Timestamp.to_string(), "TIMESTAMP");
        assert_eq!(RecordKey::Status.to_string(), "STATUS");
        assert_eq!(RecordKey::Description.to_string(), "DESCRIPTION");
    }

    #[test]
    fn test_try_from() {
        assert_eq!(RecordKey::try_from("TX_ID").unwrap(), RecordKey::TxId);
        assert_eq!(RecordKey::try_from("TX_TYPE").unwrap(), RecordKey::TxType);
        assert_eq!(
            RecordKey::try_from("FROM_USER_ID").unwrap(),
            RecordKey::FromUserId
        );
        assert_eq!(
            RecordKey::try_from("TO_USER_ID").unwrap(),
            RecordKey::ToUserId
        );
        assert_eq!(RecordKey::try_from("AMOUNT").unwrap(), RecordKey::Amount);
        assert_eq!(
            RecordKey::try_from("TIMESTAMP").unwrap(),
            RecordKey::Timestamp
        );
        assert_eq!(RecordKey::try_from("STATUS").unwrap(), RecordKey::Status);
        assert_eq!(
            RecordKey::try_from("DESCRIPTION").unwrap(),
            RecordKey::Description
        );

        assert!(RecordKey::try_from("").is_err_and(|e| e.to_string() == "Invalid key: "));
        assert!(
            RecordKey::try_from("INVALID").is_err_and(|e| e.to_string() == "Invalid key: INVALID")
        );
    }
}
