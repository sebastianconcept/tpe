use fraction::Decimal;

use crate::{input_ingestion::get_csv_reader, payments_engine::PaymentsEngine};

#[test]
fn case1() {
    // 3 deposits in 2 accounts then 2 withdrawals
    let reader = get_csv_reader("resources/case-inputs/case1.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1.5));
    assert_eq!(account.total, Decimal::from(1.5));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case2() {
    // Operations and one dispute.
    let reader = get_csv_reader("resources/case-inputs/case2.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1));
    assert_eq!(account.total, Decimal::from(5));
    assert_eq!(account.held, Decimal::from(4));
    assert!(!account.locked)
}

#[test]
fn case3() {
    // Operations and one dispute gets resolved.
    let reader = get_csv_reader("resources/case-inputs/case3.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(5));
    assert_eq!(account.total, Decimal::from(5));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case4() {
    // Operations and one dispute and then repeating the same dispute.
    // Repetitions of unresolved disputes getting ignored.
    let reader = get_csv_reader("resources/case-inputs/case4.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1));
    assert_eq!(account.total, Decimal::from(5));
    assert_eq!(account.held, Decimal::from(4));
    assert!(!account.locked);
}


#[test]
fn case5() {
    // Operations and one dispute with an invalid tx.
    // Ignores disputes pointing to a transaction that doesn't exist.
    let reader = get_csv_reader("resources/case-inputs/case5.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(9));
    assert_eq!(account.total, Decimal::from(9));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case6() {
    // Operations and one dispute that gets resolved and then more resolutions on that same tx.
    // Repeated resolutions of non pending disputes get ignored.
    let reader = get_csv_reader("resources/case-inputs/case6.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(8));
    assert_eq!(account.total, Decimal::from(8));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}