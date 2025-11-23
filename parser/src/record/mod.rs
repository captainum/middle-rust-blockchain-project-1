//! Модуль описания записи о транзакции.

use std::collections::HashMap;
use std::fmt;
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
#[derive(Debug, Clone)]
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
        let status =
            value
                .try_into()
                .map_err(|e: ParseStatusError| ParseValueError::InvalidValue {
                    value: value.to_string(),
                    description: e.to_string(),
                })?;

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

            line.truncate(line.len() - 1);

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

        line.truncate(line.len() - 1);

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
                "{},{},{},{},{},{},{},{}",
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
        let tx_type_raw = r.read_u8()?;
        let tx_type = tx_type_raw.try_into().map_err(|e: ParseTxTypeError| {
            ParseValueError::InvalidValue {
                value: tx_type_raw.to_string(),
                description: e.to_string(),
            }
        })?;
        let from_user_id = r.read_u64::<BigEndian>()?;
        let to_user_id = r.read_u64::<BigEndian>()?;
        let amount = r.read_u64::<BigEndian>()?;
        let timestamp = r.read_u64::<BigEndian>()?;
        let status_raw = r.read_u8()?;
        let status =
            status_raw
                .try_into()
                .map_err(|e: ParseStatusError| ParseValueError::InvalidValue {
                    value: status_raw.to_string(),
                    description: e.to_string(),
                })?;
        let desc_len = r.read_u32::<BigEndian>()?;
        let mut description = String::new();

        if desc_len > 0 {
            let mut buffer = vec![0u8; desc_len as usize];
            r.read_exact(&mut buffer)?;

            description =
                String::from_utf8(buffer.clone()).map_err(|e| ParseValueError::InvalidValue {
                    value: String::from_utf8_lossy(&buffer).to_string(),
                    description: e.to_string(),
                })?;
        }

        Ok(Self::new(
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        ))
    }

    /// Записать данные о транзакции в указанное место в бинарном формате.
    pub fn to_bin<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_all(&Self::BINARY_MAGIC)?;

        let record_size = Self::BINARY_MIN_RECORD_SIZE + self.description.len() as u32;
        w.write_u32::<BigEndian>(record_size)?;

        w.write_u64::<BigEndian>(self.tx_id)?;
        w.write_u8(self.tx_type as u8)?;
        w.write_u64::<BigEndian>(self.from_user_id)?;
        w.write_u64::<BigEndian>(self.to_user_id)?;
        w.write_u64::<BigEndian>(self.amount)?;
        w.write_u64::<BigEndian>(self.timestamp)?;
        w.write_u8(self.status as u8)?;
        w.write_u32::<BigEndian>(self.description.len() as u32)?;
        w.write_all(self.description.as_bytes())
    }
}

/// Реализация трейта [`fmt::Display`] для [`Record`].
impl fmt::Display for Record {
    /// Реализация метода [`fmt::Display::fmt`] для [`Record`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tx_id: {}, tx_type: {}, from_user_id: {}, to_user_id: {}, amount: {}, timestamp: {}, status: {}, description: {}",
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
    use crate::record::errors::ParseKeyError;
    use rstest::rstest;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_correct_record() {
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
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

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
        let mut reader = BufReader::new(Cursor::new(
            vec![
                "TX_ID: 1",
                "TX_TYPE: DEPOSIT",
                "FROM_USER_ID: 1",
                line.as_str(),
                "TO_USER_ID: 2",
                "AMOUNT: 100",
                "TIMESTAMP: 1623228800",
                "STATUS: SUCCESS",
                "DESCRIPTION: \"Terminal deposit\"",
            ]
            .join("\n"),
        ));

        let result = Record::from_text(&mut reader);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            ParseRecordFromTxtError::UnexpectedError(format!(
                "Could not parse string by space delimiter: {}",
                line
            ),)
        );
    }

    #[test]
    fn test_no_colon_found() {
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

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            ParseRecordFromTxtError::ColonNotFound("TO_USER_ID".to_string())
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
        let mut reader =
            BufReader::new(Cursor::new(vec![format!("{}: {}", key, value)].join("\n")));

        let result = Record::from_text(&mut reader);

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            ParseRecordFromTxtError::InvalidValue(ParseValueError::InvalidValue {
                value: value.to_string(),
                description,
            })
        );
    }

    #[test]
    fn test_unexpected_key() {
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

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            ParseRecordFromTxtError::InvalidKey(ParseKeyError::InvalidKey(
                "UNEXPECTED_KEY".to_string()
            ))
        );
    }

    #[test]
    fn test_missing_key() {
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

        assert!(result.is_err());

        let result = result.err().unwrap();

        assert_eq!(
            result,
            ParseRecordFromTxtError::MissingKey("DESCRIPTION".to_string())
        );
    }
}
