use std::error::Error;

use tpe::{input_ingestion::{get_csv_reader, input_filename}, models::{account::Accounts, transaction::Transaction}};

fn process_input() -> Result<(), Box<dyn Error>> {
    let mut reader = get_csv_reader(input_filename()?).expect("CSV reader could not be created");
    let mut accounts = Accounts::default();
    for tx in reader.deserialize::<Transaction>() {
        println!("TX: {:?}", tx?);

    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    process_input()?;
    Ok(())
}
