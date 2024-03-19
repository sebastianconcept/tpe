use std::error::Error;

use tpe::input_digestion::{transactions_iter};

fn main() -> Result<(), Box<dyn Error>> {
    // process_input()?;
    let transactions = transactions_iter()?;
    println!("Done well {:?}", transactions);
    Ok(())
}
