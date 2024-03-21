use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::{ input_ingestion::get_csv_reader, payments_engine::PaymentsEngine};

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

fn deposits_but_one_withdrawal_has_negative_amount() -> String {
    "deposit, 43, 1, 1.0\ndeposit, 43, 2, 2.42\nwithdrawal, 43,7,-2.2\ndeposit, 43, 3, 1.3"
        .to_string()
}

#[test]
fn can_read_and_add_one_deposit_to_the_client_account() {
    let data = one_deposit();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&1).unwrap();
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
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&34).unwrap();
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
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = pe.accounts.get(&22).unwrap();
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
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = pe.accounts.get(&22).unwrap();
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
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.7));
    let account = pe.accounts.get(&22).unwrap();
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
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&34).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2.3));
}

#[test]
fn ignores_widthdrawal_with_negative_amount() {
    let data = deposits_but_one_withdrawal_has_negative_amount();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&43).unwrap();
    assert_eq!(account.get_available(), Decimal::from(4.72));
}

fn deposits_but_two_disputes_repeat_on_same_tx_and_client_id() -> String {
    "deposit, 43, 1, 1.0\ndeposit, 43, 2, 2.42\ndispute, 43,1,\ndispute, 43, 1,".to_string()
}

#[test]
fn ignores_repeated_disputes_on_same_tx_from_same_client_id() {
    let data = deposits_but_two_disputes_repeat_on_same_tx_and_client_id();
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_using(reader).unwrap();
    let account = pe.accounts.get(&43).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1.42));
    assert_eq!(account.held, Decimal::from(1));
    assert_eq!(account.total, Decimal::from(3.42));
}

#[test]
fn case4() {
    let reader = get_csv_reader("input/case4-ignores-repeated-unresolved-dispute.csv".to_owned());
    // let reader = ReaderBuilder::new()
    //     .has_headers(false)
    //     .trim(Trim::All)
    //     .delimiter(b',')
    //     .from_reader(data.as_bytes());
    let mut pe = PaymentsEngine::default();
    // let mut accounts = Accounts::default();
    // let mut transactions = Transactions::default();
    // let mut disputes = Disputes::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1.0));
    assert_eq!(account.total, Decimal::from(1.0));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}
