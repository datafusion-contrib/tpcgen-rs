use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use tempfile::tempdir;

/// `--version` reports this package's name and version, not those of the
/// `tpcgen-cli` crate the implementation lives in.
#[test]
fn test_tpchgen_cli_version() {
    cargo_bin_cmd!("tpchgen-cli")
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(format!(
            "tpchgen-cli {}",
            env!("CARGO_PKG_VERSION")
        )));
}

/// Help output refers to `tpchgen-cli`, not `tpcgen-cli`, so the examples match
/// the command the user typed.
#[test]
fn test_tpchgen_cli_help_uses_binary_name() {
    cargo_bin_cmd!("tpchgen-cli")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("tpchgen-cli -s 1 --output-dir=/tmp/tpch"))
        .stdout(contains("Usage: tpchgen-cli"));
}

/// Smoke test for `tpchgen-cli` binary.
#[test]
fn test_tpchgen_cli_command_forms() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpchgen-cli")
        .arg("tbl")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .assert()
        .success();

    let expected_file = temp_dir.path().join("part.tbl");
    assert!(
        expected_file.exists(),
        "Expected file {expected_file:?} to exist with `tpchgen-cli`",
    );
}
