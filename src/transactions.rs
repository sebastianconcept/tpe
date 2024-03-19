use serde::Deserialize;

// We're using the as real value the one with the most precision available
// and consider rounding as a presentation concern.
pub type Amount = f64;

// Let's say u64 are okay values for hypergrowth at this time :)
pub type OID = u64;

#[derive(Debug,Deserialize)]
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
