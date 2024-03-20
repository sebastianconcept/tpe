use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::{
    models::{account::Accounts, transaction::Transactions},
    payments_engine::PaymentsEngine,
};

fn one_deposit() -> String {
    "deposit, 1, 1, 1.0\n".to_string()
}

fn two_deposits() -> String {
    "deposit, 34, 1, 1.0\ndeposit, 34, 2, 1.1".to_string()
}

fn three_deposits() -> String {
    "deposit, 34, 1, 1.0\ndeposit, 22, 3, 1.0\ndeposit, 34, 2, 1.7".to_string()
}

fn two_deposits_one_repeats_tx_id() -> String {
    "deposit, 34, 1, 1.0\ndeposit, 22, 3, 1.0\ndeposit, 34, 2, 1.7".to_string()
}

fn two_deposits_one_repeats_tx_id_different_clients() -> String {
    "deposit, 34, 1, 1.0\ndeposit, 22, 3, 1.0\ndeposit, 34, 3, 1.7".to_string()
}

fn three_deposits_one_has_negative_amount() -> String {
    "deposit, 34, 1, 1.0\ndeposit, 34, 2, -42.42\ndeposit, 34, 3, 1.3".to_string()
}

#[test]
fn can_read_and_add_one_deposit_to_the_client_account() {
    let data = one_deposit();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1.0));
    assert_eq!(account.total, Decimal::from(1.0));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn two_deposits_to_the_client_account() {
    let data = two_deposits();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.1))
}

#[test]
fn three_deposits_but_two_to_the_same_client_account() {
    let data = three_deposits();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = accounts.get(&22).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1))
}

#[test]
fn three_deposits_but_two_repeated_tx_id_same_client_account() {
    let data = two_deposits_one_repeats_tx_id();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = accounts.get(&22).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1))
}

#[test]
fn three_deposits_but_two_repeated_tx_id_different_client_accounts() {
    // The first deposit should be considered valid and the second one ignored.
    let data = two_deposits_one_repeats_tx_id_different_clients();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = accounts.get(&22).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1))
}

#[test]
fn three_deposits_to_the_same_client_account_but_ignores_the_one_with_negative_amount() {
    let data = three_deposits_one_has_negative_amount();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let pe = PaymentsEngine::default();
    let mut accounts = Accounts::default();
    let mut transactions = Transactions::default();
    pe.process_transactions_using(reader, &mut accounts, &mut transactions)
        .unwrap();
    let account = accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.3));
}
