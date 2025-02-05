# Xapo2ks

Translates from Xapo's account statement CSV files to [the generic import format from
Kryptosekken](https://www.kryptosekken.no/regnskap/importer-csv-generisk), for tax
reporting.

Limited transaction types and use cases are currently supported:

* Exchange from BTC to USD
* Income from interest payments
* Income from card cashback

Important assumptions are made:

* Exchange into USD is only done to fund short-term debit card spending. For tax purposes we consider such an exchange
  as consumption, and we do not report on individual card transactions. If you make longer-term trades from BTC into
  USD for the purpose of holding USD, this will not be appropriate for you.
* BTC withdrawals using on-chain or Lightning Network are assumed to be transfers into other wallets controlled by
  the taxpayer. If you use Xapo for payments or transfers to third-parties, you need to post-process the transactions,
  either by editing the resulting CSV, or editing the transactions in Kryptosekken after import.

All other features and usages are not supported, such as:

* Exchange from USD to BTC
* Deposits
* Withdrawals of USD
* Stocks and non-Bitcoin cryptocurrencies

Contributions are accepted!

## Usage

For the current feature-set, only the "BTC Account" and "BTC Savings" files are needed.

```
cargo run -- --btc-account-file=btc-account.csv --btc-savings-file=btc-savings.csv
```

CSV will be output on stdout.
