//! Библиотека, обеспечивающая парсинг и сериализацию форматов.
//!
//! Доступны операции чтения и записи данных о банковских операциях в форматах:
//! 1. [`YPBankCsv`] - таблица банковских операций;
//!
//! 2. [`YPBankText`] - текстовый формат описания списка операций;
//!
//! 3. [`YPBankBin`] - бинарное предоставление списка операций.
//!
//! Все указанные структуры данных реализуют трейт [`YPBank`], описывающий общее поведение.
//!
//! Чтение из источника данных, реализующего трейт [`Read`], производится при помощи
//! метода [`read_from`].
//!
//! Запись производится в назначение, реализующее трейт [`Write`], при помощи
//! метода ['write_to'].

mod errors;
mod record;

use record::Record;

use errors::{ReadError, WriteError};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use parser_macro::ReadWrite;

/// Трейт для парсинга данных о банковских операциях.
trait YPBank {
    /// Считать данные о банковских операциях из указанного источника.
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError>
    where
        Self: Sized;

    /// Записать данные о банковских операциях в указанное место.
    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError>;
}

/// Структура хранения данных о банковских операциях из источника данных,
/// имеющего текстовый формат.
#[derive(ReadWrite, Debug)]
#[format("text")]
struct YPBankText {
    /// Записи о банковских операциях.
    records: Vec<Record>,
}

/// Структура хранения данных о банковских операциях из источника данных,
/// имеющего CSV формат.
#[derive(ReadWrite, Debug)]
#[format("csv")]
struct YPBankCsv {
    /// Записи о банковских операциях.
    records: Vec<Record>,
}

impl YPBankCsv {
    /// Подготовить заголовок с именами полей.
    ///
    /// Заголовок соответствует следующей строке:
    ///
    /// TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
    fn prepare_header() -> String {
        Record::EXPECTED_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Валидировать переданный заголовок на соответствие ожидаемой структуре.
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

/// Структура хранения данных о банковских операциях из источника данных,
/// имеющего бинарный формат.
#[derive(ReadWrite, Debug)]
#[format("bin")]
struct YPBankBin {
    /// Записи о банковских операциях.
    records: Vec<Record>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::errors::ParseRecordFromBinError;
    use crate::record::status::Status;
    use crate::record::tx_type::TxType;
    use rstest::rstest;
    use std::io::Cursor;

    fn get_data_to_write() -> Vec<Record> {
        vec![
            Record::new(
                1234567890123456,
                TxType::Deposit,
                0,
                9876543210987654,
                10000,
                1633036800000,
                Status::Success,
                "Terminal deposit".to_string(),
            ),
            Record::new(
                2312321321321321,
                TxType::Transfer,
                1231231231231231,
                9876543210987654,
                1000,
                1633056800000,
                Status::Failure,
                "User transfer".to_string(),
            ),
            Record::new(
                3213213213213213,
                TxType::Withdrawal,
                9876543210987654,
                0,
                100,
                1633066800000,
                Status::Success,
                "User withdrawal".to_string(),
            ),
        ]
    }

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
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(
            result.to_string(),
            "Invalid format: Unexpected error: stream did not contain valid UTF-8"
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
        let records = get_data_to_write();

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

    #[test]
    fn test_read_from_csv_data_specification() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding"
1002,TRANSFER,501,502,15000,1672534800000,FAILURE,"Payment for services, invoice #123"
1003,WITHDRAWAL,502,0,1000,1672538400000,PENDING,"ATM withdrawal"
"#;
        let mut cursor = Cursor::new(data.as_bytes());
        let result = YPBankCsv::read_from(&mut cursor);

        let data = result.unwrap();
        assert_eq!(data.records.len(), 3);

        let expected_records = vec![
            Record::new(
                1001,
                TxType::Deposit,
                0,
                501,
                50000,
                1672531200000,
                Status::Success,
                "Initial account funding".to_string(),
            )
            .clone(),
            Record::new(
                1002,
                TxType::Transfer,
                501,
                502,
                15000,
                1672534800000,
                Status::Failure,
                "Payment for services, invoice #123".to_string(),
            )
            .clone(),
            Record::new(
                1003,
                TxType::Withdrawal,
                502,
                0,
                1000,
                1672538400000,
                Status::Pending,
                "ATM withdrawal".to_string(),
            )
            .clone(),
        ];

        assert_eq!(data.records, expected_records);
    }

    #[rstest]
    #[case("TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,TIMESTAMP,STATUS,DESCRIPTION")]
    #[case(",TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION")]
    fn test_read_from_csv_invalid_header(#[case] header: &str) {
        let mut cursor = Cursor::new(header.as_bytes());
        let result = YPBankCsv::read_from(&mut cursor);

        let result = result.unwrap_err();
        matches!(result, ReadError::InvalidFormat(_));
        assert_eq!(
            result.to_string(),
            "Invalid format: invalid header structure"
        );
    }

    #[test]
    fn test_read_from_csv_invalid_header_symbol() {
        let mut cursor = Cursor::new(vec![0xff]);
        let result = YPBankCsv::read_from(&mut cursor);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::UnexpectedError(_)));
        assert_eq!(
            result.to_string(),
            "Unexpected error: stream did not contain valid UTF-8"
        );
    }

    #[test]
    fn test_read_from_csv_invalid_record() {
        let mut reader = BufReader::new(Cursor::new(
            br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
"1001,DEPOSIT,0,501,50000,SUCCESS,\"Initial account funding\"""#,
        ));

        let result = YPBankCsv::read_from(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(
            result.to_string(),
            "Invalid format: Invalid count of columns: 7"
        );
    }

    #[test]
    fn test_write_to_csv_empty_record() {
        let data = YPBankCsv { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        data.write_to(&mut cursor).unwrap();
        assert_eq!(
            cursor.into_inner(),
            b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n"
        );
    }

    #[test]
    fn test_write_to_csv() {
        let records = get_data_to_write();

        let data = YPBankCsv { records };
        let mut cursor = Cursor::new(vec![]);
        data.write_to(&mut cursor).unwrap();

        assert_eq!(
            cursor.into_inner(),
            br#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1234567890123456,DEPOSIT,0,9876543210987654,10000,1633036800000,SUCCESS,"Terminal deposit"
2312321321321321,TRANSFER,1231231231231231,9876543210987654,1000,1633056800000,FAILURE,"User transfer"
3213213213213213,WITHDRAWAL,9876543210987654,0,100,1633066800000,SUCCESS,"User withdrawal"
"#
        );
    }

    #[test]
    fn test_read_from_bin_data_specification() {
        let data = [
            // Блок 1
            0x59, 0x50, 0x42, 0x4E, // MAGIC
            0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE
            0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // TX_ID
            0x00, // TX_TYPE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // FROM_USER_ID
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TO_USER_ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // AMOUNT
            0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // TIMESTAMP
            0x01, // STATUS
            0x00, 0x00, 0x00, 0x11, // DESCRIPTION_SIZE
            0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION
            // Блок 2
            0x59, 0x50, 0x42, 0x4e, // MAGIC
            0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE
            0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x01, // TX_ID
            0x01, // TX_TYPE
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // FROM_USER_ID
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TO_USER_ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, // AMOUNT
            0x00, 0x00, 0x01, 0x7c, 0x38, 0x95, 0xe4, 0xc0, // TIMESTAMP
            0x02, // STATUS
            0x00, 0x00, 0x00, 0x11, // DESCRIPTION_SIZE
            0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x20, 0x32, 0x22, // DESCRIPTION
        ];

        let mut cursor = Cursor::new(data);
        let result = YPBankBin::read_from(&mut cursor);

        let data = result.unwrap();
        assert_eq!(data.records.len(), 2);

        let expected_records = vec![
            Record::new(
                1000000000000000,
                TxType::Deposit,
                0,
                9223372036854775807,
                100,
                1633036860000,
                Status::Failure,
                "Record number 1".to_string(),
            ),
            Record::new(
                1000000000000001,
                TxType::Transfer,
                9223372036854775807,
                9223372036854775807,
                200,
                1633036920000,
                Status::Pending,
                "Record number 2".to_string(),
            ),
        ];

        assert_eq!(data.records, expected_records);
    }

    #[test]
    fn test_read_from_bin_invalid_record() {
        let mut reader = BufReader::new(Cursor::new(vec![0x59, 0x51, 0x42, 0x4E]));

        let result = YPBankBin::read_from(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(result.to_string(), "Invalid format: Invalid magic number");
    }

    #[test]
    fn test_write_to_bin_empty_record() {
        let data = YPBankBin { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        data.write_to(&mut cursor).unwrap();
        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_to_bin() {
        let records = get_data_to_write();

        let data = YPBankBin { records };
        let mut cursor = Cursor::new(vec![]);
        data.write_to(&mut cursor).unwrap();

        assert_eq!(
            cursor.into_inner(),
            [
                // Блок 1
                0x59, 0x50, 0x42, 0x4e, // MAGIC "YPBN"
                0x00, 0x00, 0x00, 0x40, // RECORD_SIZE (64)
                0x00, 0x04, 0x62, 0xd5, 0x3c, 0x8a, 0xba, 0xc0, // TX_ID
                0x00, // TX_TYPE
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // FROM_USER_ID
                0x00, 0x23, 0x16, 0xa9, 0xe9, 0xb3, 0x20, 0x86, // TO_USER_ID
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x27, 0x10, // AMOUNT (10000)
                0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0x10, 0x00, // TIMESTAMP
                0x00, // STATUS
                0x00, 0x00, 0x00, 0x12, // DESCRIPTION_SIZE (18)
                0x22, 0x54, 0x65, 0x72, 0x6d, 0x69, 0x6e, 0x61, 0x6c, 0x20, 0x64, 0x65, 0x70, 0x6f,
                0x73, 0x69, 0x74, 0x22, // DESCRIPTION "Terminal deposit"
                // Блок 2
                0x59, 0x50, 0x42, 0x4e, // MAGIC "YPBN"
                0x00, 0x00, 0x00, 0x3d, // RECORD_SIZE (61)
                0x00, 0x08, 0x37, 0x0b, 0x42, 0xf6, 0xc3, 0x69, // TX_ID
                0x01, // TX_TYPE
                0x00, 0x04, 0x5f, 0xcc, 0x5c, 0x2c, 0x84, 0xff, // FROM_USER_ID
                0x00, 0x23, 0x16, 0xa9, 0xe9, 0xb3, 0x20, 0x86, // TO_USER_ID
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, // AMOUNT (1000)
                0x00, 0x00, 0x01, 0x7c, 0x39, 0xc5, 0x3d, 0x00, // TIMESTAMP
                0x01, // STATUS
                0x00, 0x00, 0x00, 0x0f, // DESCRIPTION_SIZE (15)
                0x22, 0x55, 0x73, 0x65, 0x72, 0x20, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x66, 0x65, 0x72,
                0x22, // DESCRIPTION "User transfer"
                // Блок 3
                0x59, 0x50, 0x42, 0x4e, // MAGIC "YPBN"
                0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE (63)
                0x00, 0x0b, 0x6a, 0x66, 0x80, 0x29, 0x42, 0x1d, // TX_ID
                0x02, // TX_TYPE
                0x00, 0x23, 0x16, 0xa9, 0xe9, 0xb3, 0x20, 0x86, // FROM_USER_ID
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // TO_USER_ID
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // AMOUNT (100)
                0x00, 0x00, 0x01, 0x7c, 0x3a, 0x5d, 0xd3, 0x80, // TIMESTAMP
                0x00, // STATUS
                0x00, 0x00, 0x00, 0x11, // DESCRIPTION_SIZE (17)
                0x22, 0x55, 0x73, 0x65, 0x72, 0x20, 0x77, 0x69, 0x74, 0x68, 0x64, 0x72, 0x61, 0x77,
                0x61, 0x6c, 0x22, // DESCRIPTION "User withdrawal"
            ]
        );
    }
}
