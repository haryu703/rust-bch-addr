# bch_addr
[![CircleCI](https://circleci.com/gh/haryu703/rust-bch-addr/tree/master.svg?style=svg)](https://circleci.com/gh/haryu703/rust-bch-addr/tree/master)
[![codecov](https://codecov.io/gh/haryu703/rust-bch-addr/branch/master/graph/badge.svg)](https://codecov.io/gh/haryu703/rust-bch-addr)

cash_addr format implementation inspired by [bchaddrjs](https://github.com/bitcoincashjs/bchaddrjs).

## Usage
```rust
use bch_addr::Converter;
let converter = Converter::new();
let cash_addr = converter.to_cash_addr("1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR").unwrap();
assert_eq!(cash_addr, "bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk");

let legacy_addr = converter.to_legacy_addr(&cash_addr).unwrap();
assert_eq!(legacy_addr, "1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR");
```
