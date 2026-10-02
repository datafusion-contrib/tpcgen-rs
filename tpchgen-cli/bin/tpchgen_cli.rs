use clap::Parser;
use std::process::ExitCode;
use tpcgen_cli::tpch_cli::Cli;

// Same arguments, subcommands and help as `tpcgen-cli tpch`, plus this
// package's own `-V`/`--version`.
#[derive(Parser)]
#[command(name = "tpchgen-cli", version)]
struct TpchgenCli {
    #[command(flatten)]
    cli: Cli,
}

#[tokio::main]
async fn main() -> ExitCode {
    match TpchgenCli::parse().cli.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
