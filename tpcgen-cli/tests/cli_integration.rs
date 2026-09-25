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

#[test]
fn test_csv_rejects_unsupported_delimiters_before_generation() {
    for (benchmark, table) in [("tpch", "orders"), ("tpcds", "web_site")] {
        for delimiter in [
            "_", "-", ".", "/", ":", "@", "#", " ", "\"", "\n", "\r", "\\n", "\\r", "\\", "\\\\",
            "", "||", "\u{20ac}",
        ] {
            let temp_dir = tempfile::tempdir().unwrap();
            let output_dir = temp_dir.path().join("output");
            cargo_bin_cmd!("tpcgen-cli")
                .args([benchmark, "csv", "-s", "0.001", "-T", table])
                .arg(format!("--delimiter={delimiter}"))
                .arg("--output-dir")
                .arg(&output_dir)
                .assert()
                .code(2)
                .stdout("")
                .stderr(predicates::str::contains("CSV delimiter must be"));
            assert!(!output_dir.exists(), "Generation started for {delimiter:?}");
        }
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
