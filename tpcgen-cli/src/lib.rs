//! This library contains both the TPCH and TPCDS command line clients.
//!
//! # Feature Flags
//! This crate supports multiple versions of the Arrow/Parquet crates via
//! feature flags.
//! * `arrow_60` - Use [Arrow 60.x] and [Parquet 60.x] (default)
//! * `arrow_59` - Use [Arrow 59.x] and [Parquet 59.x]
//!
//! Feature flags are additive and `arrow_60` is a default feature, so selecting
//! `arrow_59` requires `default-features = false`. If both are enabled the
//! newer version wins.
//!
//! The selected versions are re-exported as [`arrow`] and
//! [`parquet`] for downstream crates to use.
//!
//! [Arrow 59.x]: https://crates.io/crates/arrow/59.0.0
//! [Parquet 59.x]: https://crates.io/crates/parquet/59.0.0
//! [Arrow 60.x]: https://crates.io/crates/arrow/60.0.0
//! [Parquet 60.x]: https://crates.io/crates/parquet/60.0.0

// alias the selected arrow/parquet versions as `arrow` and `parquet`
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
pub extern crate arrow_60 as arrow;
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate parquet_59 as parquet;
#[cfg(feature = "arrow_60")]
pub extern crate parquet_60 as parquet;
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("exactly one of the `arrow_59` or `arrow_60` features must be enabled");

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
