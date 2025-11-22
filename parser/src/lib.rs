mod errors;
mod record;

use record::Record;

use errors::{ReadError, WriteError};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use parser_macro::ReadWrite;

trait YPBank {
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError>
    where
        Self: Sized;

    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError>;
}

#[derive(ReadWrite)]
#[format("text")]
struct YPBankText {
    records: Vec<Record>,
}

#[derive(ReadWrite)]
#[format("csv")]
struct YPBankCsv {
    records: Vec<Record>,
}

impl YPBankCsv {
    fn prepare_header() -> String {
        Record::get_expected_keys()
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn validate_header(header: &str) -> Result<(), ReadError> {
        let expected_header = Self::prepare_header();

        if header != expected_header {
            Err(ReadError::InvalidFormat(
                "invalid header structure".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(ReadWrite)]
#[format("bin")]
struct YPBankBin {
    records: Vec<Record>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    use crate::record::status::Status;
    use crate::record::tx_type::TxType;

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
        let result = YPBankText::read_from(&mut cursor);

        let record = result.unwrap();

        assert_eq!(record.records.len(), 3);
    }

    #[test]
    fn test_write_empty_txt_record() {
        let record = YPBankText { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        record.write_to(&mut cursor).unwrap();
        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_txt_records() {
        let records = vec![
            Record::new(
                1234567890123456,
                TxType::Deposit,
                0,
                9876543210987654,
                10000,
                1633036800000,
                Status::Success,
                "Terminal deposit".to_string(),
            ).clone(),
            Record::default()
                .set_tx_id(2312321321321321)
                .set_timestamp(1633056800000)
                .set_status(Status::Failure)
                .set_tx_type(TxType::Transfer)
                .set_from_user_id(1231231231231231)
                .set_to_user_id(9876543210987654)
                .set_amount(1000)
                .set_description("User transfer".to_string())
                .clone(),
            Record::default()
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

        let record = YPBankText { records };
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
