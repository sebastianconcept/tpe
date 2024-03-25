use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::models::transaction::{Transaction, TransactionType};

#[test]
fn ignore_repeated_deposit() {
    let data = "deposit, 328, 56, 1.3\ndeposit, 328, 56, 2.6\n".to_string();
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();

    assert_eq!(deposit.client_id, 328);
    assert_eq!(deposit.tx_id, 56);
    if let TransactionType::Deposit(amount) = deposit.tx_type {
        assert_ne!(amount, Decimal::from(2.6));
        assert_eq!(amount, Decimal::from(1.3));
    } else {
        panic!()
    }
}
