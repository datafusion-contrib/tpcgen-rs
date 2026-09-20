//! Rust TPCH Data Generator
//!
//! This crate provides a native Rust implementation of functions and utilities
//! necessary for generating the TPC-H benchmark dataset in several popular
//! formats.
//!
//! # Example: TBL output format
//! ```
//! # use tpchgen::generators::LineItemGenerator;
//! // Create Generator for the LINEITEM table at Scale Factor 1 (SF 1)
//! let scale_factor = 1.0;
//! let part = 1;
//! let num_parts = 1;
//! let generator = LineItemGenerator::new(scale_factor, part, num_parts);
//!
//! // Output the first 3 rows in classic TPCH TBL format
//! // (the generators are normal rust iterators and combine well with the Rust ecosystem)
//! let lines: Vec<_> = generator.iter()
//!    .take(3)
//!    .map(|line| line.to_string()) // use Display impl to get TBL format
//!    .collect::<Vec<_>>();
//!  assert_eq!(
//!   lines.join("\n"),"\
//!   1|155190|7706|1|17|21168.23|0.04|0.02|N|O|1996-03-13|1996-02-12|1996-03-22|DELIVER IN PERSON|TRUCK|egular courts above the|\n\
//!   1|67310|7311|2|36|45983.16|0.09|0.06|N|O|1996-04-12|1996-02-28|1996-04-20|TAKE BACK RETURN|MAIL|ly final dependencies: slyly bold |\n\
//!   1|63700|3701|3|8|13309.60|0.10|0.02|N|O|1996-01-29|1996-03-05|1996-01-31|TAKE BACK RETURN|REG AIR|riously. regular, express dep|"
//!   );
//! ```
//!
//! The TPC-H dataset is composed of several tables with foreign key relations
//! between them. For each table we implement and expose a generator that uses
//! the iterator API to produce structs e.g [`LineItem`] that represent a single
//! row.
//!
//! For each struct type we expose several facilities that allow fast conversion
//! to Tbl and Csv formats but can also be extended to support other output formats.
//!
//! This crate currently supports the following output formats:
//!
//! - TBL: The `Display` impl of the row structs produces the TPCH TBL format.
//! - CSV: the [`csv`] module has formatters for CSV output (e.g. [`LineItemCsv`]).
//!
//! [`LineItem`]: generators::LineItem
//! [`LineItemCsv`]: csv::LineItemCsv
//!
//! The library was designed to be easily integrated in existing Rust projects as
//! such it avoids exposing a malleable API and purposely does not have any dependencies
//! on other Rust crates. It is focused entirely on the core
//! generation logic.
//!
//! If you want an easy way to generate the TPC-H dataset for usage with external
//! see the [`tpchgen-cli`](https://github.com/datafusion-contrib/tpcgen-rs/tree/main/tpchgen-cli)
//! tool instead.
pub mod csv;
pub mod dates;
pub mod decimal;
pub mod distribution;
pub mod generators;
pub mod q_and_a;
pub mod random;
pub mod text;
