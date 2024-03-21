use std::{error::Error, fs::File};

use csv::Reader;

use crate::models::{
    account::{Account, Accounts},
    disputes::Disputes,
    transaction::{Transaction, Transactions},
};

#[derive(Default)]
pub struct PaymentsEngine {
    pub accounts: Accounts,
    pub transactions: Transactions,
    pub disputes: Disputes,
}

impl PaymentsEngine {
    pub fn process_transactions_from(
        &mut self,
        mut reader: Reader<File>,
    ) -> Result<(), Box<dyn Error>> {
        for tx in reader.deserialize::<Transaction>() {
            self.process(tx?)?
        }
        Ok(())
    }

    pub fn process_transactions_using(
        &mut self,
        mut reader: Reader<&[u8]>,
    ) -> Result<(), Box<dyn Error>> {
        for tx in reader.deserialize::<Transaction>() {
            match tx {
                Ok(t) => self.process(t)?,
                Err(_err) => {
                    // Ignoring transactions that had an issue on parsing
                }
            }
        }
        Ok(())
    }

    pub fn process(&mut self, transaction: Transaction) -> Result<(), Box<dyn Error>> {
        let account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert(Account::new(transaction.client_id));
        account.process(transaction, &mut self.transactions, &mut self.disputes)?;
        Ok(())
    }
}
