use std::error::Error;

use tpe::{
    // input_ingestion::{get_csv_reader, input_filename},
    input_ingestion::{get_csv_reader, input_filename},
    models::{
        account::{self, Accounts},
        transaction::Transaction,
    },
    payments_engine::PaymentsEngine,
};

fn process_input() -> Result<(), Box<dyn Error>> {
    let mut reader = get_csv_reader(input_filename()?).expect("CSV reader could not be created");
    let pe = PaymentsEngine {};
    let accounts = tpe::payments_engine::PaymentsEngine::process_transactions_from(reader)?;
    for (client_id, account) in accounts.iter() {
        println!("This account {:?}", account);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    process_input()?;
    Ok(())
}
