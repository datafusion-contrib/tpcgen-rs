//! TPC-DS data generation CLI with a dbgen compatible API.
use crate::logging::configure_logging;
#[cfg(feature = "indicatif-progress")]
use crate::progress::IndicatifProgress;
use crate::progress::{no_op_progress_tracker, ProgressTracker};
use crate::tpch_cli::{Compression, DEFAULT_PARQUET_ROW_GROUP_BYTES};
use clap::builder::TypedValueParser;
use clap::{ArgAction, Args, Subcommand};
use std::collections::HashSet;
use std::io;
#[cfg(feature = "indicatif-progress")]
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{CompatMode, Session, SessionBuilder, Table};
use tpcdsgen::error::TpcdsError;

pub mod csv;
pub mod dat;
mod generate;
pub mod parquet;
mod plan;
mod progress;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_TPCDS_PARQUET_ROW_GROUP_BYTES: usize = DEFAULT_PARQUET_ROW_GROUP_BYTES as usize;

enum OutputFormat {
    Dat(dat::Dat),
    Csv(csv::Csv),
    Parquet(parquet::Parquet),
}

#[derive(Args)]
#[command(version)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    args: DatArgs,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate DAT (pipe-delimited) output
    Dat(DatArgs),
    /// Generate CSV output with CSV-specific options
    Csv(CsvArgs),
    /// Generate Apache Parquet output with Parquet-specific options
    Parquet(ParquetArgs),
}

#[derive(Args)]
struct DatArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Args)]
struct CsvArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// CSV delimiter character (default: ',')
    ///
    /// Specifies the delimiter character to use when generating CSV files.
    ///
    /// Supports escape sequences: \t (tab), \n (newline), \r (carriage return), \\ (backslash)
    /// Common delimiters: ',' (comma), '|' (pipe), '\t' (tab), ';' (semicolon)
    #[arg(long, default_value = ",", value_parser = parse_delimiter)]
    delimiter: char,
}

#[derive(Args)]
struct ParquetArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// Parquet block compression format.
    ///
    /// Supported values: UNCOMPRESSED, ZSTD(N), SNAPPY, GZIP, LZO, BROTLI, LZ4
    ///
    /// Note to use zstd you must supply the "compression" level (1-22)
    /// as a number in parentheses, e.g. `ZSTD(1)` for level 1 compression.
    ///
    /// Using `ZSTD` results in the best compression, but is about 2x slower than
    /// UNCOMPRESSED. For example, for the lineitem table at SF=10
    ///
    ///   ZSTD(1):      1.9G  (0.52 GB/sec)
    ///   SNAPPY:       2.4G  (0.75 GB/sec)
    ///   UNCOMPRESSED: 3.8G  (1.41 GB/sec)
    #[arg(short = 'c', long, default_value = "SNAPPY")]
    compression: Compression,

    /// Target size in row group bytes in Parquet files
    ///
    /// Row groups are the typical unit of parallel processing and compression
    /// with many query engines. Therefore, smaller row groups enable better
    /// parallelism and lower peak memory use but may reduce compression
    /// efficiency.
    ///
    /// Note: Parquet files are limited to 32k row groups, so at high scale
    /// factors, the row group size may be increased to keep the number of row
    /// groups under this limit.
    ///
    /// Typical values range from 10MB to 100MB.
    #[arg(
        long,
        default_value_t = DEFAULT_TPCDS_PARQUET_ROW_GROUP_BYTES,
        value_parser = parse_row_group_bytes
    )]
    row_group_bytes: usize,

    /// The number of threads for parallel generation, defaults to the number of CPUs
    #[arg(
        short,
        long,
        default_value_t = num_cpus::get(),
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..)
    )]
    num_threads: usize,
}

#[derive(Args)]
pub struct CommonArgs {
    /// Scale factor to create (supported range: 0 through 100000, inclusive)
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short = 'T', long = "tables", value_delimiter = ',', value_parser = TableValueParser)]
    tables: Option<Vec<Table>>,

    /// Reference implementation to match (default: trino)
    #[arg(long, default_value_t = CompatMode::Trino)]
    compat: CompatMode,

    /// Number of part(itions) to generate. If not specified creates a single file per table
    #[arg(short, long)]
    parts: Option<i32>,

    /// Which part(ition) to generate (1-based). If not specified, generates all parts
    #[arg(long)]
    part: Option<i32>,

    /// Verbose output
    ///
    /// When specified, sets the log level to `info` and ignores the `RUST_LOG`
    /// environment variable. When not specified, uses `RUST_LOG`
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet mode - only show error-level logs
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,

    /// Disable progress bars during data generation.
    ///
    /// Bars are also auto-suppressed by `--quiet` or when stderr is not a terminal.
    #[arg(long = "no-progress", action = ArgAction::SetFalse, default_value_t = true)]
    progress_bars_enabled: bool,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Some(Commands::Dat(args)) => args.run().await,
            Some(Commands::Csv(args)) => args.run().await,
            Some(Commands::Parquet(args)) => args.run().await,
            None => self.args.run().await,
        }
    }
}

impl DatArgs {
    async fn run(self) -> Result<()> {
        self.common.run_dat().await
    }
}

impl CsvArgs {
    async fn run(self) -> Result<()> {
        self.common.run_csv(self.delimiter).await
    }
}

impl ParquetArgs {
    async fn run(self) -> Result<()> {
        self.common
            .run_parquet(self.compression, self.row_group_bytes, self.num_threads)
            .await
    }
}

impl CommonArgs {
    async fn run_dat(self) -> Result<()> {
        let output = dat::Dat::new(self.output_dir.clone())?;
        let tables = self.row_generator_tables()?;
        self.run_output_with_tables(OutputFormat::Dat(output), tables)
            .await
    }

    async fn run_parquet(
        self,
        compression: Compression,
        row_group_bytes: usize,
        num_threads: usize,
    ) -> Result<()> {
        let output = parquet::Parquet::new(
            self.output_dir.clone(),
            compression,
            row_group_bytes,
            num_threads,
        );
        self.run_output(OutputFormat::Parquet(output)).await
    }

    async fn run_csv(self, delimiter: char) -> Result<()> {
        let output = csv::Csv::new(self.output_dir.clone(), delimiter);
        let tables = self.row_generator_tables()?;
        self.run_output_with_tables(OutputFormat::Csv(output), tables)
            .await
    }

    async fn run_output(self, output_format: OutputFormat) -> Result<()> {
        let tables = self.tables()?;
        self.run_output_with_tables(output_format, tables).await
    }

    async fn run_output_with_tables(
        self,
        output_format: OutputFormat,
        tables: Vec<Table>,
    ) -> Result<()> {
        let (progress, log_writer) = self.progress_tracker();
        configure_logging(self.verbose, self.quiet, log_writer);

        std::fs::create_dir_all(&self.output_dir)?;

        let parts = self.part_list()?;

        match output_format {
            // Parquet generates all tables in one call so that multiple
            // tables can be generated concurrently
            OutputFormat::Parquet(output) => {
                let mut table_sessions = Vec::with_capacity(tables.len() * parts.len());
                for table in &tables {
                    for &part in &parts {
                        let session = self.to_session(Some(table.get_name().to_string()), part)?;
                        table_sessions.push((*table, session));
                    }
                }
                output
                    .generate_tables(table_sessions, progress.clone())
                    .await?;
            }
            OutputFormat::Dat(output) => {
                let mut table_sessions = Vec::with_capacity(tables.len() * parts.len());
                for table in &tables {
                    for &part in &parts {
                        let session = self.to_session(Some(table.get_name().to_string()), part)?;
                        let progress = output.register_table(*table, &session, progress.clone());
                        table_sessions.push((*table, session, progress));
                    }
                }
                progress.start();
                for (table, session, progress) in table_sessions {
                    output.generate_table(table, &session, progress)?;
                }
            }
            OutputFormat::Csv(output) => {
                let mut table_sessions = Vec::with_capacity(tables.len() * parts.len());
                for table in &tables {
                    for &part in &parts {
                        let session = self.to_session(Some(table.get_name().to_string()), part)?;
                        let progress = output.register_table(*table, &session, progress.clone());
                        table_sessions.push((*table, session, progress));
                    }
                }
                progress.start();
                for (table, session, progress) in table_sessions {
                    output.generate_table(table, &session, progress)?;
                }
            }
        }

        progress.finish();
        Ok(())
    }

    fn progress_tracker(
        &self,
    ) -> (
        Arc<dyn ProgressTracker>,
        Option<Box<dyn io::Write + Send + 'static>>,
    ) {
        #[cfg(feature = "indicatif-progress")]
        if self.progress_bars_enabled && !self.quiet && io::stderr().is_terminal() {
            let progress = Arc::new(IndicatifProgress::new());
            let tracker: Arc<dyn ProgressTracker> = progress.clone();
            return (tracker, Some(progress.log_writer()));
        }

        (no_op_progress_tracker(), None)
    }

    /// Return the tables that should be generated.
    fn tables(&self) -> Result<Vec<Table>> {
        let tables = self.tables.clone().unwrap_or_else(Table::main_tables);
        let mut seen = HashSet::new();
        Ok(tables
            .into_iter()
            .filter(|table| seen.insert(*table))
            .collect())
    }

    /// Return the tables to generate for the row-generator outputs (DAT and
    /// CSV), mapping return-only selections to their sales table generators
    /// because return files are emitted as side effects of the sales tables.
    /// Parquet has direct return-table generators and does not need expansion.
    fn row_generator_tables(&self) -> Result<Vec<Table>> {
        let mut tables = Vec::new();
        for table in self.tables()? {
            let table = match table {
                Table::CatalogReturns => Table::CatalogSales,
                Table::StoreReturns => Table::StoreSales,
                Table::WebReturns => Table::WebSales,
                table => table,
            };
            if !tables.contains(&table) {
                tables.push(table);
            }
        }
        Ok(tables)
    }

    /// Return the list of 1-based part numbers to generate, or `[None]` when
    /// no `--part`/`--parts` were given (a single, unnumbered file per
    /// table).
    ///
    /// Mirrors `tpchgen-cli`'s `--parts`/`--part` semantics: `--parts` alone
    /// generates every part as a separate file, `--part` requires `--parts`
    /// to be set alongside it and restricts generation to just that part.
    fn part_list(&self) -> Result<Vec<Option<i32>>> {
        match (self.part, self.parts) {
            (Some(_), None) => Err(TpcdsError::new(
                "The --part option requires the --parts option to be set",
            )
            .into()),
            (None, Some(parts)) if parts < 1 => Err(TpcdsError::new(&format!(
                "Invalid --parts value '{parts}'. Expected a number greater than zero"
            ))
            .into()),
            (None, Some(parts)) => Ok((1..=parts).map(Some).collect()),
            (Some(part), Some(_)) => Ok(vec![Some(part)]),
            (None, None) => Ok(vec![None]),
        }
    }

    fn to_session(&self, table: Option<String>, part: Option<i32>) -> Result<Session> {
        let table = table.as_deref().map(parse_table).transpose()?;

        // store the command line arguments used to create this
        let command_line_arguments = std::env::args().collect::<Vec<_>>().join(" ");

        let mut builder = SessionBuilder::new()
            .with_scale_factor(self.scale_factor)
            .with_compat_mode(self.compat)
            .with_chunk_number(part.unwrap_or(1))
            .with_total_chunks(self.parts.unwrap_or(1))
            .with_command_line_arguments(command_line_arguments);

        if let Some(table) = table {
            builder = builder.with_table(table);
        }

        Ok(builder.build()?)
    }
}

fn parse_table(table: &str) -> Result<Table> {
    let parsed = table.parse::<Table>().map_err(|_| {
        TpcdsError::new(&format!(
            "unknown table '{table}'. Expected one of: {}",
            expected_table_names()
        ))
    })?;

    if parsed.is_main_table() {
        Ok(parsed)
    } else {
        Err(TpcdsError::new(&format!(
            "unknown table '{table}'. Expected one of: {}",
            expected_table_names()
        ))
        .into())
    }
}

/// Parses a TPC-DS table name, and supplies the list of table names (with
/// descriptions) shown in `--help`.
#[derive(Debug, Clone)]
struct TableValueParser;

impl TypedValueParser for TableValueParser {
    type Value = Table;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let to_err = |msg: String| {
            clap::Error::raw(clap::error::ErrorKind::InvalidValue, format!("{msg}\n")).with_cmd(cmd)
        };

        let value = value
            .to_str()
            .ok_or_else(|| to_err("table names must be valid UTF-8".to_string()))?;

        parse_table(value).map_err(|e| to_err(e.to_string()))
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(Table::main_tables().into_iter().map(|table| {
            clap::builder::PossibleValue::new(table.get_name()).help(table_help(table))
        })))
    }
}

/// One line description of each TPC-DS table, shown in `--help`.
fn table_help(table: Table) -> &'static str {
    match table {
        Table::CallCenter => "Call center dimension",
        Table::CatalogPage => "Catalog page dimension",
        Table::CatalogReturns => "Catalog returns fact",
        Table::CatalogSales => "Catalog sales fact",
        Table::Customer => "Customer dimension",
        Table::CustomerAddress => "Customer address dimension",
        Table::CustomerDemographics => "Customer demographics dimension",
        Table::DateDim => "Date dimension",
        Table::HouseholdDemographics => "Household demographics dimension",
        Table::IncomeBand => "Income band dimension",
        Table::Inventory => "Inventory fact",
        Table::Item => "Item dimension",
        Table::Promotion => "Promotion dimension",
        Table::Reason => "Reason dimension",
        Table::ShipMode => "Ship mode dimension",
        Table::Store => "Store dimension",
        Table::StoreReturns => "Store returns fact",
        Table::StoreSales => "Store sales fact",
        Table::TimeDim => "Time dimension",
        Table::Warehouse => "Warehouse dimension",
        Table::WebPage => "Web page dimension",
        Table::WebReturns => "Web returns fact",
        Table::WebSales => "Web sales fact",
        Table::WebSite => "Web site dimension",
        Table::DbgenVersion => "Metadata about the generator run",
        // source tables are not selectable on the command line
        _ => "",
    }
}

fn expected_table_names() -> String {
    Table::main_tables()
        .iter()
        .map(Table::get_name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_delimiter(s: &str) -> std::result::Result<char, String> {
    let parsed = match s {
        "\\t" => '\t',
        "\\n" => '\n',
        "\\r" => '\r',
        "\\\\" => '\\',
        _ => {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() != 1 {
                return Err(format!(
                    "Delimiter must be a single character or escape sequence (\\t, \\n, \\r, \\\\), got: '{}'",
                    s
                ));
            }
            chars[0]
        }
    };
    if !parsed.is_ascii() {
        return Err(format!(
            "Delimiter must be an ASCII character, got: '{}'",
            parsed
        ));
    }
    Ok(parsed)
}

fn parse_row_group_bytes(s: &str) -> std::result::Result<usize, String> {
    let parsed = s.parse::<usize>().map_err(|e| e.to_string())?;
    if parsed == 0 {
        Err("must be greater than zero".to_string())
    } else {
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_tables(tables: Vec<Table>) -> CommonArgs {
        CommonArgs {
            scale_factor: 1.0,
            output_dir: PathBuf::new(),
            tables: Some(tables),
            compat: CompatMode::Trino,
            parts: None,
            part: None,
            verbose: false,
            quiet: false,
            progress_bars_enabled: false,
        }
    }

    #[test]
    fn every_main_table_has_help_text() {
        for table in Table::main_tables() {
            assert!(
                !table_help(table).is_empty(),
                "{table} is missing a --help description"
            );
        }
    }

    #[test]
    fn tables_deduplicates_repeated_selections_in_first_seen_order() {
        let tables = args_with_tables(vec![
            Table::Reason,
            Table::Reason,
            Table::ShipMode,
            Table::Reason,
            Table::ShipMode,
        ])
        .tables()
        .unwrap();

        assert_eq!(tables, vec![Table::Reason, Table::ShipMode]);
    }

    #[test]
    fn row_generator_tables_collapses_sales_returns_pairs_in_both_orders() {
        for (sales, returns) in [
            (Table::CatalogSales, Table::CatalogReturns),
            (Table::StoreSales, Table::StoreReturns),
            (Table::WebSales, Table::WebReturns),
        ] {
            assert_eq!(
                args_with_tables(vec![sales, returns])
                    .row_generator_tables()
                    .unwrap(),
                vec![sales]
            );
            assert_eq!(
                args_with_tables(vec![returns, sales])
                    .row_generator_tables()
                    .unwrap(),
                vec![sales]
            );
            assert_eq!(
                args_with_tables(vec![sales])
                    .row_generator_tables()
                    .unwrap(),
                vec![sales]
            );
            assert_eq!(
                args_with_tables(vec![returns])
                    .row_generator_tables()
                    .unwrap(),
                vec![sales]
            );
        }
    }

    #[test]
    fn row_generator_tables_normalizes_mixed_pairs_and_duplicates() {
        let tables = args_with_tables(vec![
            Table::StoreReturns,
            Table::Reason,
            Table::StoreSales,
            Table::StoreReturns,
            Table::CatalogSales,
            Table::CatalogReturns,
            Table::WebReturns,
            Table::WebSales,
            Table::Reason,
        ])
        .row_generator_tables()
        .unwrap();

        assert_eq!(
            tables,
            vec![
                Table::StoreSales,
                Table::Reason,
                Table::CatalogSales,
                Table::WebSales,
            ]
        );
    }

    fn args_with_parts(parts: Option<i32>, part: Option<i32>) -> CommonArgs {
        let mut args = args_with_tables(vec![Table::Reason]);
        args.parts = parts;
        args.part = part;
        args
    }

    #[test]
    fn part_list_defaults_to_single_unnumbered_file() {
        assert_eq!(args_with_parts(None, None).part_list().unwrap(), vec![None]);
    }

    #[test]
    fn part_list_expands_parts_alone_into_every_part() {
        assert_eq!(
            args_with_parts(Some(3), None).part_list().unwrap(),
            vec![Some(1), Some(2), Some(3)]
        );
    }

    #[test]
    fn part_list_with_part_and_parts_generates_just_that_part() {
        assert_eq!(
            args_with_parts(Some(3), Some(2)).part_list().unwrap(),
            vec![Some(2)]
        );
    }

    #[test]
    fn part_list_rejects_part_without_parts() {
        let err = args_with_parts(None, Some(2)).part_list().unwrap_err();
        assert_eq!(
            err.to_string(),
            "The --part option requires the --parts option to be set"
        );
    }

    #[test]
    fn part_list_rejects_non_positive_parts() {
        assert!(args_with_parts(Some(0), None).part_list().is_err());
        assert!(args_with_parts(Some(-1), None).part_list().is_err());
    }
}
