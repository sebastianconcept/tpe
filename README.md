# Toy Payments Engine

## On input digestion
For the deserialization part of digesting input, I've decided to use Serde and csv as suggested as they are well known robust and well maintained crates.

The `Transaction` struct you'll find I've derived `Deserialize` but also enforced deserialization correctness on the `type` field of the CSV input data so we only have valid structs for processing. I've achieve that using the `TransactionType` enum together with serde's feature `rename_all = "lowercase"` so the names of the variants are not only consistent with the ones in the data but also very maintainable for us in the future or when other engineers might need to use them.

Regarding to numerical values to be parsed, I've made Serde's `Deserializer` to use a custom function for that named `decimal_from_string` which reads the string parsing it as `f64` but returning an instantiated `Decimal` from the `fraction` crate. This I've decided to do so the value held has maximum precision leaving any further rendering need as a concern of a presentation layer that could, for example, decide later on how many digits to print without making the program loose any precision for its inner math.

## Understanding the Processing Sequence



## General design notes
1. Input doesn't have headers. Valid input is just data without using the first row for headers.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
3. Not considering previous historical state. When instantiating an account for a client, I assumed all quantities were at 0 and the account not locked. To improve this, it would be necessary to access the database where this history is stored and reimplement the function `get_account_for(client_id: OID)`.
4. Withdrawals cannot be processed on locked accounts.
5. Disputes will not happen in locked accounts and accounts with insufficient available value will fail to process them raising `TransactionProcessingError::InsufficientFunds`.
6. What happens if the processing engine receives input to dispute a value greater than what's available? or total?