use std::{error::Error, fs::File};

use csv::Reader;

use crate::models::{
    account::{Account, Accounts},
    shared::OID,
    transaction::Transaction,
};

#[derive(Default)]
pub struct PaymentsEngine {}

impl PaymentsEngine {
    pub fn get_assured_account_mut<'a>(
        &self,
        accounts_by_client_id: &'a mut Accounts,
        client_id: OID,
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
        for tx in reader.deserialize::<Transaction>() {
            let transaction = tx?;
            self.process(transaction, &mut accounts_by_client_id)?;
        }
        Ok(accounts_by_client_id)
    }

    pub fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
    ) -> Result<(), Box<dyn Error>> {
        let account = self.get_assured_account_mut( accounts, transaction.client_id);
        account.process(transaction)?;
        Ok(())
    }
}
