# Toy Payments Engine

## Design notes
1. Input doesn't have headers. Valid input is just data without using the first row for headers.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
3. Not considering previous historical state. When instantiating an account for a client, I assumed all quantities were at 0 and the account not locked. To improve this, it would be necessary to access the database where this history is stored and reimplement the function `get_account_for(client_id: OID)`.
4. Withdrawals cannot be processed on locked accounts.
5. Disputes will not happen in locked accounts and accounts with insufficient available value will fail to process them raising `TransactionProcessingError::InsufficientFunds`.
6. What happens if the processing engine receives input to dispute a value greater than what's available? or total?