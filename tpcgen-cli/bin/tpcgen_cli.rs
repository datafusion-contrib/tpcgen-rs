//! Binary entry point for the TPC-H and TPC-DS data generator.

use clap::{Parser, Subcommand};
use std::process::ExitCode;
use tpcgen_cli::tpcds_cli::Cli as TpcdsCli;
use tpcgen_cli::tpch_cli::Cli as TpchCli;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "tpcgen-cli")]
#[command(version)]
#[command(
    about = "TPC-H and TPC-DS data generator",
    long_about = r#"
TPC-H and TPC-DS data generator (https://github.com/datafusion-contrib/tpcgen-rs)

Examples

# TPC-H TBL data:

tpcgen-cli tpch -s 1 --output-dir=/tmp/tpch

# TPC-H CSV data:

tpcgen-cli tpch csv -s 1 --output-dir=/tmp/tpch

# TPC-H Apache Parquet data:

tpcgen-cli tpch parquet -s 100 --tables=lineitem --parts=10 --output-dir=/tmp/tpch

# TPC-DS DAT data:

tpcgen-cli tpcds -s 1 --output-dir=/tmp/tpcds

# TPC-DS CSV data:

tpcgen-cli tpcds csv -s 1 --output-dir=/tmp/tpcds

# TPC-DS Apache Parquet data:

tpcgen-cli tpcds parquet -s 100 --tables=store_sales --parts=10 --output-dir=/tmp/tpcds

See `tpcgen-cli tpch --help` and `tpcgen-cli tpcds --help` for the full set of
options for each benchmark.
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// TPC-H data
    #[command(long_about = r#"
Generate TPC-H data

By default each table is written to a single file named
<output_dir>/<table>.<format>

If the `--parts` option is specified, each table is written to a subdirectory in
multiple files named <output_dir>/<table>/<table>.<part>.<format>

Examples

# Generate all tables at scale factor 1 (1GB) in TBL format (the default):

tpcgen-cli tpch -s 1 --output-dir=/tmp/tpch

# Generate all tables in CSV format, with a tab delimiter:

tpcgen-cli tpch csv -s 1 --delimiter='\t' --output-dir=/tmp/tpch

# Generate the lineitem table at scale factor 100 in 10 Apache Parquet files to
# /tmp/tpch/lineitem:

tpcgen-cli tpch parquet -s 100 --tables=lineitem --parts=10 --output-dir=/tmp/tpch

# Generate scale factor one, seeing debug output:

RUST_LOG=debug tpcgen-cli tpch -s 1 --output-dir=/tmp/tpch
"#)]
    Tpch(TpchCli),
    /// TPC-DS data
    #[command(long_about = r#"
Generate TPC-DS data

By default each table is written to a single file named
<output_dir>/<table>.<format>

If the `--parts` option is specified, each table is written to a subdirectory in
multiple files named <output_dir>/<table>/<table>.<part>.<format>

Examples

# Generate all tables at scale factor 1 (1GB) in DAT format (the default):

tpcgen-cli tpcds -s 1 --output-dir=/tmp/tpcds

# Generate all tables in CSV format, with a tab delimiter:

tpcgen-cli tpcds csv -s 1 --delimiter='\t' --output-dir=/tmp/tpcds

# Generate the store_sales table at scale factor 100 in 10 Apache Parquet files
# to /tmp/tpcds/store_sales:

tpcgen-cli tpcds parquet -s 100 --tables=store_sales --parts=10 --output-dir=/tmp/tpcds

# Match the C `dsdgen` reference implementation instead of the Java/Trino one:

tpcgen-cli tpcds -s 1 --compat c --output-dir=/tmp/tpcds
"#)]
    Tpcds(TpcdsCli),
}

impl Cli {
    async fn run(self) -> Result<()> {
        match self.command {
            Command::Tpch(args) => args.run().await?,
            Command::Tpcds(args) => args.run().await?,
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match Cli::parse().run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use tpcgen_cli::tpch_cli::Table;

    #[test]
    fn full_table_names_are_case_insensitive() {
        for (benchmark, name) in [
            ("tpch", "Nation"),
            ("tpch", "Region"),
            ("tpch", "Supplier"),
            ("tpch", "Part"),
            ("tpch", "PartSupp"),
            ("tpch", "Customer"),
            ("tpch", "Orders"),
            ("tpch", "LineItem"),
            ("tpcds", "Reason"),
            ("tpcds", "Ship_Mode"),
        ] {
            for name in [
                name.to_ascii_lowercase(),
                name.to_ascii_uppercase(),
                name.to_string(),
            ] {
                Cli::try_parse_from(["tpcgen-cli", benchmark, "--tables", &name]).unwrap();
            }
        }
    }

    #[test]
    fn tpch_table_aliases_remain_case_sensitive() {
        // Only TPC-H: the Rust TPC-DS CLI does not support table aliases.
        for (alias, table) in [
            ("n", Table::Nation),
            ("r", Table::Region),
            ("s", Table::Supplier),
            ("P", Table::Part),
            ("S", Table::Partsupp),
            ("c", Table::Customer),
            ("O", Table::Orders),
            ("L", Table::Lineitem),
        ] {
            let matches = Cli::command()
                .try_get_matches_from(["tpcgen-cli", "tpch", "--tables", alias])
                .unwrap();
            let tpch = matches.subcommand_matches("tpch").unwrap();
            assert_eq!(tpch.get_one::<Table>("tables"), Some(&table), "{alias}");
        }

        for alias in ["N", "R", "p", "C", "o", "l"] {
            assert!(
                Cli::try_parse_from(["tpcgen-cli", "tpch", "--tables", alias]).is_err(),
                "{alias}"
            );
        }
    }
}
