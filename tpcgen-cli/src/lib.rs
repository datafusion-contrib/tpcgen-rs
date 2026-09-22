//! This library contains both the TPCH and TPCDS command line clients.
//!
//! # Feature Flags
//!
//! This crate supports multiple versions of [arrow-rs] via feature flags.
//! Exactly one must be enabled:
//! * `arrow_60` - build against [Arrow 60.x] and [Parquet 60.x] (default)
//! * `arrow_59` - build against [Arrow 59.x] and [Parquet 59.x] (requires
//!   `default-features = false`)
//!
//! Enabling both at once is a compile error. The selected versions are
//! re-exported as [`arrow`] and [`parquet`], so downstream crates can name the
//! matching types without guessing.
//!
//! The remaining flag is independent of the arrow version:
//! * `indicatif-progress` - draw terminal progress bars with [indicatif]
//!   (default)
//!
//! [arrow-rs]: https://github.com/apache/arrow-rs
//! [indicatif]: https://docs.rs/indicatif
//! [Arrow 59.x]: https://docs.rs/arrow/59
//! [Arrow 60.x]: https://docs.rs/arrow/60
//! [Parquet 59.x]: https://docs.rs/parquet/59
//! [Parquet 60.x]: https://docs.rs/parquet/60

// Exactly one arrow version must be selected; see the feature flag docs above.
#[cfg(all(feature = "arrow_59", feature = "arrow_60"))]
compile_error!("the `arrow_59` and `arrow_60` features are mutually exclusive");
#[cfg(not(any(feature = "arrow_59", feature = "arrow_60")))]
compile_error!("one of the `arrow_59` or `arrow_60` features must be enabled");

// Alias the selected arrow and parquet versions as `arrow` and `parquet`. The
// `arrow_59` arms also test `arrow_60` so that enabling both reports only the
// error above, and not also duplicate definitions of `arrow` and `parquet`.
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
pub extern crate arrow_60 as arrow;
#[cfg(all(feature = "arrow_59", not(feature = "arrow_60")))]
pub extern crate parquet_59 as parquet;
#[cfg(feature = "arrow_60")]
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
