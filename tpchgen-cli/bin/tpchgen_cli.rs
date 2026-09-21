//! Binary entry point for the backwards compatible `tpchgen-cli` TPC-H data
//! generator.
//!
//! The implementation lives in [`tpcgen_cli::tpch_cli`], which is also used for
//! `tpcgen-cli tpch`. The command is re-branded here so `--version` reports
//! `tpchgen-cli` and this package's version rather than those of the shared
//! `tpcgen-cli` implementation crate.

use clap::{CommandFactory, FromArgMatches};
use std::io;
use tpcgen_cli::tpch_cli::Cli;

#[tokio::main]
async fn main() -> io::Result<()> {
    let command = Cli::command()
        .name("tpchgen-cli")
        .version(env!("CARGO_PKG_VERSION"));

    let matches = command.get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());
    cli.run().await
}
