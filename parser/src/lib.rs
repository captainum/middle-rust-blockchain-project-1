mod errors;
mod record;

use record::Record;

use std::io::{BufRead, BufReader, BufWriter, Write, Read};
use errors::{ReadError, WriteError};

trait YPBankRecord {
    fn from_read<R: Read>(r: &mut R) -> Result<Self, ReadError>
    where
        Self: Sized;

    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError>;
}

struct YPBankTextRecord {
    records: Vec<Record>,
}

impl YPBankRecord for YPBankTextRecord {
    fn from_read<R: Read>(r: &mut R) -> Result<Self, ReadError> {
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
        let mut writer = BufWriter::new(w);

        for (i, record) in self.records.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            writer.write_all(record.to_string().as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
    }
}

struct YPBankCsvHeader {}

struct YPBankBinaryRecord {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::record::status::{Status};
    use crate::record::tx_type::{TxType};

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

    #[test]
    fn test_write_empty_txt_record() {
        let record = YPBankTextRecord { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        record.write_to(&mut cursor).unwrap();
        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_txt_records() {
        let records = vec![
            Record::new()
                .set_tx_id(1234567890123456)
                .set_tx_type(TxType::Deposit)
                .set_from_user_id(0)
                .set_to_user_id(9876543210987654)
                .set_amount(10000)
                .set_timestamp(1633036800000)
                .set_status(Status::Success)
                .set_description("Terminal deposit".to_string())
                .clone(),
            Record::new()
                .set_tx_id(2312321321321321)
                .set_timestamp(1633056800000)
                .set_status(Status::Failure)
                .set_tx_type(TxType::Transfer)
                .set_from_user_id(1231231231231231)
                .set_to_user_id(9876543210987654)
                .set_amount(1000)
                .set_description("User transfer".to_string())
                .clone(),
            Record::new()
                .set_tx_id(3213213213213213)
                .set_amount(100)
                .set_tx_type(TxType::Withdrawal)
                .set_from_user_id(9876543210987654)
                .set_to_user_id(0)
                .set_timestamp(1633066800000)
                .set_status(Status::Success)
                .set_description("User withdrawal".to_string())
                .clone(),
        ];

        let record = YPBankTextRecord { records };
        let mut cursor = Cursor::new(vec![]);
        record.write_to(&mut cursor).unwrap();
        assert_eq!(
            cursor.into_inner(),
            br#"TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"

TX_ID: 2312321321321321
TX_TYPE: TRANSFER
FROM_USER_ID: 1231231231231231
TO_USER_ID: 9876543210987654
AMOUNT: 1000
TIMESTAMP: 1633056800000
STATUS: FAILURE
DESCRIPTION: "User transfer"

TX_ID: 3213213213213213
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 9876543210987654
TO_USER_ID: 0
AMOUNT: 100
TIMESTAMP: 1633066800000
STATUS: SUCCESS
DESCRIPTION: "User withdrawal"
"#
        );
    }
}
