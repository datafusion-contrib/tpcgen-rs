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

/// `-V`/`--version` reports this package's name and version, not `tpcgen-cli`'s.
#[test]
fn test_tpchgen_cli_version() {
    let expected = format!("tpchgen-cli {}\n", env!("CARGO_PKG_VERSION"));
    for flag in ["-V", "--version"] {
        let output = cargo_bin_cmd!("tpchgen-cli")
            .arg(flag)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }
}

#[test]
fn test_tpchgen_cli_invalid_inputs_before_output() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("output");
    cargo_bin_cmd!("tpchgen-cli")
        .args(["--tables=region", "--parts=0"])
        .arg("--output-dir")
        .arg(&output)
        .assert()
        .failure()
        .stdout("");
    assert!(!output.exists());
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
