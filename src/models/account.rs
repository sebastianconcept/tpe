use std::{collections::HashMap, error::Error};

use super::{
    shared::{Amount, OID},
    transaction::Transaction,
};

pub type Accounts = HashMap<OID, Account>;
#[derive(Debug)]
pub struct Account {
    client_id: OID,
    amount: Amount,
}

impl Account {
    pub fn new(client_id: OID) -> Self {
        Self {
            client_id,
            amount: Amount::from(0),
        }
    }

    pub fn process(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
        println!(
            "Account {} needs to process transaction {}",
            self.client_id, tx.client_id
        );
        Ok(())
    }
}
