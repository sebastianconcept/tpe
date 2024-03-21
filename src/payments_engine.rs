use std::{error::Error, fs::File};

use csv::Reader;

use crate::models::{
    account::{Account, Accounts},
    disputes::Disputes,
    transaction::{Transaction, TransactionProcessingError, Transactions},
};

#[derive(Default)]
pub struct PaymentsEngine {
    pub accounts: Accounts,
    pub transactions: Transactions,
    pub disputes: Disputes,
}

// This engine will process transactions and operations related to these and their respective accounts.
// It's designed to preserve the accounts integrity and continuous operation.
// For example, it ignores and move on processing the next piece of input when some input record could not be parsed or,
// after being parsed, when there was any `TransactionProcessingError` case that prevented completing an operation.
impl PaymentsEngine {
    pub fn process_transactions_from(
        &mut self,
        mut reader: Reader<File>,
    ) -> Result<(), Box<dyn Error>> {
        // With flatten here it ignores issues during parsing
        for tx in reader.deserialize::<Transaction>().flatten() {
            if let Err(e) = self.process(tx) {
                match e.downcast::<TransactionProcessingError>() {
                    Ok(_err) => {
                        // Some variant of TransactionProcessingError, move on processing the next
                    }
                    Err(_) => {
                        // Parsing errors on this one, move on processing the next
                    }
                }
            }
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
                Err(_) => {
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
