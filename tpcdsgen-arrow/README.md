# TPC-DS Data Generator in Arrow format

Generate TPC-DS data directly into [Apache Arrow] format using the [tpcdsgen] and [arrow] crate.

[Apache Arrow]: https://arrow.apache.org/
[tpcdsgen]: https://crates.io/crates/tpcdsgen
[arrow]: https://crates.io/crates/arrow

# Example usage:

See [docs.rs page](https://docs.rs/tpcdsgen-arrow/latest/tpcdsgen_arrow/)

# Testing:

This crate ensures correct results using two methods.

1. Basic functional tests are in Rust doc tests in the source code (`cargo test --locked -p tpcdsgen-arrow --doc`)
2. The `reparse` integration test ensures that the Arrow generators
   produce the same results as parsing the original DAT format (`cargo test --locked -p tpcdsgen-arrow --test reparse`)

# Contributing:

Please see [CONTRIBUTING.md] for more information on how to contribute to this project.

[CONTRIBUTING.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/CONTRIBUTING.md
