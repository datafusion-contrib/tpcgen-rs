use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn incompatible_encoding_fails_before_creating_output_directory() {
    let root = tempfile::tempdir_in(".").unwrap();
    let output_dir = root.path().join("nested/output");
    let result = cargo_bin_cmd!("tpchgen-cli")
        .args([
            "parquet",
            "--scale-factor",
            "0.001",
            "--tables",
            "nation,region",
            "--quiet",
            "--column-encoding",
            "r_regionkey=RLE",
        ])
        .arg("--output-dir")
        .arg(&output_dir)
        .assert()
        .failure()
        .stdout("");
    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(stderr.contains("table 'region'"), "{stderr}");
    assert!(
        stderr.contains("encoding RLE cannot encode column 'r_regionkey' of type INT64"),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!output_dir.exists());
    assert!(!output_dir.parent().unwrap().exists());
}
