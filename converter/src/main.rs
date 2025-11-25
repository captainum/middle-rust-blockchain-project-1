use clap::Parser;
use parser::YPBank;
use std::io::Write;
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to read
    #[arg(long, value_name = "FILE")]
    input: std::path::PathBuf,

    /// Data format in the file to read
    #[clap(long, value_name = "FORMAT")]
    input_format: String,

    /// Output data format
    #[clap(long, value_name = "FORMAT")]
    output_format: String,
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

macro_rules! convert_format {
    ($input:expr) => {
        $input
            .as_str()
            .try_into()
            .unwrap_or_else(|e: InputFormatError| panic!("{}", e.to_string()))
    };
}

fn main() {
    let args = Args::parse();

    let input_filename = args.input;
    let input_format: Format = convert_format!(args.input_format);
    let output_format: Format = convert_format!(args.output_format);

    let mut input_file = std::fs::File::open(input_filename)
        .unwrap_or_else(|e| panic!("Error reading input file: {}", e));

    let data = match input_format {
        Format::Text => YPBank::read_from_text(&mut input_file),
        Format::Csv => YPBank::read_from_csv(&mut input_file),
        Format::Bin => YPBank::read_from_bin(&mut input_file),
    }
    .unwrap_or_else(|e| panic!("Error reading data from file: {}", e));

    let mut stdout = std::io::stdout().lock();

    match output_format {
        Format::Text => data.write_to_text(&mut stdout),
        Format::Csv => data.write_to_csv(&mut stdout),
        Format::Bin => data.write_to_bin(&mut stdout),
    }
    .unwrap_or_else(|e| panic!("Data output error: {}", e));

    stdout
        .flush()
        .unwrap_or_else(|e| panic!("Error flushing stdout: {}", e));
}
