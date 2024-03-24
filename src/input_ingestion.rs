use std::{
    env,
    fs::{self, File},
};
use thiserror::Error;

use csv::{Reader, ReaderBuilder, Trim};

#[derive(Debug, Error)]
pub enum InputAccessError {
    #[error("argument with the CSV input filename was not found")]
    MissingInputFilename,
    #[error("file `{0}` was not found")]
    FileNotFound(String),
    #[error("failed to create deserializer: {0}")]
    UnableToCreateReader(String),
}

// We expect to run the program like:
// cargo run -- transactions.csv > accounts.csv
// Hence we use the first argument as input filename.
pub fn get_input_filename() -> Option<String> {
    env::args().nth(1)
}

pub fn input_filename() -> Result<String, InputAccessError> {
    match get_input_filename() {
        None => Err(InputAccessError::MissingInputFilename),
        Some(filename) => {
            let file_exists = fs::metadata(&filename).is_ok();
            if !file_exists {
                return Err(InputAccessError::FileNotFound(filename));
            }
            Ok(filename)
        }
    }
}

pub fn get_csv_reader(path: String) -> Result<Reader<File>, InputAccessError> {
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_path(path);
    match reader {
        Ok(r) => Ok(r),
        Err(e) => Err(InputAccessError::UnableToCreateReader(e.to_string())),
    }
}
