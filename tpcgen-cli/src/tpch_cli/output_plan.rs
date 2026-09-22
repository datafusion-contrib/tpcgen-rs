//! * [`OutputLocation`]: where to output the generated data
//! * [`OutputPlan`]: an output file that will be generated
//! * [`OutputPlanGenerator`]: plans the output files to be generated

pub use crate::output_location::OutputLocation;

use crate::tpch_cli::plan::GenerationPlan;
use crate::tpch_cli::{OutputFormat, Table};
use log::debug;
use parquet::basic::{Compression, Encoding};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::io;

/// Parquet writer settings applied when generating parquet output.
#[derive(Debug, Clone, PartialEq)]
pub struct ParquetWriterOptions {
    pub compression: Compression,
    pub column_encodings: Option<Vec<(String, Encoding)>>,
}

impl Default for ParquetWriterOptions {
    fn default() -> Self {
        Self {
            compression: Compression::SNAPPY,
            column_encodings: None,
        }
    }
}

/// Describes an output partition (file) that will be generated
#[derive(Debug, Clone, PartialEq)]
pub struct OutputPlan {
    /// The table
    table: Table,
    /// The scale factor
    scale_factor: f64,
    /// The output format (TODO don't depend back on something in main)
    output_format: OutputFormat,
    /// Parquet writer options (compression and column encodings)
    parquet: ParquetWriterOptions,
    /// Where to output
    output_location: OutputLocation,
    /// Plan for generating the table
    generation_plan: GenerationPlan,
    /// CSV delimiter character
    csv_delimiter: char,
}

impl OutputPlan {
    pub fn new(
        table: Table,
        scale_factor: f64,
        output_format: OutputFormat,
        parquet: ParquetWriterOptions,
        output_location: OutputLocation,
        generation_plan: GenerationPlan,
        csv_delimiter: char,
    ) -> Self {
        Self {
            table,
            scale_factor,
            output_format,
            parquet,
            output_location,
            generation_plan,
            csv_delimiter,
        }
    }

    /// Return the table this partition is for
    pub fn table(&self) -> Table {
        self.table
    }

    /// Return the scale factor for this partition
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Return the output format for this partition
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// return the output location
    pub fn output_location(&self) -> &OutputLocation {
        &self.output_location
    }

    /// Return the parquet compression level for this partition
    pub fn parquet_compression(&self) -> Compression {
        self.parquet.compression
    }

    pub fn parquet_column_encodings(&self) -> Option<&[(String, Encoding)]> {
        self.parquet.column_encodings.as_deref()
    }

    /// Return the number of chunks part(ition) count (the number of data chunks
    /// in the underlying generation plan)
    pub fn chunk_count(&self) -> usize {
        self.generation_plan.chunk_count()
    }

    /// return the generation plan for this partition
    pub fn generation_plan(&self) -> &GenerationPlan {
        &self.generation_plan
    }

    /// Return the CSV delimiter character for this partition
    pub fn csv_delimiter(&self) -> char {
        self.csv_delimiter
    }
}

impl Display for OutputPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "table {} (SF={}, {} chunks) to {}",
            self.table,
            self.scale_factor,
            self.chunk_count(),
            self.output_location
        )
    }
}

/// Plans the creation of output files
pub struct OutputPlanGenerator {
    format: OutputFormat,
    scale_factor: f64,
    parquet: ParquetWriterOptions,
    parquet_row_group_bytes: i64,
    /// Where output is written
    base_location: OutputLocation,
    csv_delimiter: char,
    /// The generated output plans
    output_plans: Vec<OutputPlan>,
    /// Output directories that have been created so far
    /// (used to avoid creating the same directory multiple times)
    created_directories: HashSet<OutputLocation>,
}

impl OutputPlanGenerator {
    pub fn new(
        format: OutputFormat,
        scale_factor: f64,
        parquet: ParquetWriterOptions,
        parquet_row_group_bytes: i64,
        base_location: OutputLocation,
        csv_delimiter: char,
    ) -> Self {
        Self {
            format,
            scale_factor,
            parquet,
            parquet_row_group_bytes,
            base_location,
            csv_delimiter,
            output_plans: Vec::new(),
            created_directories: HashSet::new(),
        }
    }

    /// Generate the output plans for the given table and partition options
    pub fn generate_plans(
        &mut self,
        table: Table,
        cli_part: Option<i32>,
        cli_part_count: Option<i32>,
    ) -> io::Result<()> {
        // If the user specified only a part count, automatically create all
        // partitions for the table
        if let (None, Some(part_count)) = (cli_part, cli_part_count) {
            if GenerationPlan::partitioned_table(table) {
                debug!("Generating all partitions for table {table} with part count {part_count}");
                for part in 1..=part_count {
                    self.generate_plan_inner(table, Some(part), Some(part_count))?;
                }
            } else {
                // there is only one partition for this table (e.g nation or region)
                debug!("Generating single partition for table {table}");
                self.generate_plan_inner(table, Some(1), Some(1))?;
            }
        } else {
            self.generate_plan_inner(table, cli_part, cli_part_count)?;
        }
        Ok(())
    }

    fn generate_plan_inner(
        &mut self,
        table: Table,
        cli_part: Option<i32>,
        cli_part_count: Option<i32>,
    ) -> io::Result<()> {
        let generation_plan = GenerationPlan::try_new(
            table,
            self.format,
            self.scale_factor,
            cli_part,
            cli_part_count,
            self.parquet_row_group_bytes,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let output_location = self.output_location(table, cli_part)?;

        let plan = OutputPlan::new(
            table,
            self.scale_factor,
            self.format,
            self.parquet.clone(),
            output_location,
            generation_plan,
            self.csv_delimiter,
        );

        self.output_plans.push(plan);
        Ok(())
    }

    /// Return the output location for the given table
    ///
    /// * if part of is None, the output location is `{output_dir}/{table}.{extension}`
    ///
    /// * if part is Some(part), then the output location
    ///   will be `{output_dir}/{table}/{table}table.{part}.{extension}`
    ///   (e.g. orders/orders.1.tbl, orders/orders.2.tbl, etc.)
    fn output_location(&mut self, table: Table, part: Option<i32>) -> io::Result<OutputLocation> {
        let extension = self.format.extension();

        let Some(part) = part else {
            // No partition specified, output to a single file
            return Ok(self.base_location.join(format!("{table}.{extension}")));
        };

        // If a partition is specified, create a subdirectory for it
        let dir = self.base_location.join(table.to_string());
        self.ensure_directory_exists(&dir)?;
        Ok(dir.join(format!("{table}.{part}.{extension}")))
    }

    /// Ensure the output directory exists, creating it if necessary
    fn ensure_directory_exists(&mut self, dir: &OutputLocation) -> io::Result<()> {
        if self.created_directories.contains(dir) {
            return Ok(());
        }
        dir.create_dir_all()?;
        self.created_directories.insert(dir.clone());
        Ok(())
    }

    /// Return the output plans generated so far
    pub fn build(self) -> Vec<OutputPlan> {
        self.output_plans
    }
}
