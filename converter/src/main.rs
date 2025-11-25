use clap::Parser;
use parser::{errors::{ReadError, WriteError}, YPBank};
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

/// Ошибка парсинга данных.
#[derive(Error, Debug)]
enum CliError {
    /// Некорректный формат данных.
    #[error("Invalid format: {0}")]
    UnknownFormat(String),

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

macro_rules! convert_format {
    ($input:expr) => {
        $input
            .as_str()
            .try_into()?
    };
}

fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let input_filename = args.input;
    let input_format: Format = convert_format!(args.input_format);
    let output_format: Format = convert_format!(args.output_format);

    let mut input_file = std::fs::File::open(input_filename)?;

    let data = match input_format {
        Format::Text => YPBank::read_from_text(&mut input_file),
        Format::Csv => YPBank::read_from_csv(&mut input_file),
        Format::Bin => YPBank::read_from_bin(&mut input_file),
    }?;

    let mut stdout = std::io::stdout().lock();

    match output_format {
        Format::Text => data.write_to_text(&mut stdout),
        Format::Csv => data.write_to_csv(&mut stdout),
        Format::Bin => data.write_to_bin(&mut stdout),
    }?;

    stdout
        .flush()?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        let exit_code = match err {
            CliError::UnknownFormat(_) => -1,
            CliError::Io(_) => -2,
            CliError::ReadDataError(_) => -3,
            CliError::WriteDataError(_) => -4,
        };

        eprintln!("{}", err.to_string());
        std::process::exit(exit_code);
    }
}