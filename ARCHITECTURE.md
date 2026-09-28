# Architecture Guide

## Crate Organization
The workspace is organized into crates that separate the data generation, Arrow
conversion, and command-line tools:

1. `tpchgen`: The core library that implements TPC-H data generation.
2. `tpchgen-arrow`: Converts TPC-H generated rows directly into the
   [Apache Arrow] in-memory format.
3. `tpcdsgen`: The core library that implements TPC-DS data generation.
4. `tpcdsgen-arrow`: Converts TPC-DS generated rows directly into the
   [Apache Arrow] in-memory format.
5. `tpcgen-cli`: The shared command-line implementation crate for TPC benchmark
   data generation.
6. `tpchgen-cli`: A compatibility package that provides the backwards
   compatible `tpchgen-cli` binary.

## Dependencies

The `tpchgen` and `tpcdsgen` crates are designed to be embeddable in as many
locations as possible. The TPC-H core crate has no dependencies by design; for
example, it does not depend on Arrow, Parquet, or display libraries.

The `tpchgen-arrow` and `tpcdsgen-arrow` crates are similarly designed to be
embeddable with minimal dependencies. They depend on the
[`arrow` crate](https://docs.rs/arrow) and keep Arrow-specific conversion logic
out of the core generator crates.

The `tpcgen-cli` crate contains CLI-oriented dependencies and features, including
CSV/Parquet output, multi-threaded orchestration, progress reporting, logging,
and command parsing. The `tpchgen-cli` package is intentionally thin and
depends on `tpcgen-cli` so existing users can continue using the old binary name.

## Performance

Speed is a very important aspect of this project, and care has been taken to keep
the code as fast as possible, using some of the following techniques:
1. Avoiding heap allocations during data generation
2. Integer arithmetic and display instead of floating point arithmetic and display
3. Using multiple cores and tuned buffer sizes


[Apache Arrow]: https://arrow.apache.org/
