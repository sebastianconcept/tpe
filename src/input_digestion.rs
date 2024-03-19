use std::{env, error, fmt, fs::{self, File}};

use csv::{Reader, ReaderBuilder, Trim};

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


pub fn transactions_iter() -> Result<Reader<File>, InvalidInput> {
  get_transactions_iter(input_filename()?)
}

// We expect to run the program like:
// cargo run -- transactions.csv > accounts.csv
// Hence we use the first argument as input filename.
fn get_input_filename() -> Option<String> {
  env::args().nth(1)
}

fn input_filename() -> Result<String, InvalidInput> {
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

fn get_transactions_iter(filename: String) -> Result<Reader<File>, InvalidInput> {
    let path = filename;
    let result = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_path(path);
    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(InvalidInput::FormatError(e.to_string()))
    }
}
