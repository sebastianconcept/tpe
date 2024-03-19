use std::collections::HashMap;

use super::shared::{Amount, OID};

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
}
