use std::collections::HashMap;

use crate::models::transaction::{TransactionProcessingError, TransactionType};

use super::{
    disputes::{Dispute, Disputes},
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
            // For all types of operations, the locked account will prevent further processing of any kind.
            // The application should decide (handle) what to do with a TargetAccountLocked.
            return Err(TransactionProcessingError::TargetAccountLocked(tx.tx_id));
        }

        // But if not locked, it moves on processing every case
        match tx.tx_type {
            TransactionType::Deposit(amount) => self.process_deposit(tx, amount, transactions)?,
            TransactionType::Withdrawal(amount) => {
                self.process_withdrawal(tx, amount, transactions)?
            }
            TransactionType::Dispute => self.process_dispute(tx, transactions, disputes)?,
            TransactionType::Resolve => self.process_resolve(tx, transactions, disputes)?,
            TransactionType::Chargeback => self.process_chargeback(tx, transactions, disputes)?,
        }
        Ok(())
    }

    fn process_deposit(
        &mut self,
        tx: Transaction,
        amount: Amount,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        // If there is a deposit at tx_id, then ignore the repeated deposit considering it as partner inconsistency 👀
        transactions.entry(tx.tx_id).or_insert_with(|| {
            // Or, since it's absent, add the deposit transaction to the record and update the account total amount 👀
            // Note: If already present in transactions, it will be ignored.
            self.total += amount;
            tx
        });
        Ok(())
    }

    fn process_withdrawal(
        &mut self,
        tx: Transaction,
        amount: Amount,
        transactions: &mut Transactions,
    ) -> Result<(), TransactionProcessingError> {
        if amount > self.get_available() {
            // Reject processing if there isn't enough available
            return Err(TransactionProcessingError::InsufficientAvailableFunds(
                tx.tx_id, amount,
            ));
        }
        transactions.entry(tx.tx_id).or_insert_with(|| {
            // Or, since it's absent, add the withdrawal transaction to the record and update the account total amount 👀
            // Note: If already present in transactions, it will be ignored.
            self.total -= amount;
            tx
        });
        Ok(())
    }

    fn process_dispute(
        &mut self,
        tx: Transaction,
        transactions: &mut Transactions,
        disputes: &mut Disputes,
    ) -> Result<(), TransactionProcessingError> {
        // Ignore processing if there is a pending (unresolved) dispute already for this transaction.
        if let Some(d) = disputes.get(&tx.tx_id) {
            if d.tx_id == tx.tx_id {
                return Ok(());
            }
        }

        // Return an error if the given tx has a `ClientID` that is not the one of this account.
        if tx.client_id != self.client_id {
            return Err(TransactionProcessingError::InconsistentOperation);
        }

        // Process this dispute
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                // Return an error if the referred tx of the given tx has a `ClientID` that is not the one of this account.
                if t.client_id != self.client_id {
                    return Err(TransactionProcessingError::InconsistentOperation);
                }

                // Disputed, hence add it as pending and increase in the transaction's amount the value held 👀
                match t.tx_type {
                    TransactionType::Deposit(amount) => self.held += amount,

                    TransactionType::Withdrawal(amount) => self.held += amount,
                    _ => return Err(TransactionProcessingError::InconsistentOperation),
                }
                // Add this one to the pending (unresolved) disputes record.
                disputes.entry(tx.tx_id).or_insert(Dispute::from(tx));
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
        // Ignore processing this resolve if there is NOT a pending (unresolved) dispute for its referred transaction
        if disputes.get(&tx.tx_id).is_none() {
            return Ok(());
        }

        // Process this resolve
        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                // Return an error if the referred tx of the given tx has a `ClientID` that is not the one of this account.
                if t.client_id != self.client_id {
                    return Err(TransactionProcessingError::InconsistentOperation);
                }

                // Resolved, hence decrease in the referred transaction's amount the value held and... 👀
                match t.tx_type {
                    TransactionType::Deposit(amount) => self.held -= amount,
                    TransactionType::Withdrawal(amount) => self.held -= amount,
                    _ => return Err(TransactionProcessingError::InconsistentOperation),
                }
                // ...remove it from pending disputes 👀
                disputes.remove(&tx.tx_id);
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
        // Ignore processing this chargeback if there is NOT a pending (unresolved) dispute for its referred transaction
        if disputes.get(&tx.tx_id).is_none() {
            return Ok(());
        }

        match transactions.get(&tx.tx_id) {
            None => Err(TransactionProcessingError::NotFound(tx.tx_id)),
            Some(t) => {
                // Return an error if the referred tx of the given tx has a `ClientID` that is not the one of this account.
                if t.client_id != self.client_id {
                    return Err(TransactionProcessingError::InconsistentOperation);
                }

                // Chargeback a deposit, hence 👀
                // 1. Decrease or increase the total value in this account by the previously disputed transaction's value.
                // 2. Decrease held of that value.
                // 3. Freeze the account.
                // 4. Remove the dispute from the record of disputes that are pending.
                let amount;
                match t.tx_type {
                    TransactionType::Deposit(a) => {
                        // Reverting a deposit, hence decrease total
                        amount = a;
                        self.total -= amount;
                    }
                    TransactionType::Withdrawal(a) => {
                        // Reverting a withdrawal, hence increase total
                        amount = a;
                        self.total += amount;
                    }
                    _ => return Err(TransactionProcessingError::InconsistentOperation),
                }
                self.held -= amount;
                self.locked = true;
                disputes.remove(&tx.tx_id);
                Ok(())
            }
        }
    }

    // Render this account in its current state following the expected format
    // as per `Rust Test.pdf`
    pub fn render_as_output_line(&self) {
        let output_line = format!(
            "{}, {:#.4}, {:#.4}, {:#.4}, {}",
            self.client_id,
            self.get_available(),
            self.held,
            self.total,
            self.locked
        );
        println!("{}", output_line);
    }
}
