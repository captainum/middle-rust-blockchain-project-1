use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use super::errors::{ReadError, WriteError};
use super::record::Record;
use super::YPBank;

#[derive(Debug)]
pub struct YPBankBin {
    /// Записи о банковских операциях.
    pub records: Vec<Record>,
}

impl YPBank for YPBankBin {
    // Считать данные о банковских операциях в бинарном формате.
    fn read_from<R: Read>(r: &mut R) -> Result<Self, ReadError> {
        let mut reader = BufReader::new(r);

        let mut records: Vec<Record> = vec![];

        while !reader.fill_buf()?.is_empty() {
            records.push(Record::from_bin(&mut reader)?);
        }

        Ok(Self { records })
    }

    /// Записать данные о банковских операциях в бинарном формате.
    fn write_to<W: Write>(&self, w: &mut W) -> Result<(), WriteError> {
        let mut writer = BufWriter::new(w);

        for record in &self.records {
            record.to_bin(&mut writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::status::Status;
    use crate::record::tx_type::TxType;
    use crate::record::errors::ParseRecordFromBinError;
    use std::io::Cursor;

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
        assert!(matches!(
            result,
            ReadError::FromBin(ParseRecordFromBinError::InvalidMagicNumber)
        ));
        assert_eq!(
            result.to_string(),
            "Binary format parsing error: Invalid magic number"
        );
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
        let records = crate::tests::get_data_to_write();

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
