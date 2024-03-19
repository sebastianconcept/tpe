# Toy Payments Engine

## Design notes
1. Input doesn't have headers. Valid input is just data without using the first row for headers.
2. Valid fields are `type, client, tx, amount` in that order as per specs.
