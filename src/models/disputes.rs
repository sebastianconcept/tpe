use std::collections::HashMap;

use super::{
    shared::{ClientID, TransactionID},
    transaction::Transaction,
};

// An index to reach what transactions are currently disputed by transaction ID
pub type Disputes = HashMap<TransactionID, Dispute>;

#[derive(Debug)]
pub struct Dispute {
    pub client_id: ClientID,
    pub tx_id: TransactionID,
}

impl Dispute {
    pub fn from(tx: Transaction) -> Self {
        Self {
            client_id: tx.client_id,
            tx_id: tx.tx_id,
        }
    }
}
