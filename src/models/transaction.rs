use crate::models::shared::{Amount, OID};
use fraction::Decimal;
use serde::{Deserialize, Deserializer};

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
    #[serde(deserialize_with = "decimal_from_string")]
    amount: Amount,
}

fn decimal_from_string<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let float = match s.parse::<f64>() {
        Ok(val) => Ok(val),
        Err(err) => Err(err).map_err(serde::de::Error::custom),
    };
    Ok(Decimal::from(float?))
}