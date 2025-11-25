use clap::Parser;
use parser::{
    YPBank, YPBankBin, YPBankCsv, YPBankText,
    errors::{ReadError, WriteError},
};
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
    ReadData(#[from] ReadError),

    #[error(transparent)]
    WriteData(#[from] WriteError),

    #[error("File is too big!")]
    TooBigFile,
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
        $input.as_str().try_into()?
    };
}

fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let input_filename = args.input;
    let input_format: Format = convert_format!(args.input_format);
    let output_format: Format = convert_format!(args.output_format);

    if std::fs::metadata(&input_filename)?.len() > 1024 * 1024 * 1024 {
        return Err(CliError::TooBigFile);
    }

    let mut input_file = std::fs::File::open(input_filename)?;

    let records = match input_format {
        Format::Text => YPBankText::read_from(&mut input_file)?.records,
        Format::Csv => YPBankCsv::read_from(&mut input_file)?.records,
        Format::Bin => YPBankBin::read_from(&mut input_file)?.records,
    };

    let mut stdout = std::io::stdout().lock();

    match output_format {
        Format::Text => YPBankText { records }.write_to(&mut stdout),
        Format::Csv => YPBankCsv { records }.write_to(&mut stdout),
        Format::Bin => YPBankBin { records }.write_to(&mut stdout),
    }?;

    stdout.flush()?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        let exit_code = match err {
            CliError::UnknownFormat(_) => -1,
            CliError::Io(_) => -2,
            CliError::ReadData(_) => -3,
            CliError::WriteData(_) => -4,
            CliError::TooBigFile => -5,
        };

        eprintln!("{}", err);
        std::process::exit(exit_code);
    }
}
