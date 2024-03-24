use std::{fs::File, process};

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
    ) -> Result<(), TransactionProcessingError> {
        // With flatten here it ignores issues during parsing
        for tx in reader.deserialize::<Transaction>().flatten() {
            match self.process(tx) {
                Ok(()) => {}
                Err(TransactionProcessingError::TargetAccountLocked(_tx_id)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to a operation issues queue for follow up?
                }
                Err(TransactionProcessingError::NotFound(_tx_id)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to an operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InsufficientAvailableFunds(_tx_id, _amount)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to an operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InconsistentOperation) => {
                    // Note: In a real payment engine, cases like this would typically generate system events
                    // that are published to a high-capacity shared queue, which can be observed by other
                    // programs. These observer programs can be decoupled client applications with the
                    // appropriate concerns to handle policies for reacting to such cases.
                    //
                    // For example, should they just re-try after a while? Or, if an operation was inconsistent,
                    // it might require logging or queuing for investigation with a partner.
                }
                Err(TransactionProcessingError::DeserializationError(_e)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system the input should be logged or published an event to an operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InputError(e)) => {
                    // No access to input, cannot operate further
                    println!("{}", e);
                    process::exit(1)
                }
            }
        }
        Ok(())
    }

    pub fn process_transactions_streaming_input(
        &mut self,
        mut reader: Reader<&[u8]>,
    ) -> Result<(), TransactionProcessingError> {
        // With flatten here it ignores issues during parsing
        for tx in reader.deserialize::<Transaction>().flatten() {
            match self.process(tx) {
                Ok(()) => {}
                Err(TransactionProcessingError::TargetAccountLocked(_tx_id)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to a operation issues queue for follow up?
                }
                Err(TransactionProcessingError::NotFound(_tx_id)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to a operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InsufficientAvailableFunds(_tx_id, _amount)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system tx_id should be logged or published an event to a operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InconsistentOperation) => {
                    // Note: same observations as noted in `process_transactions_from`
                }
                Err(TransactionProcessingError::DeserializationError(_e)) => {
                    // Ignore and continue processing the next input operation.
                    // In a real system the input should be logged or published an event to an operation issues queue for follow up?
                }
                Err(TransactionProcessingError::InputError(e)) => {
                    // No access to input, cannot operate further
                    println!("{}", e);
                    process::exit(1)
                }
            }
        }
        Ok(())
    }

    pub fn process(&mut self, transaction: Transaction) -> Result<(), TransactionProcessingError> {
        let account = self
            .accounts
            .entry(transaction.client_id)
            .or_insert(Account::new(transaction.client_id));
        account.process(transaction, &mut self.transactions, &mut self.disputes)?;
        Ok(())
    }
}
