use std::{collections::HashMap, error, fmt};

use crate::models::shared::Amount;
use fraction::{Decimal, Zero};
use serde::{Deserialize, Deserializer};

use super::shared::{ClientID, TransactionID};

// An index to reach transactions by transaction ID
pub type Transactions = HashMap<TransactionID, Transaction>;

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
    pub client_id: ClientID,
    pub tx_id: TransactionID,
    #[serde(deserialize_with = "decimal_from_string")]
    pub amount: Option<Amount>,
}

// Helps SerDe to deserialize the expected float amounts found as string into a fraction::Decimal
fn decimal_from_string<'de, D>(deserializer: D) -> Result<Option<Amount>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        // There are entries that will not deserialize an amount value.
        // So far these are: Dispute, Resolve and Chargeback
        return Ok(None);
    }
    match s.parse::<f64>() {
        Ok(val) => {
            let decimal = Decimal::from(val);
            if decimal < Decimal::zero() {
                return Err(serde::de::Error::custom(
                    TransactionDeserializingError::NegativeAmount(val),
                ));
            }
            Ok(Some(decimal))
        }
        Err(_err) => Err(serde::de::Error::custom(
            TransactionDeserializingError::UnableToParseAsFloat(s),
        )),
    }
}

pub enum TransactionDeserializingError {
    UnableToParseAsFloat(String),
    NegativeAmount(f64),
}

impl fmt::Display for TransactionDeserializingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "amount must be positive")
    }
}

#[derive(Debug)]
pub enum TransactionProcessingError {
    InsufficientAvailableFunds((TransactionID, Amount)),
    TargetAccountLocked(TransactionID),
    NotFound(TransactionID),
    InconsistentOperation,
}
impl error::Error for TransactionProcessingError {}

impl fmt::Display for TransactionProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TransactionProcessingError::NotFound(tx_id) => {
                write!(f, "Unable to process {}, transaction not found. Assuming partner's data inconsistency.", tx_id)
            }
            TransactionProcessingError::TargetAccountLocked(tx_id) => {
                write!(f, "Unable to process {}, target account is locked", tx_id)
            }
            TransactionProcessingError::InsufficientAvailableFunds((tx_id, val)) => {
                write!(
                    f,
                    "Insufficient available funds to process {:.4} in transaction {}",
                    val, tx_id
                )
            }
            TransactionProcessingError::InconsistentOperation => {
                write!(
                    f,
                    "The targeted account doesn't match the account of the referred transaction"
                )
            }
        }
    }
}
