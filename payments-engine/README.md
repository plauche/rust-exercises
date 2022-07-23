# Payments Engine

A simple payments processing engine built in Rust. This engine handles the processing of CSVs containing deposit and withdrawal
transactions for various clients, as well as disputes, resolutions, and chargebacks relating to prior transactions.

The engine is run as follows::

    cd engine
    cargo run -- transactions.csv > output.csv

The resulting output will be a CSV containing the end-state account states of any clients included in the source CSV.

Source CSVs should conform to the following format (with header included):

    type, client, tx, amount
    deposit, 1, 1, 1.0

Valid transaction types are: _deposit_, _withdrawal_, _dispute_, _resolve_, and _chargeback_.