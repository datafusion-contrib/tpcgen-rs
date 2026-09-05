//! TPCH data generation CLI and library API.

mod cli;
mod generator;

pub mod csv;
pub mod generate;
pub mod output_plan;
pub mod plan;
pub mod runner;
pub mod statistics;
pub mod tbl;

pub use crate::progress;
pub use cli::Cli;
pub use generator::{
    Compression, Encoding, GeneratorConfig, OutputFormat, Table, TpchGenerator,
    TpchGeneratorBuilder, WriterSink,
};
pub use plan::{GenerationPlan, DEFAULT_PARQUET_ROW_GROUP_BYTES};
