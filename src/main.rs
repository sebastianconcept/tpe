use std::error::Error;

use tpe::input_ingestion::{transactions_reader};

fn main() -> Result<(), Box<dyn Error>> {
    // process_input()?;
    let mut transactions_reader = transactions_reader()?;
    for tx in transactions_reader.records() {
        println!("TX: {:?}", tx.unwrap());
    }
    Ok(())
}
