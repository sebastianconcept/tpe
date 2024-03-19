use std::{error, fmt};

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
    pub tx_type: TransactionType,
    pub client_id: OID,
    pub tx_id: OID,
    #[serde(deserialize_with = "decimal_from_string")]
    pub amount: Amount,
}

// Helps SerDe to deserialize the expected float amounts found as string into a fraction::Decimal
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

#[derive(Debug)]
pub enum TransactionError {
    InsufficientFunds,
    LockedAccount
}
impl error::Error for TransactionError {}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TransactionError::LockedAccount => {
                write!(f, "This account is locked")
            }
            TransactionError::InsufficientFunds => {
                write!(f, "Insufficient funds to complete transaction")
            }
        }
    }
}