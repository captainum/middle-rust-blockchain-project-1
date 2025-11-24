//! Модуль описания записи о транзакции.

use std::collections::HashMap;
use std::io::{BufRead, Write};

pub mod errors;
pub mod keys;
pub mod status;
pub mod tx_type;

use errors::{
    ParseRecordFromBinError, ParseRecordFromCsvError, ParseRecordFromTxtError, ParseStatusError,
    ParseTxTypeError, ParseValueError,
};
use keys::RecordKey;
use status::Status;
use tx_type::TxType;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

/// Структура хранения данных записи о транзакции.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// Неотрицательное целое число, идентифицирующее транзакцию.
    tx_id: u64,

    /// Тип транзакции.
    tx_type: TxType,

    /// Неотрицательное целое число, идентифицирующее отправитель счета
    /// (0 для типа транзакции `Deposit`).
    from_user_id: u64,

    /// Неотрицательное целое число, идентифицирующее получателя счета
    /// (0 для типа транзакции `Withdrawal`).
    to_user_id: u64,

    /// Неотрицательное целое число, представляющее сумму в наименьшей единице валюты.
    amount: u64,

    /// Unix epoch timestamp в миллисекундах.
    timestamp: u64,

    /// Состояние транзакции.
    status: Status,

    /// Произвольное текстовое описание.
    description: String,
}

/// Макрос установки заданного поля записи о транзакции.
macro_rules! setter {
    ($name:ident, $field:ident, $type:ty) => {
        pub fn $name(&mut self, $field: $type) -> &mut Self {
            self.$field = $field;
            self
        }
    };
}

/// Реализаций трейта [`Default`] для [`Record`].
impl Default for Record {
    /// Реализация метода [`Default::default`] для [`Record`].
    fn default() -> Self {
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
}

impl Record {
    /// Получить ожидаемые поля записи транзакции по их ключам.
    pub const EXPECTED_KEYS: [RecordKey; 8] = [
        RecordKey::TxId,
        RecordKey::TxType,
        RecordKey::FromUserId,
        RecordKey::ToUserId,
        RecordKey::Amount,
        RecordKey::Timestamp,
        RecordKey::Status,
        RecordKey::Description,
    ];

    /// Создание нового объекта записи о транзакции на основе переданных данных.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_id: u64,
        tx_type: TxType,
        from_user_id: u64,
        to_user_id: u64,
        amount: u64,
        timestamp: u64,
        status: Status,
        description: String,
    ) -> Self {
        Self {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
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

    /// Валидация и установка значения идентификатора транзакции.
    fn validate_and_set_tx_id(&mut self, value: &str) -> Result<(), ParseValueError> {
        let tx_id = value
            .parse::<u64>()
            .map_err(|_| ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "TX_ID is not a number".to_string(),
            })?;

        self.set_tx_id(tx_id);

        Ok(())
    }

    /// Валидация и установка значения типа транзакции.
    fn validate_and_set_tx_type(&mut self, value: &str) -> Result<(), ParseValueError> {
        let tx_type = value.try_into()?;

        self.set_tx_type(tx_type);

        Ok(())
    }

    /// Валидация и установка значения идентификатора отправителя счета транзакции.
    fn validate_and_set_from_user_id(&mut self, value: &str) -> Result<(), ParseValueError> {
        let from_user_id = value
            .parse::<u64>()
            .map_err(|_| ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "FROM_USER_ID is not a number".to_string(),
            })?;

        self.set_from_user_id(from_user_id);

        Ok(())
    }

    /// Валидация и установка значения идентификатора получателя счета транзакции.
    fn validate_and_set_to_user_id(&mut self, value: &str) -> Result<(), ParseValueError> {
        let to_user_id = value
            .parse::<u64>()
            .map_err(|_| ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "TO_USER_ID is not a number".to_string(),
            })?;

        self.set_to_user_id(to_user_id);

        Ok(())
    }

    /// Валидация и установка значения суммы транзакции.
    fn validate_and_set_amount(&mut self, value: &str) -> Result<(), ParseValueError> {
        let amount = value
            .parse::<u64>()
            .map_err(|_| ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "AMOUNT is not a number".to_string(),
            })?;

        self.set_amount(amount);

        Ok(())
    }

    /// Валидация и установка значения timestamp транзакции.
    fn validate_and_set_timestamp(&mut self, value: &str) -> Result<(), ParseValueError> {
        let timestamp = value
            .parse::<u64>()
            .map_err(|_| ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "TIMESTAMP is not a number".to_string(),
            })?;

        self.set_timestamp(timestamp);

        Ok(())
    }

    /// Валидация и установка значения состояния транзакции.
    fn validate_and_set_status(&mut self, value: &str) -> Result<(), ParseValueError> {
        let status = value.try_into()?;

        self.set_status(status);

        Ok(())
    }

    /// Валидация и установка значения произвольного текстового описания транзакции.
    fn validate_and_set_description(&mut self, value: &str) -> Result<(), ParseValueError> {
        if !value.starts_with('"') || !value.ends_with('"') {
            return Err(ParseValueError::InvalidValue {
                value: value.to_string(),
                description: "DESCRIPTION must start and end with symbol \"".to_string(),
            });
        }

        self.set_description(value[1..value.len() - 1].to_string());

        Ok(())
    }

    /// Валидация и установка значения поля записи транзакции по его ключу.
    fn validate_and_set_value_by_key(
        &mut self,
        key: RecordKey,
        value: &str,
    ) -> Result<(), ParseValueError> {
        match key {
            RecordKey::TxId => self.validate_and_set_tx_id(value),
            RecordKey::TxType => self.validate_and_set_tx_type(value),
            RecordKey::FromUserId => self.validate_and_set_from_user_id(value),
            RecordKey::ToUserId => self.validate_and_set_to_user_id(value),
            RecordKey::Amount => self.validate_and_set_amount(value),
            RecordKey::Timestamp => self.validate_and_set_timestamp(value),
            RecordKey::Status => self.validate_and_set_status(value),
            RecordKey::Description => self.validate_and_set_description(value),
        }
    }

    /// Считать данные о транзакции из указанного источника, имеющего текстовый формат записи.
    pub fn from_text<R: BufRead>(r: &mut R) -> Result<Self, ParseRecordFromTxtError> {
        let mut result = Self::default();

        let mut expected_keys = Self::EXPECTED_KEYS
            .iter()
            .map(|&k| (k, false))
            .collect::<HashMap<_, _>>();

        loop {
            let mut line = String::new();

            let bytes_count = r.read_line(&mut line)?;

            if bytes_count == 0 || line == "\n" {
                break;
            }

            if line.ends_with('\n') {
                line.truncate(line.len() - 1);
            }

            if line.starts_with('#') {
                continue;
            }

            let (key, value) =
                line.split_once(' ')
                    .ok_or(ParseRecordFromTxtError::UnexpectedError(format!(
                        "Could not parse string by space delimiter: {}",
                        line
                    )))?;

            if !key.ends_with(':') {
                return Err(ParseRecordFromTxtError::ColonNotFound(key.to_string()));
            }

            let key = RecordKey::try_from(&key[..key.len() - 1])?;

            result.validate_and_set_value_by_key(key, value)?;
            expected_keys.remove(&key);
        }

        if let Some((key, _)) = expected_keys.iter().find(|(_, val)| !*val) {
            return Err(ParseRecordFromTxtError::MissingKey(key.to_string()));
        }

        Ok(result)
    }

    /// Записать данные о транзакции в указанное место в текстовом формате.
    pub fn to_text<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(
            format!(
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
            .as_bytes(),
        )?;
        w.write_all("\n".as_bytes())
    }

    /// Считать данные о транзакции из указанного источника, имеющего CSV формат записи.
    pub fn from_csv<R: BufRead>(r: &mut R) -> Result<Self, ParseRecordFromCsvError> {
        let mut result = Self::default();

        let mut line = String::new();

        let bytes_count = r.read_line(&mut line)?;

        if bytes_count == 0 {
            return Err(ParseRecordFromCsvError::UnexpectedError(
                "EOF is reached".to_string(),
            ));
        }

        if line.ends_with('\n') {
            line.truncate(line.len() - 1);
        }

        let values = line
            .splitn(Self::EXPECTED_KEYS.len(), ',')
            .collect::<Vec<_>>();

        if Self::EXPECTED_KEYS.len() != values.len() {
            return Err(ParseRecordFromCsvError::InvalidCountOfColumns(values.len()));
        }

        for (&key, value) in Self::EXPECTED_KEYS.iter().zip(values.iter()) {
            result.validate_and_set_value_by_key(key, value)?;
        }

        Ok(result)
    }

    /// Записать данные о транзакции в указанное место в CSV формате.
    pub fn to_csv<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(
            format!(
                "{},{},{},{},{},{},{},\"{}\"\n",
                self.tx_id,
                self.tx_type,
                self.from_user_id,
                self.to_user_id,
                self.amount,
                self.timestamp,
                self.status,
                self.description
            )
            .as_bytes(),
        )
    }

    const BINARY_MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];
    const BINARY_MIN_RECORD_SIZE: u32 = 46;

    /// Считать данные о транзакции из указанного источника, имеющего бинарный формат записи.
    pub fn from_bin<R: BufRead>(r: &mut R) -> Result<Self, ParseRecordFromBinError> {
        let mut result = Self::default();

        let mut magic = [0u8; 4];

        r.read_exact(&mut magic)?;

        if magic != Self::BINARY_MAGIC {
            return Err(ParseRecordFromBinError::InvalidMagicNumber);
        }

        let record_size = r.read_u32::<BigEndian>()?;

        if record_size < Self::BINARY_MIN_RECORD_SIZE {
            return Err(ParseRecordFromBinError::InvalidRecordSize(record_size));
        }

        let tx_id = r.read_u64::<BigEndian>()?;
        result.set_tx_id(tx_id);

        let tx_type_raw = r.read_u8()?;
        let tx_type = tx_type_raw.try_into().map_err(|e: ParseTxTypeError| {
            ParseValueError::InvalidValue {
                value: tx_type_raw.to_string(),
                description: e.to_string(),
            }
        })?;
        result.set_tx_type(tx_type);

        let from_user_id = r.read_u64::<BigEndian>()?;
        result.set_from_user_id(from_user_id);

        let to_user_id = r.read_u64::<BigEndian>()?;
        result.set_to_user_id(to_user_id);

        let amount = r.read_u64::<BigEndian>()?;
        result.set_amount(amount);

        let timestamp = r.read_u64::<BigEndian>()?;
        result.set_timestamp(timestamp);

        let status_raw = r.read_u8()?;
        let status =
            status_raw
                .try_into()
                .map_err(|e: ParseStatusError| ParseValueError::InvalidValue {
                    value: status_raw.to_string(),
                    description: e.to_string(),
                })?;
        result.set_status(status);

        let desc_len = r.read_u32::<BigEndian>()?;

        if desc_len > 0 {
            let mut buffer = vec![0u8; desc_len as usize];
            r.read_exact(&mut buffer)?;

            result.validate_and_set_description(
                String::from_utf8(buffer.clone())
                    .map_err(|e| ParseValueError::InvalidValue {
                        value: String::from_utf8_lossy(&buffer).to_string(),
                        description: e.to_string(),
                    })?
                    .as_str(),
            )?;
        }

        Ok(result)
    }

    /// Записать данные о транзакции в указанное место в бинарном формате.
    pub fn to_bin<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(&Self::BINARY_MAGIC)?;

        let description_len = self.description.len() as u32 + 2;
        let record_size = Self::BINARY_MIN_RECORD_SIZE + description_len;
        w.write_u32::<BigEndian>(record_size)?;

        w.write_u64::<BigEndian>(self.tx_id)?;
        w.write_u8(self.tx_type as u8)?;
        w.write_u64::<BigEndian>(self.from_user_id)?;
        w.write_u64::<BigEndian>(self.to_user_id)?;
        w.write_u64::<BigEndian>(self.amount)?;
        w.write_u64::<BigEndian>(self.timestamp)?;
        w.write_u8(self.status as u8)?;
        w.write_u32::<BigEndian>(description_len)?;
        w.write_all(format!("\"{}\"", self.description).as_bytes())
    }
}

// /// Реализация трейта [`fmt::Display`] для [`Record`].
// impl fmt::Display for Record {
//     /// Реализация метода [`fmt::Display::fmt`] для [`Record`].
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "tx_id: {}, tx_type: {}, from_user_id: {}, to_user_id: {}, amount: {}, timestamp: {}, status: {}, description: {}",
//             self.tx_id,
//             self.tx_type,
//             self.from_user_id,
//             self.to_user_id,
//             self.amount,
//             self.timestamp,
//             self.status,
//             self.description
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use super::errors::ParseKeyError;
    use rstest::rstest;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_read_from_text_correct_record() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 0",
                "TO_USER_ID: 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
                "DESCRIPTION: \"Terminal deposit\"",
            ]
            .join("\n"),
        ));
        let result = Record::from_text(&mut reader);

        let record = result.unwrap();

        assert_eq!(
            record,
            Record::new(
                1,
                TxType::Deposit,
                0,
                2,
                100,
                1623228800,
                Status::Success,
                "Terminal deposit".to_string()
            )
        );
    }

    #[test]
    fn test_read_from_text_correct_record_with_comments() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
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
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap();

        let mut cursor = Cursor::new(Vec::new());
        assert!(result.to_text(&mut cursor).is_ok());
        assert_eq!(
            cursor.into_inner(),
            br#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 1
TO_USER_ID: 2
AMOUNT: 100
TIMESTAMP: 1623228800
STATUS: SUCCESS
DESCRIPTION: "Terminal deposit"
"#
        );
    }

    #[test]
    fn test_read_from_text_incorrect_symbol() {
        let mut reader = BufReader::new(Cursor::new(vec![0xff, 0xff]));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromTxtError::UnexpectedError(_)
        ));
        assert_eq!(
            result.to_string(),
            "Unexpected error: stream did not contain valid UTF-8"
        );
    }

    #[test]
    fn test_read_from_text_incorrect_line() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 1",
                "comment",
                "TO_USER_ID: 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
                "DESCRIPTION: \"Terminal deposit\"",
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromTxtError::UnexpectedError(_)
        ));
        assert_eq!(
            result.to_string(),
            "Unexpected error: Could not parse string by space delimiter: comment"
        );
    }

    #[test]
    fn test_read_from_text_no_colon_found() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "# comment",
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 1",
                "TO_USER_ID 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
                "DESCRIPTION: \"Terminal deposit\"",
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ParseRecordFromTxtError::ColonNotFound(_)));
        assert_eq!(result.to_string(), "Colon after key=TO_USER_ID not found");
    }

    #[rstest]
    #[case("TX_ID", "ABC", "TX_ID is not a number")]
    #[case("FROM_USER_ID", "ABC", "FROM_USER_ID is not a number")]
    #[case("TO_USER_ID", "ABC", "TO_USER_ID is not a number")]
    #[case("AMOUNT", "ABC", "AMOUNT is not a number")]
    #[case("TIMESTAMP", "ABC", "TIMESTAMP is not a number")]
    #[case("TX_TYPE", "ABC", "Invalid TX_TYPE: ABC")]
    #[case("STATUS", "ABC", "Invalid STATUS: ABC")]
    #[case(
        "DESCRIPTION",
        "ABC\"",
        "DESCRIPTION must start and end with symbol \""
    )]
    fn test_read_from_text_incorrect_value(
        #[case] key: &str,
        #[case] value: &str,
        #[case] description: &str,
    ) {
        let mut reader =
            BufReader::new(Cursor::new(vec![format!("{}: {}", key, value)].join("\n")));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromTxtError::InvalidValue(ParseValueError::InvalidValue { .. })
        ));
        assert_eq!(
            result.to_string(),
            format!("Invalid value: {value} ({description})")
        );
    }

    #[test]
    fn test_read_from_text_unexpected_key() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 1",
                "TO_USER_ID: 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
                "DESCRIPTION: \"Terminal deposit\"",
                "UNEXPECTED_KEY: 1",
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromTxtError::InvalidKey(ParseKeyError::InvalidKey(_))
        ));
        assert_eq!(result.to_string(), "Invalid key: UNEXPECTED_KEY");
    }

    #[test]
    fn test_read_from_text_missing_key() {
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 1",
                "TO_USER_ID: 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(result, ParseRecordFromTxtError::MissingKey(_)));
        assert_eq!(result.to_string(), "Missing key: DESCRIPTION");
    }

    #[test]
    fn test_write_to_text() {
        let record = Record::new(
            1001,
            TxType::Deposit,
            0,
            501,
            50000,
            1672531200000,
            Status::Success,
            "Initial account funding".to_string(),
        );

        let mut cursor = Cursor::new(Vec::new());
        assert!(record.to_text(&mut cursor).is_ok());
        assert_eq!(
            cursor.into_inner(),
            br#"TX_ID: 1001
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 501
AMOUNT: 50000
TIMESTAMP: 1672531200000
STATUS: SUCCESS
DESCRIPTION: "Initial account funding"
"#
        );
    }

    #[rstest]
    #[case("1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"")]
    #[case("1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"\n")]
    fn test_read_from_csv_correct_record(#[case] line: &str) {
        let mut reader = BufReader::new(Cursor::new(vec![line].join("\n")));

        let result = Record::from_csv(&mut reader);

        let record = result.unwrap();

        assert_eq!(
            record,
            Record::new(
                1001,
                TxType::Deposit,
                0,
                501,
                50000,
                1672531200000,
                Status::Success,
                "Initial account funding".to_string()
            )
        );
    }

    #[test]
    fn test_read_from_csv_eof() {
        let mut reader = BufReader::new(Cursor::new(vec![]));

        let result = Record::from_csv(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromCsvError::UnexpectedError(_)
        ));
        assert_eq!(result.to_string(), "Unexpected error: EOF is reached");
    }

    #[test]
    fn test_read_from_csv_incorrect_symbol() {
        let mut reader = BufReader::new(Cursor::new(vec![0xff, 0xff]));

        let result = Record::from_csv(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromCsvError::UnexpectedError(_)
        ));
        assert_eq!(
            result.to_string(),
            "Unexpected error: stream did not contain valid UTF-8"
        );
    }

    #[rstest]
    #[case("1001", 1)]
    #[case("1001,DEPOSIT,0,501,50000,SUCCESS,\"Initial account funding\"", 7)]
    fn test_read_from_csv_incorrect_count_of_columns(#[case] line: &str, #[case] count: usize) {
        let mut reader = BufReader::new(Cursor::new(vec![line].join("\n")));

        let result = Record::from_csv(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromCsvError::InvalidCountOfColumns(_)
        ));
        assert_eq!(
            result.to_string(),
            format!("Invalid count of columns: {}", count)
        );
    }

    #[rstest]
    #[case(
        "ABC,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"",
        "ABC",
        "TX_ID is not a number"
    )]
    #[case(
        "1001,DEPOSIT,ABC,501,50000,1672531200000,SUCCESS,\"Initial account funding\"",
        "ABC",
        "FROM_USER_ID is not a number"
    )]
    #[case(
        "1001,DEPOSIT,0,ABC,50000,1672531200000,SUCCESS,\"Initial account funding\"",
        "ABC",
        "TO_USER_ID is not a number"
    )]
    #[case(
        "1001,DEPOSIT,0,501,ABC,1672531200000,SUCCESS,\"Initial account funding\"",
        "ABC",
        "AMOUNT is not a number"
    )]
    #[case(
        "1001,DEPOSIT,0,501,50000,ABC,SUCCESS,\"Initial account funding\"",
        "ABC",
        "TIMESTAMP is not a number"
    )]
    #[case(
        "1001,ABC,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"",
        "ABC",
        "Invalid TX_TYPE: ABC"
    )]
    #[case(
        "1001,DEPOSIT,0,501,50000,1672531200000,ABC,\"Initial account funding\"",
        "ABC",
        "Invalid STATUS: ABC"
    )]
    #[case(
        "1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,ABC\"",
        "ABC\"",
        "DESCRIPTION must start and end with symbol \""
    )]
    fn test_read_from_csv_incorrect_value(
        #[case] line: &str,
        #[case] value: &str,
        #[case] description: &str,
    ) {
        let mut reader = BufReader::new(Cursor::new(vec![line].join("\n")));

        let result = Record::from_csv(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromCsvError::InvalidValue(ParseValueError::InvalidValue { .. })
        ));
        assert_eq!(
            result.to_string(),
            format!("Invalid value: {value} ({description})")
        );
    }

    #[test]
    fn test_write_to_csv() {
        let record = Record::new(
            1001,
            TxType::Deposit,
            0,
            501,
            50000,
            1672531200000,
            Status::Success,
            "Initial account funding".to_string(),
        );

        let mut cursor = Cursor::new(Vec::new());
        assert!(record.to_csv(&mut cursor).is_ok());
        assert_eq!(
            cursor.into_inner(),
            b"1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,\"Initial account funding\"\n"
        )
    }

    #[test]
    fn test_read_from_bin_correct_record() {
        let mut reader = BufReader::new(Cursor::new(vec![
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
        ]));

        let result = Record::from_bin(&mut reader);

        let record = result.unwrap();

        assert_eq!(
            record,
            Record::new(
                1000000000000000,
                TxType::Deposit,
                0,
                9223372036854775807,
                100,
                1633036860000,
                Status::Failure,
                "Record number 1".to_string()
            )
        );
    }

    #[test]
    fn test_read_from_bin_correct_record_empty_description() {
        let mut reader = BufReader::new(Cursor::new(vec![
            0x59, 0x50, 0x42, 0x4E, // MAGIC
            0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE
            0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // TX_ID
            0x00, // TX_TYPE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // FROM_USER_ID
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TO_USER_ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // AMOUNT
            0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // TIMESTAMP
            0x01, // STATUS
            0x00, 0x00, 0x00, 0x00, // DESCRIPTION_SIZE
        ]));

        let result = Record::from_bin(&mut reader);

        let record = result.unwrap();

        assert_eq!(
            record,
            Record::new(
                1000000000000000,
                TxType::Deposit,
                0,
                9223372036854775807,
                100,
                1633036860000,
                Status::Failure,
                "".to_string()
            )
        );
    }

    #[test]
    fn test_read_from_bin_eof() {
        let mut reader = BufReader::new(Cursor::new(vec![]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::UnexpectedError(_)
        ));
        assert_eq!(
            result.to_string(),
            "Unexpected error: failed to fill whole buffer"
        );
    }

    #[test]
    fn test_read_from_bin_invalid_magic() {
        let mut reader = BufReader::new(Cursor::new(vec![0x59, 0x51, 0x42, 0x4E]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::InvalidMagicNumber
        ));
        assert_eq!(result.to_string(), "Invalid magic number");
    }

    #[test]
    fn test_read_from_bin_invalid_record_size() {
        let mut reader = BufReader::new(Cursor::new(vec![
            0x59, 0x50, 0x42, 0x4E, 0x00, 0x00, 0x00, 0x20,
        ]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::InvalidRecordSize(_)
        ));
        assert_eq!(result.to_string(), "Invalid record size: 32");
    }

    #[test]
    fn test_read_from_bin_invalid_description_size() {
        let mut reader = BufReader::new(Cursor::new(vec![
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
            0x20, 0x31, // DESCRIPTION
        ]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::UnexpectedError(_)
        ));
        assert_eq!(
            result.to_string(),
            "Unexpected error: failed to fill whole buffer"
        );
    }

    #[test]
    fn test_read_from_bin_invalid_tx_type() {
        let mut reader = BufReader::new(Cursor::new(vec![
            0x59, 0x50, 0x42, 0x4E, // MAGIC
            0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE
            0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // TX_ID
            0xf0, // TX_TYPE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // FROM_USER_ID
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TO_USER_ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // AMOUNT
            0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // TIMESTAMP
            0x01, // STATUS
            0x00, 0x00, 0x00, 0x11, // DESCRIPTION_SIZE
            0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION
        ]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::InvalidValue(ParseValueError::InvalidValue { .. })
        ));
        assert_eq!(
            result.to_string(),
            "Invalid value: 240 (Invalid TX_TYPE: 240)"
        );
    }

    #[test]
    fn test_read_from_bin_invalid_status() {
        let mut reader = BufReader::new(Cursor::new(vec![
            0x59, 0x50, 0x42, 0x4E, // MAGIC
            0x00, 0x00, 0x00, 0x3f, // RECORD_SIZE
            0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // TX_ID
            0x00, // TX_TYPE
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // FROM_USER_ID
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // TO_USER_ID
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // AMOUNT
            0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // TIMESTAMP
            0xf1, // STATUS
            0x00, 0x00, 0x00, 0x11, // DESCRIPTION_SIZE
            0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x20, 0x31, 0x22, // DESCRIPTION
        ]));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::InvalidValue(ParseValueError::InvalidValue { .. })
        ));
        assert_eq!(
            result.to_string(),
            "Invalid value: 241 (Invalid STATUS: 241)"
        );
    }

    #[rstest]
    #[case(vec![0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x20, 0x31, 0xff], "Invalid value: \"Record number 1� (invalid utf-8 sequence of 1 bytes from index 16)")]
    #[case(vec![0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72,
             0x20, 0x31, 0x21], "Invalid value: \"Record number 1! (DESCRIPTION must start and end with symbol \")")]
    fn test_read_from_bin_invalid_description(
        #[case] description: Vec<u8>,
        #[case] description_error: &str,
    ) {
        let mut bytes = vec![
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
        ];
        bytes.extend(description);

        let mut reader = BufReader::new(Cursor::new(bytes));

        let result = Record::from_bin(&mut reader);

        let result = result.unwrap_err();
        assert!(matches!(
            result,
            ParseRecordFromBinError::InvalidValue(ParseValueError::InvalidValue { .. })
        ));
        assert_eq!(result.to_string(), description_error);
    }

    #[test]
    fn test_write_to_bin() {
        let record = Record::new(
            1000000000000000,
            TxType::Deposit,
            0,
            9223372036854775807,
            100,
            1633036860000,
            Status::Failure,
            "Record number 1".to_string(),
        );

        let mut cursor = Cursor::new(Vec::new());
        assert!(record.to_bin(&mut cursor).is_ok());
        assert_eq!(
            cursor.into_inner(),
            [
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
            ]
        )
    }
}
