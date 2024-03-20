use std::{error::Error, fs::File};

use csv::Reader;

use crate::models::{
    account::{Account, Accounts},
    shared::ClientID,
    transaction::{Transaction, Transactions},
};

#[derive(Default)]
pub struct PaymentsEngine {}

impl PaymentsEngine {
    pub fn get_assured_account_mut<'a>(
        &self,
        accounts_by_client_id: &'a mut Accounts,
        client_id: ClientID,
    ) -> &'a mut Account {
        accounts_by_client_id
            .entry(client_id)
            .or_insert(Account::new(client_id))
    }

    pub fn process_transactions_from(
        &self,
        mut reader: Reader<File>,
    ) -> Result<Accounts, Box<dyn Error>> {
        let mut accounts_by_client_id = Accounts::default();
        let mut transactions_by_id = Transactions::default();
        for tx in reader.deserialize::<Transaction>() {
            self.process(tx?, &mut accounts_by_client_id, &mut transactions_by_id)?
        }
        Ok(accounts_by_client_id)
    }

    pub fn process_transactions_using(
        &self,
        mut reader: Reader<&[u8]>,
        accounts_by_client_id: &mut Accounts,
        transactions_by_id: &mut Transactions,
    ) -> Result<(), Box<dyn Error>> {
        for tx in reader.deserialize::<Transaction>() {
            self.process(tx?, accounts_by_client_id, transactions_by_id)?
        }
        Ok(())
    }

    pub fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        transactions: &mut Transactions,
    ) -> Result<(), Box<dyn Error>> {
        let account = self.get_assured_account_mut(accounts, transaction.client_id);
        account.process(transaction, transactions)?;
        Ok(())
    }
}
