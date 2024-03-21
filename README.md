# Toy Payments Engine

## Run

```bash
cargo run -- inputs/case2.csv
```

## Overview

Conceptually, `tpe` is a toy but some parts are taken seriously. For example, correctness of the processed input and numerical precision on values. Also making the engine able to process a stream of input so it could be useful as the foundation of something more valuable.

## Program architecture

The program begins in the `main` function, where it initializes a CSV reader and a custom Serde deserializer to prepare for ingesting valid input data. Next, it creates the `PaymentsEngine` and begins processing transactions using the `process_transactions_from` method. This method utilizes the CSV reader's `DeserializeRecordsIter`, which is configured to deserialize valid `Transaction` structs in a streamed fashion, iterating over them as they become available.

As the whole processing goes on, the accounts are maintained in a consistent state by the `PaymentsEngine` in a `HashMap` and creating entries only on demand.

At the end of the processing, an iteration to render these account entries is what produces the output format as expected.

## Input assupmtions

As per exercise specification, `ClientID` is `u16` and `TransactionID` is `u32` while the amount value is a `String` representing a real positive number with 4 digits.
Any negative amount in the records of the input will be considered as an inconsistency coming from the partner and if such case occurs, the deserializer on the field will return a `None` and the `Reader` will return a specific `Err` that is handled so the processing can continue efficiently.

The specs mention that transactions have globally unique IDs yet, as defensive mechanism, when two input records have for any reason the same `TransactionID`, only their first occurrence is taken as valid and computed.

Repeated chargebacks on a `TransactionID` cannot occur because any attempt will encounter a frozen account.

Repeated unresolved disputes will be ignored.

Is expected not to happen by merit of input consistency, but if for any reason a dispute or resolve or chargeback came related to a `ClientID` but the transaction they refer is pointing to another `ClientID` the system will face an `Err(TransactionProcessingError::InconsistentOperation)` and will proceed to ignore it protecting its integrity and continuous operation.

## Output assupmtions

The output is of a well known format for the systems involved and headers are not present.

Amounts are rendered as floats printed with 4 digits of precision.

Disputes can only be considered valid when the dispute has a `ClientID` value equal to the one in the disputed transaction.

## On input digestion

For the deserialization part of digesting input, I've decided to use Serde and csv as suggested as they are well known robust and well maintained crates.

For the `Transaction` struct you'll find I've made it derive `Deserialize` but also enforced deserialization correctness on the `type` field of the CSV input data so we only have valid structs for processing. The program achieves that using the `TransactionType` enum together with Serde's feature `rename_all = "lowercase"` so the names of the variants are not only consistent with the ones in the data but also comfortably maintainable.

Regarding to parsed numerical values, I've made Serde's `Deserializer` to use a custom function named `decimal_from_string`. It reads the string parsing it as `f64` but returning an instantiated `Decimal` from the `fraction` crate because it promises losless fractions and decimals. This is valuable when there are lots of transactions, which with time it will happen, and `account.held` and `account.total` values can preserve precision. Any further rendering of these values, I'm taking that as a concern of a presentation layer that could, for example, decide later on how many digits to print without making the program loose any precision for its inner math.

## Processing Sequence

Here is an example of processing a `Dispute`.

```
            ┌──────────────────┐     ┌─────────────┐    ┌────────────┐ ┌───────────────┐ ┌────────────────────────────────────┐
            │  PaymentsEngine  │     │   Reader    │    │  Account   │ │  Transaction  │ │            Transactions            │
            └─────────┬────────┘     └──────┬──────┘    └──────┬─────┘ └───────┬───────┘ │                                    │
                      │                     │                  │               │         │HashMap<TransactionID, Transaction> │
                                            │                  │               │         └──────────────────┬─────────────────┘
 process_transactions_from                  │                  │               │                            │
   ──────────────────▶                      │                  │               │                            │
                      │                     │                  │               │                            │
                      │  deserialize::<Transaction>()          │               │                            │
                      ├────────────────────▶│                  │               │                            │
                      │◀────────────────────┤                  │               │                            │
                      │                     │                  │               │                            │
                      │───┐                 │                  │               │                            │
                      │   │process(tx?                         │               │                            │
                      │   │    , &mut accounts_by_client_id    │               │                            │
                      │◀──│    , &mut transactions_by_id)?     │               │                            │
                      │                     │                  │               │                            │
                      │              process(tx, transactions) │               │                            │
                      │─────────────────────┼─────────────────▶│               │                            │
                      │                     │                  │───┐process_dispute(                        │
                      │                     │                  │   │    tx: Transaction,                    │
                      │                     │                  │   │    transactions: &mut Transactions)    │
                      │                     │                  │◀──│           │                            │
                      │                     │                  │               │   get(tx.tx_id)            │
                      │                     │                  ├───────────────┼───────────────────────────▶│
                      │                     │                  │               │                            │
                      │                     │                  │◀──────────────┼────────────────────────────┤
                      │                     │                  │               │                            │
                      │                     │                  │───┐           │                            │
                      │                     │                  │   │If sufficient,                          │
                      │                     │                  │   │increase value held                     │
                      │                     │                  │◀──│by tx.amount                            │
                      │◀────────────────────┼──────────────────│               │                            │
                      │                     │                  │               │                            │
                      │                     │                  │               │                            │
```

## General design notes

1. Input doesn't have headers. Valid input is just data without using the first row for headers.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
3. Not considering previous historical state. When instantiating an account for a client, I assumed all quantities were at 0 and the account not locked. To improve this, it would be necessary to access the database where this history is stored and reimplement the function `get_account_for(client_id: OID)`.
4. Withdrawals cannot be processed on locked accounts.
5. Disputes will not happen in locked accounts and accounts with insufficient available value will fail to process them raising `TransactionProcessingError::InsufficientFunds`.

## Questions

1. What happens if the processing engine receives input to dispute a value greater than what's available? or total?
2. What happens when a `Transaction` is disputed 2 times? or more than twice.
3. What happens when a `Transaction` is resolved 2 times? or more than twice.
4. What happens when a `Transaction` has 2 chargebacks? or more than two.
5. What happens when a `Transaction` is repeated (same `TransactionID`, same `ClientID` and same `Amount`)?
6. What happens when a `Transaction` is repeated (same `TransactionID`, same `ClientID` and _different_ `Amount`)?
7. What happens when the `TransactionID` in a dispute corresponds to a `ClientID` that is not the same? The spec states that the tx in a dispute could not exist and be safely ignored but it doesn't clarify anything about being about a different `ClientID`.

## Further contributions and recommendations

To make this processing engine more scalable, streaming the input via a networked service would be advisable. There are many options and protocols for that. If based on HTTP, [Axum](https://github.com/tokio-rs/axum) and [hyper](https://github.com/hyperium/hyper) are both based on the [Tokio](https://github.com/tokio-rs/tokio) runtime which is a great technical foundation for safe and efficient asynchronous and multithread code.

That brings the efficiency and ability to take advantage of using multi-core CPUs and some added complexity to make that safe.

For example, the structures holding the accounts and transactions, that in this program are `HashMap`s, would need be used behind a protection wall like `Mutex` or `RwLock` to be safe.

Alternatively, these could be reached via a separate networked service shared among client programs all of them growing in different hosts that operations can scale up or down following traffic demands.

If the requirements are high volume and one host cannot hold all the transactions in memory, then a design based in Deterministic Sharding for stable low-latency and efficient horizontal scaling should be explored.

## To do

- ~~Add first unit tests~~
- ~~Make specific error variant for deserializing a negative number~~
- ~~Make it render output~~
- ~~Clean println! entries used for debug~~
- ~~Add sequence diagram~~
- ~~Ignores records that have negative amount. Add unit test.~~
- ~~Ignore repeated deposits in the same `TransactionID`. Only the first one is considered valid. Add unit test.~~
- ~~Unless resolved, ignore repeated disputes on the same `TransactionID`. Add unit test.~~
- ~~Decide on what to do if a dispute has diverging `ClientID`. Only valid for same `ClientID` than the disputed transaction~~
- fraction::Decimal printing 4 decimals in the output.
