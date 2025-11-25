use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use super::errors::{ReadError, WriteError};
use super::record::Record;
use super::YPBank;

#[derive(Debug)]
pub struct YPBankText {
    /// Записи о банковских операциях.
    pub records: Vec<Record>,
}

impl YPBank for YPBankText {
    /// Считать данные о банковских операциях в текстовом формате.
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        while !reader.fill_buf()?.is_empty() {
            records.push(Record::from_text(&mut reader)?);
        }

        Ok(Self { records })
    }

    /// Записать данные о банковских операциях в текстовом формате.
    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        let mut writer = BufWriter::new(w);

        for (i, record) in self.records.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            record.to_text(&mut writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::status::Status;
    use crate::record::tx_type::TxType;
    use crate::record::errors::ParseRecordFromTxtError;
    use std::io::Cursor;

    #[test]
    fn test_read_from_text_data_specification() {
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
        let result = YPBankText::read_from(&mut cursor);

        let data = result.unwrap();

        assert_eq!(data.records.len(), 3);

        let expected_records = vec![
            Record::new(
                1234567890123456,
                TxType::Deposit,
                0,
                9876543210987654,
                10000,
                1633036800000,
                Status::Success,
                "Terminal deposit".to_string(),
            )
                .clone(),
            Record::new(
                2312321321321321,
                TxType::Transfer,
                1231231231231231,
                9876543210987654,
                1000,
                1633056800000,
                Status::Failure,
                "User transfer".to_string(),
            )
                .clone(),
            Record::new(
                3213213213213213,
                TxType::Withdrawal,
                9876543210987654,
                0,
                100,
                1633066800000,
                Status::Success,
                "User withdrawal".to_string(),
            )
                .clone(),
        ];

        assert_eq!(data.records, expected_records);
    }

    #[test]
    fn test_read_from_text_invalid_record() {
        let mut reader = BufReader::new(Cursor::new(vec![0xff, 0xff]));

        let result = YPBankText::read_from(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ReadError::FromText(ParseRecordFromTxtError::UnexpectedError(_))
        ));
        assert_eq!(
            result.to_string(),
            "Text format parsing error: Unexpected error: stream did not contain valid UTF-8"
        );
    }

    #[test]
    fn test_write_to_text_empty_record() {
        let data = YPBankText { records: vec![] };

        let mut cursor = Cursor::new(vec![]);
        data.write_to(&mut cursor).unwrap();

        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_to_text() {
        let records = crate::tests::get_data_to_write();

        let data = YPBankText { records };
        let mut cursor = Cursor::new(vec![]);
        assert!(data.write_to(&mut cursor).is_ok());

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
