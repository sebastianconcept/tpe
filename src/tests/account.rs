use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::models::transaction::Transaction;

#[test]
fn ignore_repeated_deposit() {
    let data =
        "type, client, tx, amount\ndeposit, 328, 56, 1.3\ndeposit, 328, 56, 2.6\n".to_string();
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .delimiter(b',')
        .has_headers(true)
        .from_reader(data.as_bytes());
    let deposit = reader.deserialize::<Transaction>().next().unwrap().unwrap();

    assert_eq!(deposit.client_id, 328);
    assert_eq!(deposit.tx_id, 56);
    assert_ne!(deposit.amount.unwrap(), Decimal::from(2.6));
    assert_eq!(deposit.amount.unwrap(), Decimal::from(1.3));
}
