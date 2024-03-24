use std::collections::HashMap;

use super::shared::{ClientID, TransactionID};
use crate::{input_ingestion::InputAccessError, models::shared::Amount};
use fraction::{Decimal, Zero};
use serde::{Deserialize, Deserializer};
use thiserror::Error;

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
    match s.parse::<Decimal>() {
        Ok(v) => {
            if v < Decimal::zero() {
                return Err(serde::de::Error::custom(
                    TransactionDeserializingError::NegativeAmount(v.to_string()),
                ));
            }
            Ok(Some(v))
        }
        Err(_err) => Err(serde::de::Error::custom(
            TransactionDeserializingError::UnableToParseAmount(s),
        )),
    }
}

#[derive(Debug, Error)]
pub enum TransactionDeserializingError {
    #[error("Unable to parse `{0}` (amount must represent a float)")]
    UnableToParseAmount(String),
    #[error("Unable to parse `{0}` (amount must be a positive float)")]
    NegativeAmount(String),
}

#[derive(Debug, Error)]
pub enum TransactionProcessingError {
    #[error("Insufficient available funds to process `{1}` in transaction `{0}`")]
    InsufficientAvailableFunds(TransactionID, Amount),
    #[error("Unable to process {0}, target account is locked")]
    TargetAccountLocked(TransactionID),
    #[error(
        "Unable to process `{0}`, transaction not found. Assuming partner's data inconsistency."
    )]
    NotFound(TransactionID),
    #[error("The targeted account doesn't match the account of the referred transaction")]
    InconsistentOperation,
    #[error("Input deserialization error")]
    DeserializationError(#[from] TransactionDeserializingError),
    #[error("Input access error")]
    InputError(#[from] InputAccessError),
}
