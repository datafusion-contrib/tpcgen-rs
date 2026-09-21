//! This library contains both the TPCH and TPCDS command line clients.

// The arrow / parquet versions are selected by the `arrow_59` / `arrow_60`
// feature flags. The dependencies are renamed so both versions can be declared
// at once; alias the selected pair back to `arrow` and `parquet` so the rest of
// the crate can use them under their normal names. If both features are enabled
// (`--all-features`, or feature unification with another crate) the newer
// version wins.
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
