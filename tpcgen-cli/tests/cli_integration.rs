use assert_cmd::cargo::cargo_bin_cmd;

#[path = "cli_integration/test_helpers.rs"]
mod test_helpers;

// TPCH-specific CLI coverage
#[path = "cli_integration/tpch.rs"]
mod tpch;

// TPC-DS-specific CLI coverage
#[path = "cli_integration/tpcds.rs"]
mod tpcds;

/// Test that invoking the CLI without a command reports the top-level usage.
#[test]
fn test_tpcgen_cli_requires_command() {
    cargo_bin_cmd!("tpcgen-cli")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Usage: tpcgen-cli <COMMAND>"))
        .stderr(predicates::str::contains("Commands:"))
        .stderr(predicates::str::contains("tpch"))
        .stderr(predicates::str::contains("tpcds"));
}

/// `--version` reports this crate's name and version everywhere it is
/// accepted, rather than clap's derived per-subcommand display names
/// (`tpcgen-cli-tpch`, `tpcgen-cli-tpcds`).
#[test]
fn test_tpcgen_cli_version_is_consistent() {
    let expected = format!("tpcgen-cli {}\n", env!("CARGO_PKG_VERSION"));

    for args in [
        vec!["--version"],
        vec!["tpch", "--version"],
        vec!["tpcds", "--version"],
    ] {
        cargo_bin_cmd!("tpcgen-cli")
            .args(&args)
            .assert()
            .success()
            .stdout(expected.clone());
    }
}

/// The help text for `tpcgen-cli` and its subcommands must use the `tpcgen-cli`
/// name. The `tpchgen-cli` name only belongs in the compatibility binary of the
/// same name (see `tpchgen-cli/tests/tpchgen_cli_integration.rs`).
#[test]
fn test_tpcgen_cli_help_uses_binary_name() {
    for args in [
        vec!["--help"],
        vec!["tpch", "--help"],
        vec!["tpcds", "--help"],
    ] {
        let output = cargo_bin_cmd!("tpcgen-cli")
            .args(&args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let help = String::from_utf8(output).expect("help output is valid UTF-8");

        assert!(
            help.contains("tpcgen-cli"),
            "`tpcgen-cli {}` help should mention tpcgen-cli:\n{help}",
            args.join(" ")
        );
        assert!(
            !help.contains("tpchgen-cli"),
            "`tpcgen-cli {}` help should not mention tpchgen-cli:\n{help}",
            args.join(" ")
        );
    }
}

#[test]
fn test_parquet_rejects_non_positive_row_group_bytes() {
    for (benchmark, table) in [("tpch", "region"), ("tpcds", "reason")] {
        for value in ["0", "-1"] {
            let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
            let output_dir = temp_dir.path().join("output");

            cargo_bin_cmd!("tpcgen-cli")
                .args([
                    benchmark,
                    "parquet",
                    "--scale-factor",
                    "0.001",
                    "--tables",
                    table,
                ])
                .arg("--output-dir")
                .arg(&output_dir)
                .arg(format!("--row-group-bytes={value}"))
                .assert()
                .code(2)
                .stdout("")
                .stderr(predicates::str::contains(format!(
                    "error: invalid value '{value}' for '--row-group-bytes <ROW_GROUP_BYTES>': must be greater than zero"
                )));

            assert!(
                !output_dir.exists(),
                "Invalid row-group size must not create output: {benchmark} {value}"
            );
        }
    }
}
