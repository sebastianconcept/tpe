use crate::models::shared::{Amount, OID};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    tx_type: TransactionType,
    client_id: OID,
    tx_id: OID,
    amount: Amount,
}
