use parser::YPBank;
use std::env;
use thiserror::Error;

fn help() -> &'static str {
    r#"Usage:
    comparer --file1 [FILE] --format1 [FORMAT] --file2 [FILE] --format2 [FORMAT]

Options:
    --file1             First file to read
    --format1           Data format in the first file to read
    --file2             Second file to read
    --format2           Data format in the second file to read
    --help              Print this message
"#
}

/// Формат данных.
enum Format {
    /// Текстовый формат.
    Text,

    /// CSV-формат.
    Csv,

    /// Бинарный формат.
    Bin,
}

/// Ошибка парсинга формата данных.
#[derive(Error, Debug)]
enum InputFormatError {
    /// Некорректный формат.
    #[error("Invalid format: {0}")]
    UnknownFormat(String),
}

impl TryFrom<&str> for Format {
    type Error = InputFormatError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(Self::Text),
            "csv" => Ok(Self::Csv),
            "bin" => Ok(Self::Bin),
            _ => Err(InputFormatError::UnknownFormat(value.to_string())),
        }
    }
}

macro_rules! open_and_read {
    ($file:expr, $format:expr, $name:literal) => {{
        let mut file = std::fs::File::open($file)
            .unwrap_or_else(|e| panic!("Error reading input file {}: {}", $name, e));
        match $format {
            Format::Text => YPBank::read_from_text(&mut file),
            Format::Csv => YPBank::read_from_csv(&mut file),
            Format::Bin => YPBank::read_from_bin(&mut file),
        }
        .unwrap_or_else(|e| panic!("Error reading data from file: {}", e.to_string()))
    }};
}

fn main() {
    let mut args = env::args();

    if args.len() != 9 {
        println!("{}", help());
        return;
    }

    args.next();

    let mut file1 = String::default();
    let mut file2 = String::default();
    let mut format1: Option<Format> = None;
    let mut format2: Option<Format> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{}", help());
                return;
            }
            file if file == "--file1" || file == "--file2" => {
                if let Some(filename) = args.next() {
                    if file == "--file1" {
                        file1 = filename;
                    } else {
                        file2 = filename;
                    }
                } else {
                    panic!("No input file specified after parameter {}!", file);
                }
            }
            format_key if format_key == "--format1" || format_key == "--format2" => {
                if let Some(format) = args.next() {
                    match format.as_str().try_into() {
                        Ok(fmt) => {
                            if format_key == "--format1" {
                                format1 = Some(fmt);
                            } else {
                                format2 = Some(fmt);
                            }
                        }
                        Err(e) => {
                            panic!("{}", e.to_string());
                        }
                    }
                } else {
                    panic!("Input file format not specified {}!", format_key);
                }
            }
            _ => {
                panic!("An unknown parameter was passed!");
            }
        }
    }

    if file1.is_empty() {
        panic!("No input file specified --file1!");
    }

    if file2.is_empty() {
        panic!("No input file specified --file2!");
    }

    let format1 = format1.unwrap_or_else(|| panic!("File format not specified --format1!"));
    let format2 = format2.unwrap_or_else(|| panic!("File format not specified --format2!"));

    let data1 = open_and_read!(file1.clone(), format1, "--file1");
    let data2 = open_and_read!(file2.clone(), format2, "--file2");

    if data1.records.len() != data2.records.len() {
        panic!(
            "The number of transactions in the files differs ({} != {})!",
            data1.records.len(),
            data2.records.len()
        );
    }

    match data1
        .records
        .iter()
        .zip(data2.records.iter())
        .position(|(r1, r2)| r1 != r2)
    {
        Some(idx) => println!("Transactions numbered {} are different!", idx + 1),
        None => println!(
            "Transactions in files `{}` and `{}` are completely identical!",
            file1, file2
        ),
    };
}
