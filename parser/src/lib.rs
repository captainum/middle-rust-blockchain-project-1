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

pub mod errors;
pub mod record;
mod text_format;
mod bin_format;
mod csv_format;

use errors::{ReadError, WriteError};
use std::io::{Read, Write};

pub use text_format::YPBankText;
pub use csv_format::YPBankCsv;
pub use bin_format::YPBankBin;

/// Трейт для парсинга и хранения данных о банковских операциях.
pub trait YPBank: Sized {
    /// Считать данные о банковских операциях.
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError>;

    /// Записать данные о банковских операциях.
    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError>;
}

#[cfg(test)]
mod tests {
    use super::record::status::Status;
    use super::record::tx_type::TxType;
    use super::record::Record;

    pub(super) fn get_data_to_write() -> Vec<Record> {
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
}
