# TPC-H Data Generator in Arrow format

Generate TPCH data directly into [Apache Arrow] format using the [tpchgen] and [arrow] crate.

[Apache Arrow]: https://arrow.apache.org/
[tpchgen]: https://crates.io/crates/tpchgen
[arrow]: https://crates.io/crates/arrow

Supports arrow 60 (default) and arrow 59:

```toml
# arrow 60
tpchgen-arrow = "..."
# arrow 59
tpchgen-arrow = { version = "...", default-features = false, features = ["arrow_59"] }
```

# Example usage:

See [docs.rs page](https://docs.rs/tpchgen-arrow/latest/tpchgen_arrow/)

# Testing:
This crate ensures correct results using two methods.

1. Basic functional tests are in Rust doc tests in the source code (`cargo test --locked --doc`)
2. The `reparse` integration test ensures that the Arrow generators
   produce the same results as parsing the original `tbl` format (`cargo test --locked --test reparse`)

# Contributing:

Please see [CONTRIBUTING.md] for more information on how to contribute to this project.

[CONTRIBUTING.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/CONTRIBUTING.md
