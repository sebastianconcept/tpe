use std::collections::HashMap;

use crate::models::transaction::{TransactionProcessingError, TransactionType};

use super::{
    disputes::Disputes,
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
        disputes: &mut Disputes,
    ) -> Result<(), TransactionProcessingError> {
        if self.locked {
            // For all types of operations, the locked account will prevent further processing of any transaction.
            return Err(TransactionProcessingError::TargetAccountLocked(tx.tx_id));
        }

        // But if not locked, can move on processing every case
        match tx.tx_type {
            TransactionType::Deposit => self.process_deposit(tx, transactions)?,
            TransactionType::Withdrawal => self.process_withdrawal(tx, transactions)?,
            TransactionType::Dispute => self.process_dispute(tx, transactions, disputes)?,
            TransactionType::Resolve => self.process_resolve(tx, transactions, disputes)?,
            TransactionType::Chargeback => self.process_chargeback(tx, transactions, disputes)?,
        }
        Ok(())
    }

    // {
    //     if let Err(err) = self.process_resolve(tx, transactions, disputes) {
    //         match err {
    //             TransactionProcessingError => {}
    //             _ => return Err(err),
    //         }
    //         {}
    //     }
    // },
    // match self.process_resolve(tx, transactions, disputes) {
    //     Err(err) => {
    //         match err {
    //             TransactionProcessingError => {}
    //             _ => return Err(err),
    //         }
    //         {}
    //     }
    //     Ok(_) => {}
    // },

    fn process_deposit(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        match tx.amount {
            None => {
                unreachable!("There is always a valid amount for deposits")
            }
            Some(val) => {
                // If there is a deposit at tx_id, then ignore the repeated deposit considering it as partner inconsistency 👀
                transactions.entry(tx.tx_id).or_insert_with(|| {
                    // Or, since it's absent, add the deposit transaction to the record and update the account total amount 👀
                    self.total += val;
                    tx
                });
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
                unreachable!("There is always a valid amount for withdrawals")
            }
            Some(val) => {
                if val > self.get_available() {
                    // Reject processing if there isn't enough available
                    return Err(TransactionProcessingError::InsufficientAvailableFunds((
                        tx.tx_id, val,
                    )));
                }
                // Subtracts the withdrawal transaction amount from the total 👀
                self.total -= val;
                transactions.insert(tx.tx_id, tx);
                Ok(())
            }
        }
    }

    fn process_dispute(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
        disputes: &mut Disputes,
    ) -> Result<(), TransactionProcessingError> {
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                if let Some(val) = t.amount {
                    if val > self.get_available() {
                        // Reject processing if there isn't enough available
                        return Err(TransactionProcessingError::InsufficientAvailableFunds((
                            tx.tx_id, val,
                        )));
                    }
                    // Disputed, hence increase in val the value held 👀
                    self.held += val;
                } else {
                    unreachable!(
                        "There is always a valid amount for transactions aimed by a dispute"
                    );
                }
                Ok(())
            }
        }
    }

    fn process_resolve(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
        disputes: &mut Disputes,
    ) -> Result<(), TransactionProcessingError> {
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                if let Some(val) = t.amount {
                    // Resolved, hence decrease in val the value held 👀
                    self.held -= val;
                } else {
                    unreachable!(
                        "There is always a valid amount for transactions aimed by a resolution"
                    );
                }
                Ok(())
            }
        }
    }

    fn process_chargeback(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
        disputes: &mut Disputes,
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
                    unreachable!(
                        "There is always a valid amount for transactions aimed by a chargeback"
                    );
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
