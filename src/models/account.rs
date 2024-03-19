use std::collections::HashMap;

use super::shared::{Amount, OID};

#[derive(Debug)]
pub struct Account {
    client_id: OID,
    amount: Amount,
}

pub type Accounts = HashMap<OID, Account>;
