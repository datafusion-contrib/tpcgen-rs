//! This library contains both the TPCH and TPCDS command line clients.
//!
//! # Feature Flags
//!
//! This crate builds against [arrow-rs] and [parquet] 60 by default. Two
//! feature flags change what is built:
//! * `arrow_59` - build against [Arrow 59.x] and [Parquet 59.x] instead of
//!   [Arrow 60.x] and [Parquet 60.x]
//! * `indicatif-progress` - draw terminal progress bars with [indicatif]
//!   (default)
//!
//! The selected versions are re-exported as [`arrow`] and [`parquet`], so
//! downstream crates can name the matching types without guessing.
//!
//! [arrow-rs]: https://github.com/apache/arrow-rs
//! [parquet]: https://docs.rs/parquet
//! [indicatif]: https://docs.rs/indicatif
//! [Arrow 59.x]: https://docs.rs/arrow/59
//! [Arrow 60.x]: https://docs.rs/arrow/60
//! [Parquet 59.x]: https://docs.rs/parquet/59
//! [Parquet 60.x]: https://docs.rs/parquet/60

// Alias the selected arrow and parquet versions as `arrow` and `parquet`.
#[cfg(feature = "arrow_59")]
pub extern crate arrow_59 as arrow;
#[cfg(not(feature = "arrow_59"))]
pub extern crate arrow_60 as arrow;
#[cfg(feature = "arrow_59")]
pub extern crate parquet_59 as parquet;
#[cfg(not(feature = "arrow_59"))]
pub extern crate parquet_60 as parquet;

mod args;
pub mod generate;
mod logging;
mod parquet_output;
pub mod progress;
pub mod sink;
pub mod statistics;
mod temp_path;
pub mod tpcds_cli;
pub mod tpch_cli;
mod worker_queue;
