use parser::YPBank;
use std::env;
use thiserror::Error;
fn help() -> &'static str {
    r#"Usage:
    converter --input [FILE] --input-format [FORMAT] --output-format [FORMAT]

Options:
    --input             File to read
    --input-format      Data format in the file to read
    --output-format     Output data format
    --help              Print this message
"#
}

enum Format {
    TEXT,
    CSV,
    BIN,
}

#[derive(Error, Debug)]
enum InputFormatError {
    #[error("Invalid format: {0}")]
    UnknownFormat(String),
}

impl TryFrom<&str> for Format {
    type Error = InputFormatError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "text" => Ok(Self::TEXT),
            "csv" => Ok(Self::CSV),
            "bin" => Ok(Self::BIN),
            _ => Err(InputFormatError::UnknownFormat(value.to_string())),
        }
    }
}

fn main() {
    let mut args = env::args();

    if args.len() != 7 {
        println!("{}", help());
        return;
    }

    args.next();

    let mut input_filename = String::default();
    let mut input_format: Option<Format> = None;
    let mut output_format: Option<Format> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{}", help());
                return;
            }
            "--input" => {
                input_filename = args
                    .next()
                    .unwrap_or_else(|| panic!("No input file specified!"));
            }
            val if val == "--input-format" || val == "--output-format" => {
                if let Some(format) = args.next() {
                    match format.as_str().try_into() {
                        Ok(fmt) => {
                            if val == "--input-format" {
                                input_format = Some(fmt);
                            } else {
                                output_format = Some(fmt);
                            }
                        }
                        Err(e) => {
                            panic!("{}", e.to_string());
                        }
                    }
                } else {
                    panic!("Input file format not specified!");
                }
            }
            _ => {
                panic!("An unknown parameter was passed!");
            }
        }
    }

    if input_filename.is_empty() {
        panic!("No input file specified!");
    }

    let input_format = input_format.unwrap_or_else(|| panic!("Input file format not specified!"));

    let output_format =
        output_format.unwrap_or_else(|| panic!("Output data format not specified!"));

    let mut input_file = std::fs::File::open(input_filename)
        .unwrap_or_else(|e| panic!("Error reading input file: {}", e.to_string()));

    let data = match input_format {
        Format::TEXT => YPBank::read_from_text(&mut input_file),
        Format::CSV => YPBank::read_from_csv(&mut input_file),
        Format::BIN => YPBank::read_from_bin(&mut input_file),
    }
    .unwrap_or_else(|e| panic!("Error reading data from file: {}", e.to_string()));

    match output_format {
        Format::TEXT => data.write_to_text(&mut std::io::stdout()),
        Format::CSV => data.write_to_csv(&mut std::io::stdout()),
        Format::BIN => data.write_to_bin(&mut std::io::stdout()),
    }
    .unwrap_or_else(|e| panic!("Data output error: {}", e.to_string()));
}
