# Toy Payments Engine

## Design notes
1. Input doesn't have headers. Valid input is just data without using the first row for headers.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
3. Not considering previous historical state. When instantiating an account for a client, I assumed all quantities were at 0 and the account not locked. To improve this, it would be necessary to access the database where this history is stored and reimplement the function `get_account_for(client_id: OID)`.
4.