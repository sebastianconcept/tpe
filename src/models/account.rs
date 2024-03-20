use std::{collections::HashMap, process::Output};

use crate::models::transaction::{TransactionProcessingError, TransactionType};

use super::{
    shared::{Amount, ClientID},
    transaction::{Transaction, Transactions},
};

// An index to reach accounts by client ID
pub type Accounts = HashMap<ClientID, Account>;
#[derive(Debug)]
pub struct Account {
    client_id: ClientID,
    pub total: Amount,
    pub held: Amount,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: ClientID) -> Self {
        Self {
            client_id,
            total: Amount::from(0),
            held: Amount::from(0),
            locked: false,
        }
    }

    pub fn get_available(&self) -> Amount {
        self.total - self.held
    }

    pub fn process(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        if self.locked {
            // For all types of operations, the locked account will prevent further processing of any transaction.
            return Err(TransactionProcessingError::TargetAccountLocked(tx.tx_id));
        }

        // But if not locked, can move on processing every case
        match tx.tx_type {
            TransactionType::Deposit => self.process_deposit(tx, transactions)?,
            TransactionType::Withdrawal => self.process_withdrawal(tx, transactions)?,
            TransactionType::Dispute => self.process_dispute(tx, transactions)?,
            TransactionType::Resolve => self.process_resolve(tx, transactions)?,
            TransactionType::Chargeback => self.process_chargeback(tx, transactions)?,
        }
        Ok(())
    }

    fn process_deposit(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match tx.amount {
            None => {
                unreachable!()
            }
            Some(val) => {
                // Adds the deposit transaction amount to the total 👀
                self.total += val;
                let tx_id = tx.tx_id;
                transactions.insert(tx.tx_id, tx);
                Ok(())
            }
        }
    }

    fn process_withdrawal(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match tx.amount {
            None => {
                unreachable!()
            }
            Some(val) => {
                if val > self.get_available() {
                    // Reject processing if there isn't enough available
                    return Err(TransactionProcessingError::InsufficientFunds((
                        tx.tx_id, val,
                    )));
                }
                // Subtracts the withdrawal transaction amount from the total 👀
                self.total -= val;
                let tx_id = tx.tx_id;
                transactions.insert(tx.tx_id, tx);
                Ok(())
            }
        }
    }

    fn process_dispute(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                if let Some(val) = t.amount {
                    if val > self.get_available() {
                        // Reject processing if there isn't enough available
                        return Err(TransactionProcessingError::InsufficientFunds((
                            tx.tx_id, val,
                        )));
                    }
                    // Disputed, hence increase in val the value held 👀
                    self.held += val;
                } else {
                    unreachable!();
                }
                Ok(())
            }
        }
    }

    fn process_resolve(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                if let Some(val) = t.amount {
                    // Resolved, hence decrease in val the value held 👀
                    self.held -= val;
                } else {
                    unreachable!();
                }
                Ok(())
            }
        }
    }

    fn process_chargeback(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                if let Some(val) = t.amount {
                    // Chargeback, hence decrease the held and total values
                    // in this account by the previously disputed transaction's value val and freeze the account 👀
                    self.total -= val;
                    self.held -= val;
                    self.locked = true;
                } else {
                    unreachable!();
                }
                Ok(())
            }
        }
    }

    // Renders this account in its current state following the expected format
    // as per `Rust Test.pdf`
    pub fn render_as_output_line(&self) {
        let output_line = format!(
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client_id,
            self.get_available(),
            self.held,
            self.total,
            self.locked
        );
        println!("{}", output_line);
    }
}
