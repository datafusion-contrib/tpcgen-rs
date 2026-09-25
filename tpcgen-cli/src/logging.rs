use log::{info, LevelFilter};
use std::io;

pub(crate) fn configure_logging(
    verbose: bool,
    quiet: bool,
    log_writer: Option<Box<dyn io::Write + Send + 'static>>,
) {
    let mut builder = env_logger::builder();
    if quiet {
        // Quiet mode: only show error-level logs
        builder.filter_level(LevelFilter::Error);
    } else if verbose {
        builder.filter_level(LevelFilter::Info);
    } else {
        // Default: show warnings and errors, but respect RUST_LOG if set
        builder.filter_level(LevelFilter::Warn).parse_default_env();
    }
    if let Some(log_writer) = log_writer {
        builder.target(env_logger::Target::Pipe(log_writer));
    }

    builder.init();

    if verbose && std::env::var_os("RUST_LOG").is_some() {
        info!("Verbose output enabled (ignoring RUST_LOG environment variable)");
    }
}
