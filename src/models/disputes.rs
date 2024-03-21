use std::collections::HashMap;

use super::shared::{ClientID, TransactionID};

// An index to reach what transactions are currently disputed by transaction ID
pub type Disputes = HashMap<TransactionID, Dispute>;

#[derive(Debug)]
pub struct Dispute {
    pub client_id: ClientID,
    pub tx_id: TransactionID,
}
