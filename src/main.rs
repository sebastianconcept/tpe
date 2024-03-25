use std::process;

use anyhow::Result;
use tpe::{
    input_ingestion::{get_csv_reader, input_filename},
    payments_engine::PaymentsEngine,
};

fn process_input() -> Result<()> {
    match get_csv_reader(input_filename()?) {
        Err(_) => {
            println!("Unable to create the input reader");
            process::exit(1);
        }
        Ok(reader) => {
            let mut pe = PaymentsEngine::default();
            pe.process_transactions_from(reader)?;
            for (_, account) in pe.accounts.iter() {
                account.render_as_output_line();
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    process_input()?;
    Ok(())
}
