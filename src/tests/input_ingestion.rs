use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::models::transaction::{Transaction, TransactionType};

#[test]
fn can_parse_one_deposit() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, 1.0\n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(deposit.tx_type, TransactionType::Deposit));
    assert_eq!(deposit.client_id, 1);
    assert_eq!(deposit.tx_id, 1);
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.0));
}

#[test]
fn can_parse_one_deposit_and_one_withdrawal() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, 1.0\nwithdrawal, 1, 4, 1.5\n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();

    assert_eq!(deposit.client_id, 1);
    assert_eq!(deposit.tx_id, 1);
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.0));

    let withdrawal = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(withdrawal.tx_type, TransactionType::Withdrawal));
    assert_eq!(withdrawal.client_id, 1);
    assert_eq!(withdrawal.tx_id, 4);
    assert_eq!(withdrawal.amount.unwrap(), Decimal::from(1.5));
}

#[test]
fn can_parse_one_deposit_with_negative_ammount() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, -1.0\n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    assert!(reader.deserialize::<Transaction>().next().unwrap().is_err())
}

#[test]
fn can_parse_one_deposit_and_one_dispute_on_it() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, 1.0\ndispute, 1, 1, \n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(deposit.tx_type, TransactionType::Deposit));
    assert_eq!(deposit.client_id, 1);
    assert_eq!(deposit.tx_id, 1);
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.0));

    let dispute = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(dispute.tx_type, TransactionType::Dispute));
    assert_eq!(dispute.client_id, 1);
    assert_eq!(dispute.tx_id, 1);
    assert_eq!(dispute.amount, None);
}

#[test]
fn can_parse_one_deposit_and_one_dispute_and_one_resolve_on_it() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, 1.0\ndispute, 1, 1, \nresolve, 1, 1, \n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());

    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(deposit.tx_type, TransactionType::Deposit));
    assert_eq!(deposit.client_id, 1);
    assert_eq!(deposit.tx_id, 1);
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.0));

    let dispute = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(dispute.tx_type, TransactionType::Dispute));
    assert_eq!(dispute.client_id, 1);
    assert_eq!(dispute.tx_id, 1);
    assert_eq!(dispute.amount, None);

    let resolve = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(resolve.tx_type, TransactionType::Resolve));
    assert_eq!(resolve.client_id, 1);
    assert_eq!(resolve.tx_id, 1);
    assert_eq!(resolve.amount, None);
}

#[test]
fn can_parse_one_deposit_and_one_dispute_then_a_chargeback() {
    let data = "type, client, tx, amount\ndeposit, 1, 1, 1.0\ndispute, 1, 1, \nchargeback, 1, 1, \n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(deposit.tx_type, TransactionType::Deposit));
    assert_eq!(deposit.client_id, 1);
    assert_eq!(deposit.tx_id, 1);
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.0));

    let dispute = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(dispute.tx_type, TransactionType::Dispute));
    assert_eq!(dispute.client_id, 1);
    assert_eq!(dispute.tx_id, 1);
    assert_eq!(dispute.amount, None);

    let chargeback = reader.deserialize::<Transaction>().next().unwrap().unwrap();
    assert!(matches!(chargeback.tx_type, TransactionType::Chargeback));
    assert_eq!(chargeback.client_id, 1);
    assert_eq!(chargeback.tx_id, 1);
    assert_eq!(chargeback.amount, None);
}
