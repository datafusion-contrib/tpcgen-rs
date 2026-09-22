//! This library contains both the TPCH and TPCDS command line clients.

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
