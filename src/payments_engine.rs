use std::{error::Error, fs::File};

use csv::Reader;

use crate::models::{account::{Account, Accounts}, transaction::Transaction};

pub struct PaymentsEngine {}

impl PaymentsEngine {
    pub fn process_transactions_from(mut reader: Reader<File>) -> Result<Accounts, Box<dyn Error>> {
        let mut accounts_by_client_id = Accounts::default();
        for tx in reader.deserialize::<Transaction>() {
            // println!("TX: {:?}", tx?);
            let transaction = tx?;

            match accounts_by_client_id.get_mut(&transaction.client_id) {
              None => {
                // The account wasn't found. Lets lazily add it here.
                accounts_by_client_id.insert(transaction.client_id, Account::new(transaction.client_id));

              },
              Some(_) => {
                // There was an account already
              }
            };
        };

        Ok(accounts_by_client_id)
    }
}
