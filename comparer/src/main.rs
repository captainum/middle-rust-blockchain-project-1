use clap::Parser;
use parser::YPBank;
use thiserror::Error;
use parser::errors::{ReadError, WriteError};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// First file to read
    #[arg(long, value_name = "FILE")]
    file1: std::path::PathBuf,

    /// Data format in the first file to read
    #[clap(long, value_name = "FORMAT")]
    format1: String,

    /// Second file to read
    #[arg(long, value_name = "FILE")]
    file2: std::path::PathBuf,

    /// Data format in the second file to read
    #[clap(long, value_name = "FORMAT")]
    format2: String,
}

/// Ошибка парсинга данных.
#[derive(Error, Debug)]
enum CliError {
    /// Некорректный формат данных.
    #[error("Invalid format: {0}")]
    UnknownFormat(String),

    #[error("The number of transactions in the files differs ({len1} != {len2})!")]
    UnequalData {
        len1: usize,
        len2: usize,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ReadDataError(#[from] ReadError),

    #[error(transparent)]
    WriteDataError(#[from] WriteError),
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

impl TryFrom<&str> for Format {
    type Error = CliError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(Self::Text),
            "csv" => Ok(Self::Csv),
            "bin" => Ok(Self::Bin),
            _ => Err(CliError::UnknownFormat(value.to_string())),
        }
    }
}

macro_rules! open_and_read {
    ($file:expr, $format:expr, $name:literal) => {{
        let mut file = std::fs::File::open($file)?;
        match $format {
            Format::Text => YPBank::read_from_text(&mut file),
            Format::Csv => YPBank::read_from_csv(&mut file),
            Format::Bin => YPBank::read_from_bin(&mut file),
        }
    }};
}

macro_rules! convert_format {
    ($input:expr) => {
        $input
            .as_str()
            .try_into()?
    };
}

fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let file1 = args.file1;
    let file2 = args.file2;
    let format1: Format = convert_format!(args.format1);
    let format2: Format = convert_format!(args.format2);

    let data1 = open_and_read!(file1.clone(), format1, "--file1")?;
    let data2 = open_and_read!(file2.clone(), format2, "--file2")?;

    if data1.records.len() != data2.records.len() {
        return Err(CliError::UnequalData {
            len1: data1.records.len(),
            len2: data2.records.len(),
        });
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
            file1.to_str().unwrap_or("file1"),
            file2.to_str().unwrap_or("file2")
        ),
    };

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        let exit_code = match err {
            CliError::UnknownFormat(_) => -1,
            CliError::Io(_) => -2,
            CliError::ReadDataError(_) => -3,
            CliError::WriteDataError(_) => -4,
            CliError::UnequalData { .. } => -5,
        };

        eprintln!("{}", err.to_string());
        std::process::exit(exit_code);
    }
}