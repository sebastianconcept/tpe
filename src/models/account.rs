use std::{collections::HashMap, error::Error};

use crate::models::transaction::{TransactionError, TransactionType};

use super::{
    shared::{Amount, OID},
    transaction::Transaction,
};

pub type Accounts = HashMap<OID, Account>;
#[derive(Debug)]
pub struct Account {
    client_id: OID,
    total: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    pub fn new(client_id: OID) -> Self {
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

    pub fn process(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        match tx.tx_type {
            TransactionType::Deposit => self.process_deposit(tx)?,
            TransactionType::Withdrawal => self.process_withdrawal(tx)?,
            TransactionType::Dispute => self.process_dispute(tx)?,
            TransactionType::Resolve => self.process_resolve(tx)?,
            TransactionType::Chargeback => self.process_chargeback(tx)?,
        }
        Ok(())
    }

    fn process_deposit(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        println!("Deposit ID {} for account {}", tx.tx_id, self.client_id);
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }
        self.total += tx.amount;
        println!(
            "Deposit ID {} amount {} => account: {:?}",
            tx.tx_id, tx.amount, self
        );

        Ok(())
    }
    fn process_withdrawal(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        println!("Withdrawal ID {} for account {}", tx.tx_id, self.client_id);
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }
        if self.get_available() < tx.amount {
            return Err(TransactionError::InsufficientFunds);
        }

        self.total -= tx.amount;
        println!(
            "Withdrawal ID {} amount {} => account: {:?}",
            tx.tx_id, tx.amount, self
        );
        Ok(())
    }
    fn process_dispute(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        println!("Dispute ID {} for account {}", tx.tx_id, self.client_id);
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }
        if self.get_available() < tx.amount {
            return Err(TransactionError::InsufficientFunds);
        }
        self.held += tx.amount;
        Ok(())
    }
    fn process_resolve(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        println!("Resolve ID {} for account {}", tx.tx_id, self.client_id);
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }        
        self.held -= tx.amount;
        Ok(())
    }
    fn process_chargeback(&mut self, tx: Transaction) -> Result<(), TransactionError> {
        println!("Chargeback ID {} for account {}", tx.tx_id, self.client_id);
        if self.locked {
            return Err(TransactionError::LockedAccount);
        }
        self.locked = true;
        Ok(())
    }
}
