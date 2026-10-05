//! Rust TPC-DS Data Generator
//!
//! This crate provides a native Rust implementation of the [TPC-DS]
//! dataset data generator in several popular formats.
//!
//! [TPC-DS]: http://www.tpc.org/tpcds/
//!
//! # Example: DAT output format
//! ```
//! # use tpcdsgen::config::{Session, Table};
//! # use tpcdsgen::row::ReasonRowGenerator;
//! // Create a Session at Scale Factor 1 (SF 1)
//! let session = Session::default();
//! // Row counts depend on the scale factor, so ask the session for them
//! let row_count = session.get_scaling().get_row_count(Table::Reason);
//! let generator = ReasonRowGenerator::new(session, row_count);
//!
//! // Output the first 3 rows in classic TPC-DS DAT format
//! // (the generators are normal rust iterators and combine well with the Rust ecosystem)
//! let lines: Vec<_> = generator
//!    .take(3)
//!    .map(|row| row.to_string()) // use Display impl to get DAT format
//!    .collect::<Vec<_>>();
//! assert_eq!(
//!   lines.join("\n"), "\
//!   1|AAAAAAAABAAAAAAA|Package was damaged|\n\
//!   2|AAAAAAAACAAAAAAA|Stopped working|\n\
//!   3|AAAAAAAADAAAAAAA|Did not get it on time|"
//! );
//! ```
//!
//! The TPC-DS dataset is composed of 24 tables (plus the `dbgen_version`
//! metadata table) with foreign key relations between them. Each table has a
//! generator in the [`row`] module that uses the iterator API to produce
//! structs e.g [`StoreSalesRow`] that represent a single row.
//!
//! For each struct type we expose several facilities that allow fast conversion
//! to various output formats such as:
//! - DAT: The `Display` impl of the row structs produces the TPC-DS `dsdgen` DAT
//!   format. [`DatWriter`] handles the character encoding (see below).
//! - CSV: the [`csv`] module has formatters for CSV output (e.g. [`StoreSalesCsv`]).
//!
//! See the [tpcdsgen-arrow] crate for direct generation of Arrow [`RecordBatch`].
//!
//! [`StoreSalesRow`]: row::StoreSalesRow
//! [`StoreSalesCsv`]: csv::StoreSalesCsv
//! [`DatWriter`]: output::DatWriter
//! [tpcdsgen-arrow]: https://docs.rs/tpcdsgen-arrow/latest/tpcdsgen_arrow/
//! [RecordBatch]: https://docs.rs/arrow/latest/arrow/array/struct.RecordBatch.html
//!
//! The library was designed to be easily integrated in existing Rust projects
//! and thus has no dependencies on other Rust crates. It is focused entirely on
//! the core generation logic.
//!
//! If you want an easy way to generate the TPC-DS dataset for use with external
//! tools, see the [`tpcgen-cli`] command line tool.
//!
//! [`tpcgen-cli`]: https://github.com/datafusion-contrib/tpcgen-rs/tree/main/tpcgen-cli
//! [`tpcdsgen-arrow`]: https://docs.rs/tpcdsgen-arrow/latest/tpcdsgen_arrow/
//!
//! # Configuration: [`Session`]
//!
//! A [`Session`] holds the scale factor, compatibility mode and chunking
//! options for a generation run and is built with [`SessionBuilder`]:
//!
//! ```
//! # use tpcdsgen::config::{CompatMode, SessionBuilder, Table};
//! # use tpcdsgen::row::StoreSalesRowGenerator;
//! let session = SessionBuilder::new()
//!     .with_scale_factor(10.0)
//!     .with_compat_mode(CompatMode::C)
//!     .with_chunk_number(2)   // generate the second of
//!     .with_total_chunks(4)   // four parts
//!     .build()
//!     .unwrap();
//!
//! // Each chunk generates a contiguous range of a table's source rows
//! let rows = session.get_source_row_range(Table::StoreSales);
//! let mut generator = StoreSalesRowGenerator::new(session, *rows.end());
//! generator.skip_rows_until_starting_row_number(*rows.start());
//! ```
//!
//! [`Session`]: config::Session
//! [`SessionBuilder`]: config::SessionBuilder
//!
//! # Reference implementations and [`CompatMode`]
//!
//! TPC-DS has two common reference implementations: the original C `dsdgen` and
//! the Java port used by Trino, which this crate was originally derived from.
//! They differ in a small number of places, so [`CompatMode`] selects which one
//! to match byte-for-byte:
//!
//! - [`CompatMode::Trino`] (default): matches the Java port, including its
//!   ISO-8859-1 (Latin-1) DAT output.
//! - [`CompatMode::C`]: matches the C `dsdgen`, correcting the known
//!   divergences of the Java port and writing UTF-8.
//!
//! The output is verified against both reference implementations in CI, see
//! [TESTING.md] for details.
//!
//! [`CompatMode`]: config::CompatMode
//! [`CompatMode::Trino`]: config::CompatMode::Trino
//! [`CompatMode::C`]: config::CompatMode::C
//! [TESTING.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/TESTING.md
//!
//! # Known Bugs
//!
//! The TPC-DS reference implementations contain several bugs that must be
//! replicated for benchmark compliance: fixing them would produce different
//! data and invalidate results that rely on this library. These bugs
//! originated in the C implementation and were reproduced in the Java port,
//! so this crate replicates them as well. See [BUGS.md] for the list.
//!
//! [BUGS.md]: https://github.com/datafusion-contrib/tpcgen-rs/blob/main/tpcdsgen/BUGS.md
//!
//! # Scale factors above 100,000
//!
//! The TPC-DS spec only defines scale factors up to 100,000; scaling beyond
//! that is not defined. Larger scale factors are allowed by this crate, but we
//! had to modify the generator logic above 100,000 to accommodate growth.

pub mod business_key_generator;
pub mod column;
pub mod config;
pub mod csv;
pub mod distribution;
pub mod error;
pub mod generator;
pub mod join_key_utils;
pub mod nulls;
pub mod output;
pub mod permutations;
pub mod pseudo_table_scaling_infos;
pub mod random;
pub mod row;
pub mod scaling_info;
pub mod slowly_changing_dimension_utils;
pub mod table;
pub mod table_flags;
pub mod types;

pub use error::TpcdsError;
