use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use crate::record::errors::ParseRecordFromCsvError;
use super::errors::{ReadError, WriteError};
use super::record::Record;
use super::YPBank;

#[derive(Debug)]
pub struct YPBankCsv {
    /// Записи о банковских операциях.
    pub records: Vec<Record>,
}

impl YPBankCsv {
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
            Err(ParseRecordFromCsvError::UnexpectedError(
                "invalid header structure".to_string(),
            ))?
        } else {
            Ok(())
        }
    }
}

impl YPBank for YPBankCsv {
    /// Считать данные о банковских операциях в CSV формате.
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        let mut header = String::new();
        reader.read_line(&mut header)?;

        header = header.trim_end_matches(['\r', '\n']).to_string();

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
    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::status::Status;
    use crate::record::tx_type::TxType;
    use crate::record::errors::ParseRecordFromCsvError;
    use std::io::Cursor;
    use rstest::rstest;

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
        matches!(
            result,
            ReadError::FromCsv(ParseRecordFromCsvError::UnexpectedError(_))
        );
        assert_eq!(
            result.to_string(),
            "CSV format parsing error: Unexpected error: invalid header structure"
        );
    }

    #[test]
    fn test_read_from_csv_invalid_header_symbol() {
        let mut cursor = Cursor::new(vec![0xff]);
        let result = YPBankCsv::read_from(&mut cursor);

        let result = result.unwrap_err();
        assert!(matches!(result, ReadError::Io(std::io::Error { .. })));
        assert_eq!(
            result.to_string(),
            "Read data error: stream did not contain valid UTF-8"
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
        assert!(matches!(
            result,
            ReadError::FromCsv(ParseRecordFromCsvError::InvalidCountOfColumns(_))
        ));
        assert_eq!(
            result.to_string(),
            "CSV format parsing error: Invalid count of columns: 7"
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
        let records = crate::tests::get_data_to_write();

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
}