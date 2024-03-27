use csv::{ReaderBuilder, Trim};
use fraction::Decimal;

use crate::{input_ingestion::get_csv_reader, payments_engine::PaymentsEngine};

#[test]
fn can_process_input_as_stream_of_bytes() {
    // The PaymentEngine can work streaming data instead of using input from a CSV file.
    let data = "deposit, 1, 1, 1.0\ndeposit, 2, 2, 2.0";
    let reader = ReaderBuilder::new()
        .has_headers(false)
        .trim(Trim::All)
        .delimiter(b',')
        .from_reader(data.as_bytes());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_streaming_input(reader).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1));
    let account = pe.accounts.get(&2).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2))
}

#[test]
fn case1() {
    // 3 deposits in 2 accounts then 2 withdrawals
    let reader = get_csv_reader("resources/case-inputs/case1.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    let expected_available_and_total =
        Decimal::from("987654321987654.1001") + Decimal::from(2) + Decimal::from(0.0101)
            - Decimal::from(1.5);
    assert_eq!(account.get_available(), expected_available_and_total);
    assert_eq!(account.total, expected_available_and_total);
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
    // Repeated resolves of non pending disputes get ignored.
    let reader = get_csv_reader("resources/case-inputs/case6.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(8));
    assert_eq!(account.total, Decimal::from(8));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case7() {
    // Operations and one dispute that gets a chargeback.
    // The value held by the disputed transaction gets subtracted from the total and the account becomes frozen.
    // A that comes deposit after the account is locked gets ignored.
    let reader = get_csv_reader("resources/case-inputs/case7.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(4));
    assert_eq!(account.total, Decimal::from(4));
    assert_eq!(account.held, Decimal::from(0));
    assert!(account.locked);
}

#[test]
fn case8() {
    // Two deposits and one withdrawal followed by a dispute that repeats and then the first deposit repeats a lot and then the withdrawal repeats once.
    // The repeated transactions are ignored.
    let reader = get_csv_reader("resources/case-inputs/case8.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1));
    assert_eq!(account.total, Decimal::from(5));
    assert_eq!(account.held, Decimal::from(4));
    assert!(!account.locked);
}

#[test]
fn case9() {
    // Some deposits and a dispute to one of the accounts, but the dispute tx id doesn't belong to the same account.
    // The disputes aimed to an account that refer to transactions that happened in a different account are ignored.
    let reader = get_csv_reader("resources/case-inputs/case9.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();

    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(4));
    assert_eq!(account.total, Decimal::from(4));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);

    let account = pe.accounts.get(&2).unwrap();
    assert_eq!(account.get_available(), Decimal::from(3));
    assert_eq!(account.total, Decimal::from(3));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case10() {
    // Two deposits to account 1 and one withdrawal followed by a dispute to that withdrawal and a new deposit.
    // The dispute on withdrawal has its value held.
    let reader = get_csv_reader("resources/case-inputs/case10.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();

    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(2));
    assert_eq!(account.total, Decimal::from(3));
    assert_eq!(account.held, Decimal::from(1));
    assert!(!account.locked);
}

#[test]
fn case11() {
    // Deposit gets a dispute and then a chargeback.
    // The value held by the disputed transaction gets subtracted from the total and the account becomes frozen.
    // All deposits and withdrawals after the account is locked are ignored.
    let reader = get_csv_reader("resources/case-inputs/case11.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(1));
    assert_eq!(account.total, Decimal::from(1));
    assert_eq!(account.held, Decimal::from(0));
    assert!(account.locked);
}

#[test]
fn case12() {
    // Withdrawals gets a dispute and then a chargeback.
    // The value held by the disputed transaction gets subtracted from the total and the account becomes frozen.
    // All deposits and withdrawals after the account is locked are ignored.
    let reader = get_csv_reader("resources/case-inputs/case12.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(3));
    assert_eq!(account.total, Decimal::from(3));
    assert_eq!(account.held, Decimal::from(0));
    assert!(account.locked);
}

#[test]
fn case13() {
    // Operations in two accounts and account 1 experiences all the different types of input.
    let reader = get_csv_reader("resources/case-inputs/case13.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();
    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(3.5));
    assert_eq!(account.total, Decimal::from(3.5));
    assert_eq!(account.held, Decimal::from(0));
    assert!(account.locked);
}

#[test]
fn case14() {
    // Some deposits and a dispute to one of the accounts, then a chargeback with the same disputed tx id but set for a different account.
    // The chargebacks aimed to an account that refer to transactions that happened in a different account are ignored.
    let reader = get_csv_reader("resources/case-inputs/case14.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();

    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(account.get_available(), Decimal::from(5));
    assert_eq!(account.total, Decimal::from(6));
    assert_eq!(account.held, Decimal::from(1));
    assert!(!account.locked);

    let account = pe.accounts.get(&2).unwrap();
    assert_eq!(account.get_available(), Decimal::from(4));
    assert_eq!(account.total, Decimal::from(4));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}

#[test]
fn case15() {
    // One deposit and the tiniest withdrawal
    let reader = get_csv_reader("resources/case-inputs/case15.csv".to_owned());
    let mut pe = PaymentsEngine::default();
    pe.process_transactions_from(reader.unwrap()).unwrap();

    let account = pe.accounts.get(&1).unwrap();
    assert_eq!(
        account.get_available(),
        Decimal::from("0.9999999999999999999")
    );
    assert_eq!(account.total, Decimal::from("0.9999999999999999999"));
    assert_eq!(account.held, Decimal::from(0));
    assert!(!account.locked);
}
