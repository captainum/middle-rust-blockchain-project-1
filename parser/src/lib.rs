//! Библиотека, обеспечивающая парсинг и сериализацию форматов.
//!
//! Доступны операции чтения и записи данных о банковских операциях в форматах:
//! 1. CSV-таблица банковских операций;
//!
//! 2. текстовый формат описания списка операций;
//!
//! 3. бинарное предоставление списка операций.
//!
//! Чтение из источника данных, реализующего трейт [`Read`], производится при помощи
//! методов [`read_from_text`], [`read_from_csv`], [`read_from_bin`] для соответствующих форматов данных.
//!
//! Запись производится в назначение, реализующее трейт [`Write`], при помощи
//! методов ['write_to_text'], ['write_to_csv'], ['write_to_bin'] для соответствующих форматов данных.

#![deny(unreachable_pub)]

mod errors;
mod record;

use record::Record;

use errors::{ReadError, WriteError};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// Структура для парсинга и хранения данных о банковских операциях.
#[derive(Debug)]
pub struct YPBank {
    /// Записи о банковских операциях.
    pub records: Vec<Record>,
}

impl YPBank {
    /// Считать данные о банковских операциях в текстовом формате.
    pub fn read_from_text<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        while !reader.fill_buf()?.is_empty() {
            records.push(Record::from_text(&mut reader)?);
        }

        Ok(Self { records })
    }

    /// Записать данные о банковских операциях в текстовом формате.
    pub fn write_to_text<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        let mut writer = BufWriter::new(w);

        for (i, record) in self.records.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            record.to_text(&mut writer)?;
        }

        Ok(())
    }

    /// Считать данные о банковских операциях в CSV формате.
    pub fn read_from_csv<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        let mut header = String::new();
        reader.read_line(&mut header)?;

        if header.ends_with('\n') {
            header.truncate(header.len() - 1);
        }

        Self::validate_header(&header)?;

        loop {
            if reader.fill_buf()?.is_empty() {
                break;
            }

            records.push(Record::from_csv(&mut reader)?);
        }

        Ok(Self { records })
    }

    /// Записать данные о банковских операциях в CSV формате.
    pub fn write_to_csv<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        let mut writer = BufWriter::new(w);

        let header = Self::prepare_header();
        writer
            .write_all(header.as_bytes())
            .map_err(|e| WriteError::WriteHeaderError(e.to_string()))?;
        writer.write_all(b"\n")?;

        for record in &self.records {
            record.to_csv(&mut writer)?;
        }

        Ok(())
    }

    /// Считать данные о банковских операциях в бинарном формате.
    pub fn read_from_bin<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        while !reader.fill_buf()?.is_empty() {
            records.push(Record::from_bin(&mut reader)?);
        }

        Ok(Self { records })
    }

    /// Записать данные о банковских операциях в бинарном формате.
    pub fn write_to_bin<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        let mut writer = BufWriter::new(w);

        for record in &self.records {
            record.to_bin(&mut writer)?;
        }

        Ok(())
    }
}

impl YPBank {
    /// Подготовить заголовок для CSV-формата с именами полей.
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

    /// Валидировать переданный заголовок для CSV-формата на соответствие ожидаемой структуре.
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

#[cfg(test)]
mod tests {
    use super::record::status::Status;
    use super::record::tx_type::TxType;
    use super::*;
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
        let result = YPBank::read_from_text(&mut cursor);

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

        let result = YPBank::read_from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(
            result.to_string(),
            "Invalid format: Unexpected error: stream did not contain valid UTF-8"
        );
    }

    #[test]
    fn test_write_to_text_empty_record() {
        let data = YPBank { records: vec![] };

        let mut cursor = Cursor::new(vec![]);
        data.write_to_text(&mut cursor).unwrap();

        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_to_text() {
        let records = get_data_to_write();

        let data = YPBank { records };
        let mut cursor = Cursor::new(vec![]);
        assert!(data.write_to_text(&mut cursor).is_ok());

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
        let result = YPBank::read_from_csv(&mut cursor);

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
        let result = YPBank::read_from_csv(&mut cursor);

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
        let result = YPBank::read_from_csv(&mut cursor);

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

        let result = YPBank::read_from_csv(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(
            result.to_string(),
            "Invalid format: Invalid count of columns: 7"
        );
    }

    #[test]
    fn test_write_to_csv_empty_record() {
        let data = YPBank { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        data.write_to_csv(&mut cursor).unwrap();
        assert_eq!(
            cursor.into_inner(),
            b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n"
        );
    }

    #[test]
    fn test_write_to_csv() {
        let records = get_data_to_write();

        let data = YPBank { records };
        let mut cursor = Cursor::new(vec![]);
        data.write_to_csv(&mut cursor).unwrap();

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
        let result = YPBank::read_from_bin(&mut cursor);

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

        let result = YPBank::read_from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::InvalidFormat(_)));
        assert_eq!(result.to_string(), "Invalid format: Invalid magic number");
    }

    #[test]
    fn test_write_to_bin_empty_record() {
        let data = YPBank { records: vec![] };
        let mut cursor = Cursor::new(vec![]);
        data.write_to_bin(&mut cursor).unwrap();
        assert_eq!(cursor.into_inner(), b"");
    }

    #[test]
    fn test_write_to_bin() {
        let records = get_data_to_write();

        let data = YPBank { records };
        let mut cursor = Cursor::new(vec![]);
        data.write_to_bin(&mut cursor).unwrap();

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
