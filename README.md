# Toy Payments Engine

## Run

```bash
cargo run -- resources/case-inputs/case1.csv
```

## Overview

Conceptually, `tpe` is a toy but some parts are taken seriously. For example, correctness of the processed input and numerical precision on values. Also, making the engine able to process a stream of input preserving integrity even when the input might include inconsistencies from partners that can be overcome. 

## Program architecture

The program begins in the `main` function, where it initializes a CSV reader and a custom Serde deserializer to prepare for ingesting valid input data. Next, it creates the `PaymentsEngine` and begins processing transactions using the `process_transactions_from` method. This method utilizes the CSV reader's `DeserializeRecordsIter`, which is configured to deserialize valid `Transaction` structs in a **streamed fashion**, iterating over them as they become available. This should be convenient as part of an effort to use this payment engine functionality as a continuous service.

As the whole processing goes on, the accounts are maintained in a consistent state by the `PaymentsEngine` in a `HashMap` and creating entries only on demand.

At the end of the processing, an iteration to render these account entries is what produces the output format as expected.

## Input assumptions

**Headers are expected in the input as the first row**.

As per exercise specification, `ClientID` is `u16` and `TransactionID` is `u32` while the amount value is a `String` representing a real positive number with 4 digits.
Any negative amount in the records of the input will be considered as an inconsistency coming from the partner and if such case occurs, the deserializer on the field will return a `None` and the `Reader` will return a specific `Err` that is handled so the processing can continue efficiently.

The specs mention that transactions have globally unique IDs yet, as defensive mechanism, when two input records have for any reason the same `TransactionID`, only their first occurrence is taken as valid and computed.

Repeated chargebacks on a `TransactionID` cannot occur because any attempt will encounter a frozen account.

Any transaction or operation on a frozen account will be ignored.

Repeated unresolved disputes will be ignored.

The specs mention a precision of 4 digits past the decimal but if for any reason a more precise value comes it will be parsed. The tiniest amount accepted for parsing  is `0.0000000000000000001`.

Is expected not to happen by merit of input consistency, but if for any reason a dispute or resolve or chargeback came related to a `ClientID` but the transaction they refer is pointing to another `ClientID` the system will face an `Err(TransactionProcessingError::InconsistentOperation)` and will proceed to ignore it protecting its integrity and continuous operation.

## Output assumptions


**Headers are emitted in the output as the first row.**

Comma separated values.

Amounts are rendered as floats printed with 4 digits of precision.

## On input digestion

For the deserialization part of digesting input, I've decided to use [Serde](https://github.com/serde-rs/serde) and [csv](https://github.com/BurntSushi/rust-csv) as suggested as they are well known robust and well maintained crates.

For the `Transaction` struct you'll find I've made it derive `Deserialize` but also enforced deserialization correctness on the `type` field of the CSV input data so we only have valid structs for processing. The program achieves that using the `TransactionType` enum together with Serde's feature `rename_all = "lowercase"` and `#[serde(rename = "type")]` so the names of the variants are not only consistent with the ones in the data but also comfortably maintainable in the code.

Regarding to parsed numerical values, I've made Serde's `Deserializer` to use a custom function named `decimal_from_string`. It reads the string parsing it as `fraction::Decimal` which has its own `Deserialize` implementation from the `fraction` crate (which is very precise). I've chosen the `fraction` crate because it promises lossless fractions and decimals for its operations. This is valuable when there are lots of transactions, which with time it will happen, and `account.held` and `account.total` values can preserve precision which is specially valuable for values in coins that deal with either monumental or extremely small numerical values. Any further rendering of these values, I'm taking that as a concern of the presentation layer that could, for example, decide later on how many digits to print without making the program loose any precision for its inner math. In this program, `account.render_as_output_line()` is dealing with that.

## Processing Sequence

Here is an example of the main parts and sequence involved in processing a `Dispute`.

```
          ┌──────────────────┐     ┌─────────────┐    ┌────────────┐         ┌────────────────────────────────────┐
          │  PaymentsEngine  │     │   Reader    │    │  Account   │         │            Transactions            │
          └─────────┬────────┘     └──────┬──────┘    └──────┬─────┘         │                                    │
                    │                     │                  │               │HashMap<TransactionID, Transaction> │
                                          │                  │               └──────────────────┬─────────────────┘
process_transactions_from(reader)         │                  │                                  │                  
 ──────────────────▶                      │                  │                                  │                  
                    │                     │                  │                                  │                  
                    │  deserialize::<Transaction>()          │                                  │                  
                    ├────────────────────▶│                  │                                  │                  
                    │                     │                  │                                  │                  
                    │◀────────────────────┤                  │                                  │                  
                    │                     │                  │                                  │                  
                    │───┐process(tx?                         │                                  │                  
                    │   │    , &mut accounts_by_client_id    │                                  │                  
                    │◀──┘    , &mut transactions_by_id       │                                  │                  
                    │        , &mut disputes_by_tx_id)?      │                                  │                  
                    │                     │                  │                                  │                  
                    │    process(tx, transactions, disputes) │ process_dispute(                                    
                    │─────────────────────┼─────────────────▶│     tx: Transaction,                                
                    │                     │                  │───┐ transactions: &mut Transactions,                
                    │                     │                  │   │ disputes: &mut Disputes)                        
                    │                     │                  │◀──┘                              │                  
                    │                     │                  │                   get(tx.tx_id)                     
                    │                     │                  ├─────────────────────────────────▶│                  
                    │                     │                  │                                  │                  
                    │                     │                  │◀─────────────────────────────────│                  
                    │                     │                  │                                  │                  
                    │                     │                  │                                  │                  
                    │                     │                  │───┐If not already disputed,      │                  
                    │                     │                  │   │increase value held by        │                  
                    │                     │                  │◀──┘tx.amount                     │                  
                    │                     │                  │                                  │                  
                    │◀────────────────────┼──────────────────│                                  │                  
                    │                     │                  │                                  │                  
                    ┆   process next...   │                  │                                  │                  
                     ───┐                 │                  │                                  │                  
                        │                 │                  │                                  │                  
                     ◀──┘                 │                  │                                  │                  
                    ┆                     │                  │                                  │                  
```

## General design notes

1. Input have headers. Valid input is a first row of headers followed by data about the supported operations in rows.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
3. Not considering previous historical state for the accounts. When instantiating an account for a client, I'm assuming all quantities are at 0 and the account is not locked. To improve this, it would be necessary to access the database where this history is stored and reimplement the function `get_account_for(client_id: OID)` accordingly.
4. Nor withdrawals nor deposits can be processed for locked accounts.
5. No operation or transaction will be processed for locked accounts.
6. Accounts with insufficient available funds will fail to process raising a `TransactionProcessingError::InsufficientFunds`.

## Questions

1. What happens if the processing engine receives input to dispute a value greater than what's available? or total? Can that happen?
- R: In the current design, a dispute will be processed when the account has sufficient available funds. When it doesn't, the system produces an `Err(TransactionProcessingError::InsufficientFunds)`.
2. What happens when a `Transaction` is disputed 2 times? or more than twice.
- R: Depends, if the dispute is pending, then the second dispute gets ignored. If there are disputes and resolutions in sequence these will be processed normally. If a pending dispute gets a chargeback, the account will stay locked and the engine will ignore input that aims at it.
3. What happens when a `Transaction` dispute is resolved 2 times? or more than twice.
- R: In dispute/resolve sequence, disputes on transactions can be resolved any amount of times.
4. What happens when a disputed `Transaction` has 2 chargebacks? or more than two.
- R: It can't. The first chageback will lock the account and all input aiming at it will be ignored.
5. What happens when a `Transaction` is repeated (same `TransactionID`, same `ClientID` and same `Amount`)?
- R: Deposits and withdrawals are using `transactions.entry(tx.tx_id).or_insert_with(` so they will ignore any `tx_id` that already got processed.
6. What happens when a `Transaction` is repeated (same `TransactionID`, same `ClientID` and _different_ `Amount`)?
- R: Same answer than 5. 
7. What happens when the `TransactionID` in a dispute corresponds to a `ClientID` that is not the same? 
- R: The spec states that the tx in a dispute could not exist and be safely ignored but it doesn't clarify anything about being about a different `ClientID`. In this `PaymentEngine` that is considered invalid input and these cases will be treated as input inconsistencies potentially coming from a partner's inconcistency hence, these transactions will be ignored (in a real system it should be observed using a pub/sub queue or logged for tracking, diagnosing and generally enabling its resolution).

## Further contributions and recommendations

To make this processing engine more scalable, streaming the input via a networked service would be advisable. There are many options and protocols for that. If based on HTTP, [Axum](https://github.com/tokio-rs/axum) and [hyper](https://github.com/hyperium/hyper) are both based on the [Tokio](https://github.com/tokio-rs/tokio) runtime which is a great technical foundation for safe and efficient asynchronous and multithread code. Interestingly, Tokio can be tunned with different strategies on how it gets load and work distributed among cores. Measurements in the lab under different load scenarios and strategies would trigger many very interesting discussions among the system engineers scaling this.

While that brings the efficiency and ability to take advantage of using multi-core CPUs it also carries some added complexity to make that safe.

For example, the structures holding the accounts and transactions, that in this program are `HashMap`s, would need to be used behind a protection wall like `Mutex` or `RwLock` to be safe.

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
- ~~fraction::Decimal printing 4 decimals in the output.~~
- ~~Add `TransactionProcessingError::InconsistentOperation`. With test case.~~
- ~~Parsing amount input using `fraction::Decimal::from(input: String)`. Add case.~~
- ~~Match `TransactionProcessingError` without downcasting boxed dyn errors.~~
- Explore adopting malachite for improved precision.