mod record;

use record::{FromLinesError, Record};

use std::io::{BufRead, BufReader, Write};

use thiserror::Error;

#[derive(Debug, Error)]
enum ReadError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

impl From<FromLinesError> for ReadError {
    fn from(e: FromLinesError) -> Self {
        Self::InvalidFormat(e.to_string())
    }
}

enum WriteError {
    InvalidFormat,
}

trait YPBankRecord {
    fn from_read<R: std::io::Read>(r: &mut R) -> Result<Self, ReadError>
    where
        Self: Sized;

    fn write_to<W: std::io::Write>(&self, w: &mut W) -> Result<(), WriteError>;
}

struct YPBankTextRecord {
    records: Vec<Record>,
}

impl YPBankRecord for YPBankTextRecord {
    fn from_read<R: std::io::Read>(r: &mut R) -> Result<Self, ReadError> {
        let reader = BufReader::new(r);

        let lines = reader.lines();

        let mut record_lines = vec![];

        let mut records: Vec<Record> = vec![];

        for line in lines {
            if let Ok(line) = line {
                if line.len() > 0 {
                    record_lines.push(line);
                } else if record_lines.len() > 0 {
                    let record = Record::try_from(record_lines)?;
                    records.push(record);

                    record_lines = vec![];
                }
            } else {
                return Err(ReadError::UnexpectedError(
                    "could not parse line".to_string(),
                ));
            }
        }

        if record_lines.len() > 0 {
            let record = Record::try_from(record_lines)?;
            records.push(record);
        }

        Ok(Self { records })
    }

    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        todo!()
    }
}

struct YPBankCsvHeader {}

struct YPBankBinaryRecord {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_data_from_specification() {
        let data = r#"# Record 1 (Deposit)
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

# Record 2 (Transfer)
TX_ID: 2312321321321321
TIMESTAMP: 1633056800000
STATUS: FAILURE
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
AMOUNT: 1000
DESCRIPTION: "User transfer"

# Record 3 (Withdrawal)
TX_ID: 3213213213213213
AMOUNT: 100
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
TIMESTAMP: 1633066800000
STATUS: SUCCESS
DESCRIPTION: "User withdrawal"
"#;
        let mut cursor = Cursor::new(data);
        let result = YPBankTextRecord::from_read(&mut cursor);

        let record = result.unwrap();

        assert_eq!(record.records.len(), 3);
    }
}
