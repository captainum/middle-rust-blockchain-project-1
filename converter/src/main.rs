use clap::Parser;
use parser::{
    YPBankImpl,
    errors::{FormatError, ReadError, WriteError},
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
    #[error(transparent)]
    UnknownFormat(#[from] FormatError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ReadData(#[from] ReadError),

    #[error(transparent)]
    WriteData(#[from] WriteError),

    #[error("File is too big!")]
    TooBigFile,
}

macro_rules! open_and_read {
    ($file:expr, $format:expr) => {{
        if std::fs::metadata(&$file)?.len() > 1024 * 1024 * 1024 {
            return Err(CliError::TooBigFile);
        }

        let mut file = std::fs::File::open($file)?;
        $format.read_from(&mut file)?
    }};
}

macro_rules! convert_format {
    ($input:expr) => {
        YPBankImpl::try_from($input)?
    };
}

fn run() -> Result<(), CliError> {
    let args = Args::parse();

    let input_filename = args.input;
    let input_format = convert_format!(args.input_format.as_str());
    let output_format = convert_format!(args.output_format.as_str());

    let records = open_and_read!(input_filename, input_format);

    let mut stdout = std::io::stdout().lock();

    output_format.write_to(records, &mut stdout)?;

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
