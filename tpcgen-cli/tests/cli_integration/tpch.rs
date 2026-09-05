use super::test_helpers::{expect_column_encoding, expect_row_group_sizes, RowGroups};
use arrow::record_batch::RecordBatchReader;
use assert_cmd::cargo::cargo_bin_cmd;
use parquet::arrow::arrow_reader::{ArrowReaderOptions, ParquetRecordBatchReaderBuilder};
use parquet::basic::Encoding;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tempfile::tempdir;
use tpchgen::generators::OrderGenerator;
use tpchgen_arrow::OrderArrow;

/// Test the TPC-H command forms for the `tpcgen-cli` binary.
#[test]
fn test_tpcgen_cli_tpch_command_forms() {
    let forms: &[(&[&str], &[&str], &str)] = &[
        (&["tpch"], &[], "part.tbl"),
        (&["tpch", "tbl"], &[], "part.tbl"),
        (&["tpch", "csv"], &["--delimiter", "|"], "part.csv"),
        (
            &["tpch", "parquet"],
            &["--compression", "SNAPPY", "--row-group-bytes", "1000000"],
            "part.parquet",
        ),
    ];

    for (form, format_args, expected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen-cli")
            .args(*form)
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg("part")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .arg("--no-progress")
            .args(*format_args)
            .assert()
            .success();

        let expected_file = temp_dir.path().join(expected_file);
        assert!(
            expected_file.exists(),
            "Expected file {:?} to exist with `tpcgen-cli {}`",
            expected_file,
            form.join(" ")
        );
    }
}

#[test]
fn test_tpcgen_cli_tpch_parquet_column_encoding() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("lineitem")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .arg("--column-encoding")
        .arg("l_comment=DELTA_LENGTH_BYTE_ARRAY, l_shipinstruct = delta_length_byte_array ")
        .assert()
        .success();

    let path = temp_dir.path().join("lineitem.parquet");
    expect_column_encoding(&path, "l_comment", Encoding::DELTA_LENGTH_BYTE_ARRAY);
    expect_column_encoding(&path, "l_shipinstruct", Encoding::DELTA_LENGTH_BYTE_ARRAY);
}

#[test]
fn test_tpcgen_cli_tpch_parquet_rejects_invalid_column_encoding() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("l_comment=NOT_AN_ENCODING")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("invalid value") && stderr.contains("--column-encoding"),
        "unexpected stderr: {stderr}"
    );

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("nocolonequal")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("expected COLUMN=ENCODING"),
        "unexpected stderr: {stderr}"
    );

    for invalid in ["=PLAIN", "l_comment="] {
        let assert = cargo_bin_cmd!("tpcgen-cli")
            .args(["tpch", "parquet"])
            .arg("--output-dir")
            .arg(temp_dir.path())
            .arg("--column-encoding")
            .arg(invalid)
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(
            stderr.contains("expected COLUMN=ENCODING"),
            "unexpected stderr for {invalid}: {stderr}"
        );
    }
}

/// A `--column-encoding` column that exists on only some selected tables
/// applies there and is skipped elsewhere. Selecting tables that do not
/// share every named column is not an error.
#[test]
fn test_tpcgen_cli_tpch_parquet_column_encoding_applies_only_where_the_column_exists() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // l_comment only exists on lineitem, not orders.
    cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("lineitem,orders")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .arg("--column-encoding")
        .arg("l_comment=DELTA_LENGTH_BYTE_ARRAY")
        .assert()
        .success();

    let lineitem_path = temp_dir.path().join("lineitem.parquet");
    expect_column_encoding(
        &lineitem_path,
        "l_comment",
        Encoding::DELTA_LENGTH_BYTE_ARRAY,
    );
    assert!(
        temp_dir.path().join("orders.parquet").exists(),
        "expected orders.parquet to still be generated, just without l_comment applied to it"
    );
}

/// A `--column-encoding` column that matches no selected table (a typo)
/// must fail before any table is written.
#[test]
fn test_tpcgen_cli_tpch_parquet_column_encoding_typo_fails_before_any_output() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("lineitem,orders")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .arg("--column-encoding")
        .arg("l_comment_typo=DELTA_LENGTH_BYTE_ARRAY")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("column 'l_comment_typo'"),
        "unexpected stderr: {stderr}"
    );
    assert_eq!(
        fs::read_dir(temp_dir.path())
            .expect("Failed to read output directory")
            .count(),
        0,
        "expected no output files when validation fails before generation starts"
    );
}

/// PLAIN_DICTIONARY, RLE_DICTIONARY, and BIT_PACKED are always rejected.
/// This must fail before any table is written, same as a typo, even when
/// the column exists on only one of the selected tables.
#[test]
fn test_tpcgen_cli_tpch_parquet_dictionary_encoding_fails_before_any_output() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // l_comment only exists on lineitem. This must still fail up front,
    // before either table is scheduled.
    let assert = cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "parquet"])
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("lineitem,orders")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .arg("--column-encoding")
        .arg("l_comment=PLAIN_DICTIONARY")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("cannot be set with --column-encoding"),
        "unexpected stderr: {stderr}"
    );
    assert_eq!(
        fs::read_dir(temp_dir.path())
            .expect("Failed to read output directory")
            .count(),
        0,
        "expected no output files when validation fails before generation starts"
    );
}

/// Repeated TPC-H table selections should schedule each table once.
#[test]
fn test_tpcgen_cli_tpch_deduplicates_selected_tables() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .args(["tpch", "tbl"])
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region,region,nation,region,nation")
        .arg("--num-threads")
        .arg("4")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--verbose")
        .assert()
        .success();

    assert!(temp_dir.path().join("region.tbl").exists());
    assert!(temp_dir.path().join("nation.tbl").exists());
    assert_eq!(
        fs::read_dir(temp_dir.path())
            .expect("Failed to read generated output directory")
            .count(),
        2
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert_eq!(stderr.matches("Writing table region").count(), 1);
    assert_eq!(stderr.matches("Writing table nation").count(), 1);
}

/// Test TBL output for scale factor 0.001 using tpchgen-cli
#[test]
fn test_tpchgen_cli_tbl_scale_factor_0_001() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // Run the tpchgen-cli command
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    // List of expected files
    let expected_files = vec![
        "customer.tbl",
        "lineitem.tbl",
        "nation.tbl",
        "orders.tbl",
        "part.tbl",
        "partsupp.tbl",
        "region.tbl",
        "supplier.tbl",
    ];

    // Verify that all expected files are created
    for file in &expected_files {
        let generated_file = temp_dir.path().join(file);
        assert!(
            generated_file.exists(),
            "File {:?} does not exist",
            generated_file
        );
        let generated_contents = fs::read(generated_file).expect("Failed to read generated file");
        let generated_contents = String::from_utf8(generated_contents)
            .expect("Failed to convert generated contents to string");

        // load the reference file
        let reference_file = format!("../tpchgen/data/sf-0.001/{}.gz", file);
        let reference_contents = match read_gzipped_file_to_string(&reference_file) {
            Ok(contents) => contents,
            Err(e) => {
                panic!("Failed to read reference file {reference_file}: {e}");
            }
        };

        assert_eq!(
            generated_contents, reference_contents,
            "Contents of {:?} do not match reference",
            file
        );
    }
}

/// Test that when creating output, if the file already exists it is not overwritten
#[test]
fn test_tpchgen_cli_tbl_no_overwrite() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let expected_file = temp_dir.path().join("part.tbl");

    // First run - create the file
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let original_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), 23498);

    // Run the tpchgen-cli command again with the same parameters and expect the
    // file to not be overwritten and a warning to be logged
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("already exists, skipping generation"),
        "Expected warning message not found in stderr: {}",
        stderr
    );

    let new_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), new_metadata.len());
    assert_eq!(
        original_metadata
            .modified()
            .expect("Failed to get modified time"),
        new_metadata
            .modified()
            .expect("Failed to get modified time")
    );
}

// Test that when creating output, if the file already exists it is not for parquet
#[test]
fn test_tpchgen_cli_parquet_no_overwrite() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let expected_file = temp_dir.path().join("part.parquet");

    // First run - create the file
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let original_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), 12793);

    // Run the tpchgen-cli command again with the same parameters and expect the
    // file to not be overwritten and a warning to be logged
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("already exists, skipping generation"),
        "Expected warning message not found in stderr: {}",
        stderr
    );

    let new_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), new_metadata.len());
    assert_eq!(
        original_metadata
            .modified()
            .expect("Failed to get modified time"),
        new_metadata
            .modified()
            .expect("Failed to get modified time")
    );
}

/// Test that --quiet flag suppresses stdout output
#[test]
fn test_tpchgen_cli_quiet_flag() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let expected_file = temp_dir.path().join("part.tbl");

    // First run - create the file
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let original_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), 23498);

    // Run the tpchgen-cli command again with --quiet flag
    // Expect the file to not be overwritten and NO warning even though warnings show by default
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--quiet")
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        !stderr.contains("already exists"),
        "Expected no warning message in stderr with --quiet flag, but found: {}",
        stderr
    );

    // Verify file was not overwritten
    let new_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), new_metadata.len());
    assert_eq!(
        original_metadata
            .modified()
            .expect("Failed to get modified time"),
        new_metadata
            .modified()
            .expect("Failed to get modified time")
    );
}

/// Test generating the order table using 4 parts implicitly
#[test]
fn test_tpchgen_cli_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // generate 4 parts of the orders table with scale factor 0.001 and let
    // tpchgen-cli generate the multiple files

    let num_parts = 4;
    let output_dir = temp_dir.path().to_path_buf();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--parts")
        .arg(num_parts.to_string())
        .arg("--tables")
        .arg("orders")
        .assert()
        .success();

    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
}

/// Test generating the order table with multiple invocations using --parts and
/// --part options
#[test]
fn test_tpchgen_cli_parts_explicit() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // generate 4 parts of the orders table with scale factor 0.001
    // use threads to run the command concurrently to minimize the time taken
    let num_parts = 4;
    let mut threads = vec![];
    for part in 1..=num_parts {
        let output_dir = temp_dir.path().to_path_buf();
        threads.push(std::thread::spawn(move || {
            // Run the tpchgen-cli command for each part
            // output goes into `output_dir/orders/orders.{part}.tbl`
            cargo_bin_cmd!("tpcgen-cli")
                .arg("tpch")
                .arg("--scale-factor")
                .arg("0.001")
                .arg("--output-dir")
                .arg(&output_dir)
                .arg("--parts")
                .arg(num_parts.to_string())
                .arg("--part")
                .arg(part.to_string())
                .arg("--tables")
                .arg("orders")
                .assert()
                .success();
        }));
    }
    // Wait for all threads to finish
    for thread in threads {
        thread.join().expect("Thread panicked");
    }
    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
}

/// Create all tables using --parts option and verify the output layouts
#[test]
fn test_tpchgen_cli_parts_all_tables() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let num_parts = 8;
    let output_dir = temp_dir.path().to_path_buf();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--parts")
        .arg(num_parts.to_string())
        .assert()
        .success();

    verify_table(temp_dir.path(), "lineitem", num_parts, "0.001");
    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
    verify_table(temp_dir.path(), "part", num_parts, "0.001");
    verify_table(temp_dir.path(), "partsupp", num_parts, "0.001");
    verify_table(temp_dir.path(), "customer", num_parts, "0.001");
    verify_table(temp_dir.path(), "supplier", num_parts, "0.001");
    // Note, nation and region have only a single part regardless of --parts
    verify_table(temp_dir.path(), "nation", 1, "0.001");
    verify_table(temp_dir.path(), "region", 1, "0.001");
}

/// Read the N files from `output_dir/table_name/table_name.part.tml` into a
/// single buffer and compare them to the contents of the reference file
fn verify_table(output_dir: &Path, table_name: &str, parts: usize, scale_factor: &str) {
    let mut output_contents = Vec::new();
    for part in 1..=parts {
        let generated_file = output_dir
            .join(table_name)
            .join(format!("{table_name}.{part}.tbl"));
        assert!(
            generated_file.exists(),
            "File {:?} does not exist",
            generated_file
        );
        let generated_contents =
            fs::read_to_string(generated_file).expect("Failed to read generated file");
        output_contents.append(&mut generated_contents.into_bytes());
    }
    let output_contents =
        String::from_utf8(output_contents).expect("Failed to convert output contents to string");

    // load the reference file
    let reference_file = read_reference_file(table_name, scale_factor);
    assert_eq!(output_contents, reference_file);
}

#[tokio::test]
async fn test_write_parquet_orders() {
    // Run the CLI command to generate parquet data
    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("orders.parquet");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--tables")
        .arg("orders")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    let batch_size = 4000;

    // Create the reference Arrow data using OrderArrow
    let generator = OrderGenerator::new(0.001, 1, 1);
    let mut arrow_generator = OrderArrow::new(generator).with_batch_size(batch_size);

    // Read the generated parquet file
    let file = File::open(&output_path).expect("Failed to open parquet file");
    let options = ArrowReaderOptions::new().with_schema(arrow_generator.schema());

    let reader = ParquetRecordBatchReaderBuilder::try_new_with_options(file, options)
        .expect("Failed to create ParquetRecordBatchReaderBuilder")
        .with_batch_size(batch_size)
        .build()
        .expect("Failed to build ParquetRecordBatchReader");

    // Compare the record batches
    for batch in reader {
        let parquet_batch = batch.expect("Failed to read record batch from parquet");
        let arrow_batch = arrow_generator
            .next()
            .expect("Failed to generate record batch from OrderArrow");
        let arrow_batch = arrow_batch.expect("Arrow generation should not fail");
        assert_eq!(
            parquet_batch, arrow_batch,
            "Mismatch between parquet and arrow record batches"
        );
    }
}

#[tokio::test]
async fn test_write_parquet_row_group_size_default() {
    // Run the CLI command to generate parquet data with default settings
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    expect_row_group_sizes(
        output_dir.path(),
        vec![
            RowGroups {
                table: "customer",
                row_group_bytes: vec![6522719, 6507058, 6507800, 6515798],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    7157554, 7106900, 7090842, 7120906, 7145325, 7120319, 7142364, 7099258,
                    7111326, 7107355, 7107174, 7140691, 7103258, 7098064, 7140780, 7114738,
                    7145231, 7112989, 7107260, 7094419, 7109164, 7153132, 7106588, 7107901,
                    7145001, 7101142, 7110720, 7127039, 7118498, 7158328, 7122729, 7135124,
                    7115110, 7113817, 7118599, 7096420, 7129813, 7124217, 7116502, 7105980,
                    7124396, 7143315, 7102503, 7130464, 7101232, 7101367, 7139904, 7108710,
                    7091458, 7093976, 7158507, 7157452, 7132894,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2684],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![
                    7842293, 7841931, 7847396, 7844507, 7849243, 7847495, 7838444, 7841044,
                    7840217, 7837271, 7841056, 7839265, 7843712, 7834117, 7839886, 7838091,
                ],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![7012918, 7014223],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![
                    7292900, 7275703, 7290373, 7286175, 7284159, 7291041, 7278512, 7298320,
                    7283253, 7289609, 7285376, 7295104, 7290407, 7293930, 7287756, 7278354,
                ],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![554],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1636998],
            },
        ],
    );
}

#[tokio::test]
async fn test_write_parquet_row_group_size_20mb() {
    // Run the CLI command to generate parquet data with larger row group size
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--output-dir")
        .arg(output_dir.path())
        .arg("--row-group-bytes")
        .arg("20000000") // 20 MB
        .assert()
        .success();

    expect_row_group_sizes(
        output_dir.path(),
        vec![
            RowGroups {
                table: "customer",
                row_group_bytes: vec![12844748, 12838467],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    18114785, 18167648, 18114968, 18092636, 18098372, 18153536, 18137038, 18081920,
                    18110927, 18140643, 18131304, 18186767, 18103994, 18101890, 18131440, 18120528,
                    18119019, 18114395, 18107484, 18171954,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2684],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![19815261, 19819445, 19810193, 19806532, 19802204, 19795267],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![13919709],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![18978072, 18990959, 18973658, 18976682, 18995233, 18981274],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![554],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1636998],
            },
        ],
    );
}

#[test]
fn test_tpchgen_cli_part_no_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // CLI Error test --part and but not --parts
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("42")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "The --part option requires the --parts option to be set",
        ));
}

#[test]
fn test_tpchgen_cli_too_many_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // This should fail because --part is 42 which is more than the --parts 10
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("42")
        .arg("--parts")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected at most the value of --parts (10), got 42",
        ));
}

#[test]
fn test_tpchgen_cli_zero_part() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("0")
        .arg("--parts")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected a number greater than zero, got 0",
        ));
}
#[test]
fn test_tpchgen_cli_zero_part_zero_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("0")
        .arg("--parts")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected a number greater than zero, got 0",
        ));
}

/// Test that --num-threads=0 is rejected at argument parse time
#[test]
fn test_tpchgen_cli_rejects_zero_num_threads() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--num-threads")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "error: invalid value '0' for '--num-threads <NUM_THREADS>'",
        ));
}

/// Test specifying parquet options even when writing tbl output
#[tokio::test]
async fn test_incompatible_options_warnings() {
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("csv")
        .arg("--tables")
        .arg("orders")
        .arg("--scale-factor")
        .arg("0.0001")
        .arg("--output-dir")
        .arg(output_dir.path())
        // pass in parquet options that are incompatible with csv
        .arg("--parquet-compression")
        .arg("zstd(1)")
        .arg("--parquet-row-group-bytes")
        .arg("8192")
        .assert()
        // still success, but should see warnings in stderr
        .success()
        .stderr(predicates::str::contains(
            "--parquet-compression ignored: output format is not parquet",
        ))
        .stderr(predicates::str::contains(
            "--parquet-row-group-bytes ignored: output format is not parquet",
        ));
}

/// Test that --quiet flag suppresses warning messages
#[tokio::test]
async fn test_quiet_flag_suppresses_warnings() {
    let output_dir = tempdir().unwrap();
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .env("RUST_LOG", "warn")
        .arg("--format")
        .arg("csv")
        .arg("--tables")
        .arg("orders")
        .arg("--scale-factor")
        .arg("0.0001")
        .arg("--output-dir")
        .arg(output_dir.path())
        // pass in parquet options that are incompatible with csv
        .arg("--parquet-compression")
        .arg("zstd(1)")
        .arg("--parquet-row-group-bytes")
        .arg("8192")
        .arg("--quiet")
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        !stderr.contains("Parquet"),
        "Expected no warning messages in stderr with --quiet flag, but found: {}",
        stderr
    );
}

/// Test that --no-progress is accepted and produces no progress bar output.
/// Note: in `assert_cmd`-driven tests stderr is not a TTY so progress is also
/// auto-disabled; this test mainly locks in the flag's existence and verifies
/// no progress glyphs leak into the output.
#[test]
fn test_tpchgen_cli_no_progress_flag() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--no-progress")
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    for glyph in ["█", "▓", "░", "Progress:"] {
        assert!(
            !stderr.contains(glyph),
            "Expected no progress bar glyph {glyph:?} in stderr, but found: {stderr}"
        );
    }
}

/// Test that the progress bar is auto-suppressed when stderr is not a TTY
/// (as is the case under `assert_cmd`, CI logs, and pipe redirection),
/// even without passing `--no-progress`. This locks in the contract that
/// CI logs are never polluted with progress glyphs by default.
#[test]
fn test_tpchgen_cli_progress_auto_disabled_on_non_tty() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let output = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    for glyph in ["█", "▓", "░", "Progress:"] {
        assert!(
            !stderr.contains(glyph),
            "Expected progress to be auto-disabled on non-TTY stderr, but found {glyph:?} in: {stderr}"
        );
    }
}

fn read_gzipped_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;
    let contents = String::from_utf8(contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(contents)
}

/// Reads the reference file for the specified table and scale factor.
///
/// example usage: `read_reference_file("orders", "0.001")`
fn read_reference_file(table_name: &str, scale_factor: &str) -> String {
    let reference_file = format!("../tpchgen/data/sf-{scale_factor}/{table_name}.tbl.gz");
    match read_gzipped_file_to_string(&reference_file) {
        Ok(contents) => contents,
        Err(e) => {
            panic!("Failed to read reference file {reference_file}: {e}");
        }
    }
}

/// Test that --format=parquet emits a warning about v4.0.0 migration
#[tokio::test]
async fn test_format_parquet_warns_about_subcommand() {
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("parquet")
        .arg("--tables")
        .arg("part")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("will be removed in v4.0.0"));
}

/// Test that using --format together with a subcommand errors
#[test]
fn test_format_with_subcommand_conflict() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("parquet")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));
}

/// Test that using --parquet-compression together with a subcommand errors
#[test]
fn test_parquet_compression_with_subcommand_conflict() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // With parquet subcommand
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--parquet-compression")
        .arg("SNAPPY")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));

    // With tbl subcommand
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--parquet-compression")
        .arg("SNAPPY")
        .arg("tbl")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));
}

/// Test that using --parquet-row-group-bytes together with a subcommand errors
#[test]
fn test_parquet_row_group_bytes_with_subcommand_conflict() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // With parquet subcommand
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--parquet-row-group-bytes")
        .arg("1000000")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));

    // With csv subcommand
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--parquet-row-group-bytes")
        .arg("1000000")
        .arg("csv")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));
}

/// Test that common args before a subcommand are rejected
#[test]
fn test_common_args_with_subcommand_conflict() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // -s before subcommand should error
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("-s")
        .arg("0.01")
        .arg("parquet")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));

    // -s after subcommand should work
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("parquet")
        .arg("-s")
        .arg("0.01")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();
}

/// Test that running with no --format and no subcommand defaults to TBL
#[test]
fn test_default_format_is_tbl() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_file = temp_dir.path().join("part.tbl");
    assert!(
        expected_file.exists(),
        "Expected TBL file {:?} to exist when no --format or subcommand is specified",
        expected_file
    );
}

/// Test that the `tbl` subcommand generates TBL files
#[test]
fn test_tbl_subcommand() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("tbl")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_file = temp_dir.path().join("part.tbl");
    assert!(
        expected_file.exists(),
        "Expected TBL file {:?} to exist with `tbl` subcommand",
        expected_file
    );
}

/// Test that the `csv` subcommand generates CSV files
#[test]
fn test_csv_subcommand() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("csv")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_file = temp_dir.path().join("part.csv");
    assert!(
        expected_file.exists(),
        "Expected CSV file {:?} to exist with `csv` subcommand",
        expected_file
    );
}

/// Test that --format=csv emits a deprecation warning
#[tokio::test]
async fn test_format_csv_warns_about_subcommand() {
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("csv")
        .arg("--tables")
        .arg("part")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("will be removed in v4.0.0"));

    let expected_file = output_dir.path().join("part.csv");
    assert!(
        expected_file.exists(),
        "Expected CSV file {:?} to exist with deprecated --format=csv path",
        expected_file
    );
}

/// Test that --format=tbl emits a deprecation warning
#[tokio::test]
async fn test_format_tbl_warns_about_subcommand() {
    let output_dir = tempdir().unwrap();
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("tbl")
        .arg("--tables")
        .arg("part")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("will be removed in v4.0.0"));
}

/// Test that the `csv` subcommand with a custom delimiter produces tab-delimited output
#[test]
fn test_csv_subcommand_custom_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("csv")
        .arg("--delimiter")
        .arg("\\t")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let csv_file = temp_dir.path().join("region.csv");
    assert!(
        csv_file.exists(),
        "Expected CSV file {:?} to exist",
        csv_file
    );

    let contents = std::fs::read_to_string(&csv_file).unwrap();
    // Region table has 5 rows; each should contain tabs as delimiters
    assert!(
        contents.contains('\t'),
        "Expected tab-delimited output, got:\n{}",
        contents
    );
    // Verify multiple tab-separated fields per line
    let first_line = contents.lines().next().unwrap();
    let tab_count = first_line.matches('\t').count();
    assert!(
        tab_count >= 2,
        "Expected at least 2 tabs per line, got {} in: {}",
        tab_count,
        first_line
    );
}

/// Test that the `csv` subcommand rejects a non-ASCII delimiter at parse time
#[test]
fn test_csv_subcommand_rejects_non_ascii_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("csv")
        .arg("--delimiter")
        .arg("€")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("region")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("ASCII"));
}

/// Test that the `tbl` subcommand rejects --delimiter
#[test]
fn test_tbl_subcommand_rejects_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("tbl")
        .arg("--delimiter")
        .arg(",")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("unexpected argument"));
}

/// Test that deprecated --format=parquet with --parquet-compression still works
#[tokio::test]
async fn test_deprecated_parquet_compression_flag_works() {
    let output_dir = tempdir().unwrap();

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("parquet")
        .arg("--parquet-compression")
        .arg("ZSTD(1)")
        .arg("--tables")
        .arg("region")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "--parquet-compression flag is deprecated",
        ));

    let parquet_file = output_dir.path().join("region.parquet");
    assert!(
        parquet_file.exists(),
        "Expected Parquet file {:?} to exist",
        parquet_file
    );
}

/// Test that deprecated --format=parquet with --parquet-row-group-bytes still works
#[tokio::test]
async fn test_deprecated_parquet_row_group_bytes_flag_works() {
    let output_dir = tempdir().unwrap();

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpch")
        .arg("--format")
        .arg("parquet")
        .arg("--parquet-row-group-bytes")
        .arg("1000000")
        .arg("--tables")
        .arg("region")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "--parquet-row-group-bytes flag is deprecated",
        ));

    let parquet_file = output_dir.path().join("region.parquet");
    assert!(
        parquet_file.exists(),
        "Expected Parquet file {:?} to exist",
        parquet_file
    );
}
