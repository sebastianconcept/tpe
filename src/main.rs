use std::{env, error, fmt, fs};
use std::error::Error;

#[derive(Debug)]
enum InvalidInput {
    MissingInputFilename,
    FileNotFound(String),
}

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid input")
    }
}

impl error::Error for InvalidInput {}

// impl error::Error for InvalidInput {}

fn get_input_filename() -> Option<String> {
    env::args().nth(1)
}

fn digest_input() -> Result<String, InvalidInput> {
    match get_input_filename() {
        None => Err(InvalidInput::MissingInputFilename),
        Some(filename) => {
            let file_exists = fs::metadata(&filename).is_ok();
            if !file_exists {
                return Err(InvalidInput::FileNotFound(filename));
            }
            Ok("Input digested".to_string())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // process_input()?;
    digest_input()?;
    println!("Done well");
    Ok(())
}
