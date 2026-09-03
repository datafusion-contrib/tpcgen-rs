use super::{
    Compression, OutputFormat, Table, TpchGenerator, TpchGeneratorBuilder,
    DEFAULT_PARQUET_ROW_GROUP_BYTES,
};
use crate::logging::configure_logging;
#[cfg(feature = "indicatif-progress")]
use crate::progress::IndicatifProgress;
use clap::builder::TypedValueParser;
use clap::{ArgAction, Parser};
use std::collections::HashSet;
use std::io;
#[cfg(feature = "indicatif-progress")]
use std::io::IsTerminal;
use std::path::PathBuf;
use std::str::FromStr;
#[cfg(feature = "indicatif-progress")]
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "tpchgen")]
#[command(version)]
#[command(
    // -h output
    about = "TPC-H Data Generator",
    // --help output
    long_about = r#"
TPCH Data Generator (https://github.com/datafusion-contrib/tpcgen-rs)

By default each table is written to a single file named <output_dir>/<table>.<format>

If `--part` option is specified, each table is written to a subdirectory in
multiple files named <output_dir>/<table>/<table>.<part>.<format>

Examples

# Generate all tables at scale factor 1 (1GB) in TBL format (default) to /tmp/tpch directory:

tpchgen-cli -s 1 --output-dir=/tmp/tpch

# Generate all tables in CSV format:

tpchgen-cli csv -s 1 --output-dir=/tmp/tpch

# Generate scale factor one in CSV format with tab delimiter:

tpchgen-cli csv -s 1 --delimiter='\t' --output-dir=/tmp/tpch

# Generate the lineitem table at scale factor 100 in 10 Apache Parquet files to
# /tmp/tpch/lineitem:

tpchgen-cli parquet -s 100 --tables=lineitem --parts=10 --output-dir=/tmp/tpch

# Generate scale factor one in current directory, seeing debug output

RUST_LOG=debug tpchgen-cli -s 1 --output-dir=/tmp/tpch
"#,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    args: CommonArgs,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate TBL (pipe-delimited) output
    Tbl(TblArgs),
    /// Generate CSV output with CSV-specific options
    Csv(CsvArgs),
    /// Generate Apache Parquet output with Parquet-specific options
    Parquet(ParquetArgs),
}

#[derive(clap::Args)]
struct CommonArgs {
    /// Scale factor to create (supported range: 0 through 100000, inclusive)
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short = 'T', long = "tables", value_delimiter = ',', value_parser = TableValueParser)]
    tables: Option<Vec<Table>>,

    /// Number of part(itions) to generate. If not specified creates a single file per table
    #[arg(short, long)]
    parts: Option<i32>,

    /// Which part(ition) to generate (1-based). If not specified, generates all parts
    #[arg(long)]
    part: Option<i32>,

    /// The number of threads for parallel generation, defaults to the number of CPUs
    #[arg(
        short,
        long,
        default_value_t = num_cpus::get(),
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..)
    )]
    num_threads: usize,

    /// Verbose output
    ///
    /// When specified, sets the log level to `info` and ignores the `RUST_LOG`
    /// environment variable. When not specified, uses `RUST_LOG`
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet mode - only show error-level logs
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,

    /// Write the output to stdout instead of a file.
    #[arg(long, default_value_t = false)]
    stdout: bool,

    /// Disable progress bars during data generation.
    ///
    /// Bars are also auto-suppressed by `--quiet`, `--stdout`, or when
    /// stderr is not a terminal.
    #[arg(long = "no-progress", action = ArgAction::SetFalse, default_value_t = true)]
    progress_bars_enabled: bool,
}

impl CommonArgs {
    /// Initialize CLI logging/progress output and create a
    /// [`TpchGeneratorBuilder`] pre-configured with the common options.
    fn builder(self, format: OutputFormat) -> TpchGeneratorBuilder {
        let tables = self.tables();

        #[cfg(feature = "indicatif-progress")]
        let progress = self
            .should_show_progress_bars()
            .then(|| Arc::new(IndicatifProgress::new()));

        let mut builder = TpchGenerator::builder()
            .with_scale_factor(self.scale_factor)
            .with_output_dir(self.output_dir)
            .with_format(format)
            .with_num_threads(self.num_threads)
            .with_stdout(self.stdout);

        if let Some(tables) = tables {
            builder = builder.with_tables(tables);
        }
        if let Some(parts) = self.parts {
            builder = builder.with_parts(parts);
        }
        if let Some(part) = self.part {
            builder = builder.with_part(part);
        }

        #[cfg(feature = "indicatif-progress")]
        configure_logging(
            self.verbose,
            self.quiet,
            progress.as_ref().map(|progress| progress.log_writer()),
        );
        #[cfg(not(feature = "indicatif-progress"))]
        configure_logging(self.verbose, self.quiet, None);

        #[cfg(feature = "indicatif-progress")]
        if let Some(progress) = progress {
            builder = builder.with_progress_tracker(progress);
        }

        builder
    }

    /// Return the selected tables without repeated values, preserving the
    /// command-line order of their first occurrence.
    fn tables(&self) -> Option<Vec<Table>> {
        let mut seen = HashSet::new();
        self.tables.as_ref().map(|tables| {
            tables
                .iter()
                .copied()
                .filter(|table| seen.insert(*table))
                .collect()
        })
    }

    #[cfg(feature = "indicatif-progress")]
    fn should_show_progress_bars(&self) -> bool {
        // Show progress only on an interactive terminal and when no flag
        // suppresses it. `--stdout` is included so piped data isn't
        // interleaved with bar redraws on shared shells.
        self.progress_bars_enabled && !self.quiet && !self.stdout && io::stderr().is_terminal()
    }
}

#[derive(clap::Args)]
struct TblArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(clap::Args)]
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

#[derive(clap::Args)]
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
    #[arg(long, default_value_t = DEFAULT_PARQUET_ROW_GROUP_BYTES)]
    row_group_bytes: i64,
}

/// Parse a delimiter string, handling escape sequences.
///
/// The underlying arrow-csv writer requires an ASCII byte for the delimiter,
/// so non-ASCII characters are rejected here rather than failing mid-generation.
fn parse_delimiter(s: &str) -> Result<char, String> {
    // Handle common escape sequences
    let parsed = match s {
        "\\t" => '\t',
        "\\n" => '\n',
        "\\r" => '\r',
        "\\\\" => '\\',
        _ => {
            // If it's not an escape sequence, it should be a single character
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

// TableValueParser is CLI-specific and uses the Table type from the library
#[derive(Debug, Clone)]
struct TableValueParser;

impl TypedValueParser for TableValueParser {
    type Value = Table;

    /// Parse the value into a Table enum.
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value
            .to_str()
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd))?;
        Table::from_str(value)
            .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd))
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(
            [
                clap::builder::PossibleValue::new("region").help("Region table (alias: r)"),
                clap::builder::PossibleValue::new("nation").help("Nation table (alias: n)"),
                clap::builder::PossibleValue::new("supplier").help("Supplier table (alias: s)"),
                clap::builder::PossibleValue::new("customer").help("Customer table (alias: c)"),
                clap::builder::PossibleValue::new("part").help("Part table (alias: P)"),
                clap::builder::PossibleValue::new("partsupp").help("PartSupp table (alias: S)"),
                clap::builder::PossibleValue::new("orders").help("Orders table (alias: O)"),
                clap::builder::PossibleValue::new("lineitem").help("LineItem table (alias: L)"),
            ]
            .into_iter(),
        ))
    }
}

impl Cli {
    /// Run data generation for the selected command.
    pub async fn run(self) -> io::Result<()> {
        match self.command {
            Some(Commands::Tbl(args)) => args.run().await,
            Some(Commands::Csv(args)) => args.run().await,
            Some(Commands::Parquet(args)) => args.run().await,
            None => self.run_default().await,
        }
    }

    /// Generate TBL output when no output-format subcommand is specified.
    async fn run_default(self) -> io::Result<()> {
        self.args
            .builder(OutputFormat::Tbl)
            .build()
            .generate()
            .await
    }
}

impl TblArgs {
    async fn run(self) -> io::Result<()> {
        self.common
            .builder(OutputFormat::Tbl)
            .build()
            .generate()
            .await
    }
}

impl CsvArgs {
    async fn run(self) -> io::Result<()> {
        self.common
            .builder(OutputFormat::Csv)
            .with_csv_delimiter(self.delimiter)
            .build()
            .generate()
            .await
    }
}

impl ParquetArgs {
    async fn run(self) -> io::Result<()> {
        self.common
            .builder(OutputFormat::Parquet)
            .with_parquet_compression(self.compression)
            .with_parquet_row_group_bytes(self.row_group_bytes)
            .build()
            .generate()
            .await
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
            parts: None,
            part: None,
            num_threads: 1,
            verbose: false,
            quiet: false,
            stdout: false,
            progress_bars_enabled: false,
        }
    }

    #[test]
    fn tables_deduplicates_repeated_selections_in_first_seen_order() {
        let tables = args_with_tables(vec![
            Table::Region,
            Table::Region,
            Table::Nation,
            Table::Region,
            Table::Nation,
        ])
        .tables();

        assert_eq!(tables, Some(vec![Table::Region, Table::Nation]));
    }

    #[test]
    fn tables_deduplicates_values_from_repeated_flags_and_aliases() {
        let cli = Cli::try_parse_from([
            "tpchgen", "tbl", "--tables", "region,r", "--tables", "nation", "--tables", "region",
        ])
        .unwrap();
        let Some(Commands::Tbl(args)) = cli.command else {
            panic!("expected tbl command")
        };

        assert_eq!(
            args.common.tables(),
            Some(vec![Table::Region, Table::Nation])
        );
    }
}
