use std::error::Error;

use tpe::{
    input_ingestion::{get_csv_reader, input_filename},
    payments_engine::PaymentsEngine,
};

fn process_input() -> Result<(), Box<dyn Error>> {
    let reader = get_csv_reader(input_filename()?).expect("CSV reader could not be created");
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader)?;
    for (_, account) in pe.accounts.iter() {
        account.render_as_output_line();
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    process_input()?;
    Ok(())
}
