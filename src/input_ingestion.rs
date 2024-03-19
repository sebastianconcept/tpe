use std::{
    env, error, fmt,
    fs::{self, File},
};

use csv::{Reader, ReaderBuilder, Trim};

use crate::transactions::Transaction;

#[derive(Debug)]
pub enum InvalidInput {
    MissingInputFilename,
    FileNotFound(String),
    FormatError(String),
}

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid input")
    }
}

impl error::Error for InvalidInput {}

// We expect to run the program like:
// cargo run -- transactions.csv > accounts.csv
// Hence we use the first argument as input filename.
pub fn get_input_filename() -> Option<String> {
    env::args().nth(1)
}

pub fn input_filename() -> Result<String, InvalidInput> {
    match get_input_filename() {
        None => Err(InvalidInput::MissingInputFilename),
        Some(filename) => {
            let file_exists = fs::metadata(&filename).is_ok();
            if !file_exists {
                return Err(InvalidInput::FileNotFound(filename));
            }
            Ok(filename)
        }
    }
}

pub fn get_csv_reader(path: String) -> Result<Reader<File>, InvalidInput> {
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_path(path);
    match reader {
        Ok(r) => Ok(r),
        Err(e) => Err(InvalidInput::FormatError(e.to_string())),
    }
}
