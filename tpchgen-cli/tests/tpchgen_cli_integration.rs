use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::tempdir;

/// Help text refers to `tpchgen-cli`, not `tpcgen-cli`.
#[test]
fn test_tpchgen_cli_help_uses_binary_name() {
    let output = cargo_bin_cmd!("tpchgen-cli")
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).unwrap();
    assert!(
        help.contains("tpchgen-cli -s 1 --output-dir=/tmp/tpch"),
        "{help}"
    );
    assert!(!help.contains("tpcgen-cli"), "{help}");
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
