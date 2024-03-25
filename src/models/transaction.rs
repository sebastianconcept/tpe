use core::fmt;
use std::collections::HashMap;

use super::shared::{ClientID, TransactionID};
use crate::{input_ingestion::InputAccessError, models::shared::Amount};
use fraction::Zero;
use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

// An index to reach transactions by transaction ID
pub type Transactions = HashMap<TransactionID, Transaction>;

#[derive(Debug)]
pub enum TransactionType {
    Deposit(Amount),
    Withdrawal(Amount),
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub client_id: ClientID,
    pub tx_id: TransactionID,
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Transaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionVisitor;

        impl<'de> de::Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid transaction")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Transaction, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let tx_type = seq.next_element::<String>()?.ok_or_else(|| {
                    de::Error::custom(TransactionDeserializingError::UnableToParseTransactionType)
                })?;

                match tx_type.as_str() {
                    "deposit" => {
                        let client_id: ClientID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseClientID)
                        })?;

                        let tx_id: TransactionID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(
                                TransactionDeserializingError::UnableToParseTransactionID,
                            )
                        })?;

                        let amount_string: String = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseAmount(
                                "failed to read amount".to_string(),
                            ))
                        })?;

                        let amount = match amount_string.parse::<Amount>() {
                            Ok(v) => {
                                if v < Amount::zero() {
                                    return Err(serde::de::Error::custom(
                                        TransactionDeserializingError::NegativeAmount(
                                            v.to_string(),
                                        ),
                                    ));
                                }
                                v
                            }
                            Err(_err) => {
                                return Err(serde::de::Error::custom(
                                    TransactionDeserializingError::UnableToParseAmount(
                                        amount_string,
                                    ),
                                ))
                            }
                        };
                        Ok(Transaction {
                            tx_type: TransactionType::Deposit(amount),
                            client_id,
                            tx_id,
                        })
                    }
                    "withdrawal" => {
                        let client_id: ClientID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseClientID)
                        })?;

                        let tx_id: TransactionID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(
                                TransactionDeserializingError::UnableToParseTransactionID,
                            )
                        })?;

                        let amount_string: String = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseAmount(
                                "failed to read amount".to_string(),
                            ))
                        })?;

                        let amount = match amount_string.parse::<Amount>() {
                            Ok(v) => {
                                if v < Amount::zero() {
                                    return Err(serde::de::Error::custom(
                                        TransactionDeserializingError::NegativeAmount(
                                            v.to_string(),
                                        ),
                                    ));
                                }
                                v
                            }
                            Err(_err) => {
                                return Err(serde::de::Error::custom(
                                    TransactionDeserializingError::UnableToParseAmount(
                                        amount_string,
                                    ),
                                ))
                            }
                        };
                        Ok(Transaction {
                            tx_type: TransactionType::Withdrawal(amount),
                            client_id,
                            tx_id,
                        })
                    }
                    "dispute" => {
                        let client_id: ClientID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseClientID)
                        })?;

                        let tx_id: TransactionID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(
                                TransactionDeserializingError::UnableToParseTransactionID,
                            )
                        })?;
                        Ok(Transaction {
                            tx_type: TransactionType::Dispute,
                            client_id,
                            tx_id,
                        })
                    }
                    "resolve" => {
                        let client_id: ClientID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseClientID)
                        })?;

                        let tx_id: TransactionID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(
                                TransactionDeserializingError::UnableToParseTransactionID,
                            )
                        })?;
                        Ok(Transaction {
                            tx_type: TransactionType::Resolve,
                            client_id,
                            tx_id,
                        })
                    }
                    "chargeback" => {
                        let client_id: ClientID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(TransactionDeserializingError::UnableToParseClientID)
                        })?;

                        let tx_id: TransactionID = seq.next_element()?.ok_or_else(|| {
                            de::Error::custom(
                                TransactionDeserializingError::UnableToParseTransactionID,
                            )
                        })?;
                        Ok(Transaction {
                            tx_type: TransactionType::Chargeback,
                            client_id,
                            tx_id,
                        })
                    }
                    _ => Err(de::Error::custom(
                        TransactionDeserializingError::InvalidTransactionType,
                    )),
                }
            }
        }

        deserializer.deserialize_seq(TransactionVisitor)
    }
}

#[derive(Debug, Error)]
pub enum TransactionDeserializingError {
    #[error("Unable to parse the transaction type")]
    UnableToParseTransactionType,
    #[error("Unable to parse the client ID")]
    UnableToParseClientID,
    #[error("Unable to parse the transaction ID")]
    UnableToParseTransactionID,
    #[error("Unable to parse `{0}` transaction amount")]
    UnableToParseAmount(String),
    #[error("Unable to parse `{0}` (amount must be a positive float)")]
    NegativeAmount(String),
    #[error("Transaction type not supported")]
    InvalidTransactionType,
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
