use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

mod status;
mod tx_type;

use status::{Status, StatusParseError};
use tx_type::{TxType, TxTypeParseError};

pub struct Record {
    tx_id: u64,
    tx_type: TxType,
    from_user_id: u64,
    to_user_id: u64,
    amount: u64,
    timestamp: u64,
    status: Status,
    description: String,
}

macro_rules! setter {
    ($name:ident, $field:ident, $type:ty) => {
        fn $name(&mut self, $field: $type) -> &Self {
            self.$field = $field;
            self
        }
    };
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

impl Record {
    fn new() -> Self {
        Self {
            tx_id: 0,
            tx_type: TxType::Deposit,
            from_user_id: 0,
            to_user_id: 0,
            amount: 0,
            timestamp: 0,
            status: Status::Success,
            description: "".to_string(),
        }
    }

    setter!(set_tx_id, tx_id, u64);
    setter!(set_tx_type, tx_type, TxType);
    setter!(set_from_user_id, from_user_id, u64);
    setter!(set_to_user_id, to_user_id, u64);
    setter!(set_amount, amount, u64);
    setter!(set_timestamp, timestamp, u64);
    setter!(set_status, status, Status);
    setter!(set_description, description, String);

    fn from_lines(lines: Vec<&str>) -> Result<Self, FromLinesError> {
        let mut result = Self::new();

        let mut expected_keys = HashMap::from([
            ("TX_ID", false),
            ("TX_TYPE", false),
            ("FROM_USER_ID", false),
            ("TO_USER_ID", false),
            ("AMOUNT", false),
            ("TIMESTAMP", false),
            ("STATUS", false),
            ("DESCRIPTION", false),
        ]);

        for line in lines {
            if line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once(' ')
                .ok_or(FromLinesError::UnexpectedError {
                    line: line.to_string(),
                    description: "Could not parse string by space delimiter".to_string(),
                })?;

            if !key.ends_with(':') {
                return Err(FromLinesError::ColonNotFound(key.to_string()));
            }

            match key[..key.len() - 1].to_string().as_str() {
                "TX_ID" => {
                    let tx_id =
                        value
                            .parse::<u64>()
                            .map_err(|_| FromLinesError::IncorrectValueFormat {
                                description: "TX_ID is not a number".to_string(),
                            })?;

                    result.set_tx_id(tx_id);

                    expected_keys.remove("TX_ID");
                }
                "TX_TYPE" => {
                    let tx_type = value.try_into()?;

                    result.set_tx_type(tx_type);

                    expected_keys.remove("TX_TYPE");
                }
                "FROM_USER_ID" => {
                    let from_user_id =
                        value
                            .parse::<u64>()
                            .map_err(|_| FromLinesError::IncorrectValueFormat {
                                description: "FROM_USER_ID is not a number".to_string(),
                            })?;

                    result.set_from_user_id(from_user_id);

                    expected_keys.remove("FROM_USER_ID");
                }
                "TO_USER_ID" => {
                    let to_user_id =
                        value
                            .parse::<u64>()
                            .map_err(|_| FromLinesError::IncorrectValueFormat {
                                description: "TO_USER_ID is not a number".to_string(),
                            })?;

                    result.set_to_user_id(to_user_id);

                    expected_keys.remove("TO_USER_ID");
                }
                "AMOUNT" => {
                    let amount =
                        value
                            .parse::<u64>()
                            .map_err(|_| FromLinesError::IncorrectValueFormat {
                                description: "AMOUNT is not a number".to_string(),
                            })?;

                    result.set_amount(amount);

                    expected_keys.remove("AMOUNT");
                }
                "TIMESTAMP" => {
                    let timestamp =
                        value
                            .parse::<u64>()
                            .map_err(|_| FromLinesError::IncorrectValueFormat {
                                description: "TIMESTAMP is not a number".to_string(),
                            })?;

                    result.set_timestamp(timestamp);

                    expected_keys.remove("TIMESTAMP");
                }
                "STATUS" => {
                    let status = value.try_into()?;

                    result.set_status(status);

                    expected_keys.remove("STATUS");
                }
                "DESCRIPTION" => {
                    if !value.starts_with('"') || !value.ends_with('"') {
                        return Err(FromLinesError::IncorrectValueFormat {
                            description: "DESCRIPTION must start and end with symbol \""
                                .to_string(),
                        });
                    }
                    result.set_description(value[1..value.len() - 1].to_string());

                    expected_keys.remove("DESCRIPTION");
                }
                val => {
                    return Err(FromLinesError::UnexpectedKeyFound(val.to_string()));
                }
            }
        }

        if let Some((&key, _)) = expected_keys.iter().find(|(_, val)| !*val) {
            return Err(FromLinesError::MissingKey(key.to_string()));
        }

        Ok(result)
    }
}

impl TryFrom<Vec<&str>> for Record {
    type Error = FromLinesError;

    fn try_from(value: Vec<&str>) -> Result<Self, Self::Error> {
        Self::from_lines(value)
    }
}

impl TryFrom<Vec<String>> for Record {
    type Error = FromLinesError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Self::from_lines(value.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"TX_ID: {}
TX_TYPE: {}
FROM_USER_ID: {}
TO_USER_ID: {}
AMOUNT: {}
TIMESTAMP: {}
STATUS: {}
DESCRIPTION: "{}""#,
            self.tx_id,
            self.tx_type,
            self.from_user_id,
            self.to_user_id,
            self.amount,
            self.timestamp,
            self.status,
            self.description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_correct_record() {
        let lines = vec![
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            "TO_USER_ID: 2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
        ];

        let result = Record::from_lines(lines);

        let result = result.unwrap();

        assert_eq!(
            r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 1
TO_USER_ID: 2
AMOUNT: 100
TIMESTAMP: 1623228800
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit""#,
            result.to_string()
        );
    }

    #[test]
    fn test_correct_record_with_comments() {
        let lines = vec![
            "# comment1",
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            "TO_USER_ID: 2",
            "# comment2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
        ];

        let result = Record::from_lines(lines.clone());

        let result = result.unwrap();

        assert_eq!(
            r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 1
TO_USER_ID: 2
AMOUNT: 100
TIMESTAMP: 1623228800
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit""#,
            result.to_string()
        );
    }

    #[rstest]
    #[case("")]
    #[case("comment")]
    fn test_incorrect_lines(#[case] line: String) {
        let lines = vec![
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            line.as_str(),
            "TO_USER_ID: 2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
        ];

        let result = Record::from_lines(lines);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            FromLinesError::UnexpectedError {
                line,
                description: "Could not parse string by space delimiter".to_string()
            }
        );
    }

    #[test]
    fn test_no_colon_found() {
        let lines = vec![
            "# comment",
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            "TO_USER_ID 2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
        ];

        let result = Record::from_lines(lines);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            FromLinesError::ColonNotFound("TO_USER_ID".to_string())
        );
    }

    #[rstest]
    #[case("TX_ID", "ABC", "TX_ID is not a number")]
    #[case("FROM_USER_ID", "ABC", "FROM_USER_ID is not a number")]
    #[case("TO_USER_ID", "ABC", "TO_USER_ID is not a number")]
    #[case("AMOUNT", "ABC", "AMOUNT is not a number")]
    #[case("TIMESTAMP", "ABC", "TIMESTAMP is not a number")]
    #[case("TX_TYPE", "ABC", "Invalid tx type: ABC")]
    #[case("STATUS", "ABC", "Invalid status: ABC")]
    #[case(
        "DESCRIPTION",
        "ABC\"",
        "DESCRIPTION must start and end with symbol \""
    )]
    fn test_incorrect_number_values(
        #[case] key: &str,
        #[case] value: &str,
        #[case] description: String,
    ) {
        let lines = vec![format!("{}: {}", key, value)];

        let result = Record::from_lines(lines.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(result, FromLinesError::IncorrectValueFormat { description });
    }

    #[test]
    fn test_unexpected_key() {
        let lines = vec![
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            "TO_USER_ID: 2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
            "UNEXPECTED_KEY: 1",
        ];

        let result = Record::from_lines(lines);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            FromLinesError::UnexpectedKeyFound("UNEXPECTED_KEY".to_string())
        );
    }

    #[test]
    fn test_missing_key() {
        let lines = vec![
            "TX_ID: 1",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 1",
            "TO_USER_ID: 2",
            "AMOUNT: 100",
            "TIMESTAMP: 1623228800",
            "STATUS: SUCCESS",
        ];

        let result = Record::from_lines(lines);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            FromLinesError::MissingKey("DESCRIPTION".to_string())
        );
    }
}
