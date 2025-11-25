use clap::Parser;
use parser::{
    YPBankImpl,
    errors::{FormatError, ReadError, WriteError},
};
use thiserror::Error;

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
    #[error(transparent)]
    UnknownFormat(#[from] FormatError),

    #[error("The number of transactions in the files differs ({len1} != {len2})!")]
    UnequalData { len1: usize, len2: usize },

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

    let file1 = args.file1;
    let file2 = args.file2;
    let format1 = convert_format!(args.format1.as_str());
    let format2 = convert_format!(args.format2.as_str());

    let records1 = open_and_read!(file1.clone(), format1);
    let records2 = open_and_read!(file2.clone(), format2);

    if records1.len() != records2.len() {
        return Err(CliError::UnequalData {
            len1: records1.len(),
            len2: records2.len(),
        });
    }

    match records1
        .iter()
        .zip(records2.iter())
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
            CliError::ReadData(_) => -3,
            CliError::WriteData(_) => -4,
            CliError::UnequalData { .. } => -5,
            CliError::TooBigFile => -6,
        };

        eprintln!("{}", err);
        std::process::exit(exit_code);
    }
}
