use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;

#[path = "cli_integration/test_helpers.rs"]
mod test_helpers;

// TPCH-specific CLI coverage
#[path = "cli_integration/tpch.rs"]
mod tpch;

// TPC-DS-specific CLI coverage
#[path = "cli_integration/tpcds.rs"]
mod tpcds;

/// Scales above SF100000 warn once on stderr; SF100000 does not.
#[test]
fn test_above_benchmark_scale_warning() {
    for (scale, warnings) in [("100000", 0), ("100001", 1)] {
        let output = cargo_bin_cmd!("tpcgen-cli")
            .env_remove("RUST_LOG")
            .args(["tpch", "-s", scale, "-T", "region", "--stdout"])
            .assert()
            .success()
            .get_output()
            .clone();
        assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 5);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            stderr.matches("generated data may not be valid").count(),
            warnings,
            "{stderr}"
        );
    }
}

/// `-v` logs an INFO message once for an unapproved in-range scale factor, and
/// nothing for an approved one.
#[test]
fn test_unapproved_scale_factor_info() {
    for (benchmark, table, unapproved) in [("tpch", "region", "2"), ("tpcds", "ship_mode", "10")] {
        for (scale, messages) in [("1", 0), (unapproved, 1)] {
            let output = cargo_bin_cmd!("tpcgen-cli")
                .env_remove("RUST_LOG")
                .args([benchmark, "-s", scale, "-T", table, "--stdout", "-v"])
                .assert()
                .success()
                .get_output()
                .clone();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(
                stderr.matches("is not an approved").count(),
                messages,
                "{stderr}"
            );
        }
    }
}

/// TPC-DS still rejects SF > 100000 in its library, before creating output.
#[test]
fn test_tpcds_above_benchmark_scale_rejected_before_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output");
    cargo_bin_cmd!("tpcgen-cli")
        .args(["tpcds", "-s", "100001", "-T", "ship_mode"])
        .arg("--output-dir")
        .arg(&output)
        .assert()
        .failure()
        .stdout("")
        .stderr(predicates::str::contains(
            "Scale must be between 0 and 100000",
        ));
    assert!(!output.exists());
}

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

/// Help text refers to `tpcgen-cli`, never to the `tpchgen-cli` binary.
#[test]
fn test_tpcgen_cli_help_uses_binary_name() {
    for args in [
        ["--help"].as_slice(),
        &["tpch", "--help"],
        &["tpcds", "--help"],
    ] {
        cargo_bin_cmd!("tpcgen-cli")
            .args(args)
            .assert()
            .success()
            .stdout(predicates::str::contains("tpcgen-cli"))
            .stdout(predicates::str::contains("tpchgen-cli").not());
    }
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
