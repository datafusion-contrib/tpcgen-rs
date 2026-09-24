use super::test_helpers::{
    assert_flags_under_help_heading, assert_stdout_matches_file_output, expect_column_encoding,
    expect_row_group_sizes, RowGroups,
};
use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use arrow::datatypes::{DataType, TimeUnit};
use arrow::record_batch::RecordBatchReader;
use assert_cmd::cargo::cargo_bin_cmd;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::{Compression, Encoding};
use parquet::file::metadata::ParquetMetaDataReader;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use tpcdsgen::config::{Session, SessionBuilder, Table};
use tpcdsgen_arrow::{ItemArrow, StoreReturnsArrow, StoreSalesArrow};

/// Test that TPC-DS DAT generation is quiet unless logging is explicitly enabled.
#[test]
fn test_tpcgen_cli_tpcds_dat_is_quiet_by_default() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .env_remove("RUST_LOG")
        .assert()
        .success();

    assert!(
        assert.get_output().stdout.is_empty(),
        "Expected TPC-DS DAT generation to write no stdout by default, got: {}",
        String::from_utf8_lossy(&assert.get_output().stdout)
    );
    assert!(
        assert.get_output().stderr.is_empty(),
        "Expected TPC-DS DAT generation to write no stderr by default, got: {}",
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
}

/// Test that TPC-DS DAT verbose mode enables status logging on stderr.
#[test]
fn test_tpcgen_cli_tpcds_dat_verbose_enables_status_logging() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("-v")
        .env("RUST_LOG", "warn")
        .assert()
        .success();

    assert!(
        assert.get_output().stdout.is_empty(),
        "Expected verbose TPC-DS DAT logging to use stderr, got stdout: {}",
        String::from_utf8_lossy(&assert.get_output().stdout)
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("Verbose output enabled (ignoring RUST_LOG environment variable)"),
        "Expected verbose mode setup log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("Writing") && stderr.contains("reason.dat using"),
        "Expected TPC-DS table start log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("Generated") && stderr.contains("reason.dat"),
        "Expected TPC-DS table completion log, got stderr: {stderr}"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_single_table() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_file = temp_dir.path().join("reason.parquet");
    assert!(expected_file.exists());

    let file = File::open(&expected_file).expect("Failed to open Parquet file");
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(file).expect("Failed to read Parquet metadata");
    assert_eq!(builder.schema().fields().len(), 3);

    let row_count = builder
        .build()
        .expect("Failed to build Parquet reader")
        .map(|batch| batch.expect("Failed to read Parquet batch").num_rows())
        .sum::<usize>();
    assert_eq!(row_count, 35);
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_verbose_enables_logging() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("-v")
        .env("RUST_LOG", "warn")
        .assert()
        .success();

    assert!(
        assert.get_output().stdout.is_empty(),
        "Expected verbose TPC-DS Parquet logging to use stderr, got stdout: {}",
        String::from_utf8_lossy(&assert.get_output().stdout)
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("Verbose output enabled (ignoring RUST_LOG environment variable)"),
        "Expected verbose mode setup log, got stderr: {stderr}"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_files: BTreeSet<_> = [
        "call_center.parquet",
        "catalog_page.parquet",
        "catalog_returns.parquet",
        "catalog_sales.parquet",
        "customer.parquet",
        "customer_address.parquet",
        "customer_demographics.parquet",
        "date_dim.parquet",
        "dbgen_version.parquet",
        "household_demographics.parquet",
        "income_band.parquet",
        "inventory.parquet",
        "item.parquet",
        "promotion.parquet",
        "reason.parquet",
        "ship_mode.parquet",
        "store.parquet",
        "store_returns.parquet",
        "store_sales.parquet",
        "time_dim.parquet",
        "warehouse.parquet",
        "web_page.parquet",
        "web_returns.parquet",
        "web_sales.parquet",
        "web_site.parquet",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let actual_files = fs::read_dir(temp_dir.path())
        .expect("Failed to read generated output directory")
        .map(|entry| {
            entry
                .expect("Failed to read generated output directory entry")
                .file_name()
                .into_string()
                .expect("Generated output file name is not valid UTF-8")
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_files, expected_files,
        "Expected default TPC-DS Parquet generation to produce every main table"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_compression() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--compression")
        .arg("UNCOMPRESSED")
        .assert()
        .success();

    let expected_file = temp_dir.path().join("reason.parquet");
    let file = File::open(&expected_file).expect("Failed to open Parquet file");
    let mut metadata_reader = ParquetMetaDataReader::new();
    metadata_reader.try_parse(&file).unwrap();
    let metadata = metadata_reader.finish().unwrap();

    for row_group in metadata.row_groups() {
        for column in row_group.columns() {
            assert_eq!(column.compression(), Compression::UNCOMPRESSED);
        }
    }
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_column_encoding() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("r_reason_desc=DELTA_LENGTH_BYTE_ARRAY")
        .assert()
        .success();

    let path = temp_dir.path().join("reason.parquet");
    expect_column_encoding(&path, "r_reason_desc", Encoding::DELTA_LENGTH_BYTE_ARRAY);
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_rejects_invalid_column_encoding() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("r_reason_desc=NOT_AN_ENCODING")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("invalid value") && stderr.contains("--column-encoding"),
        "unexpected stderr: {stderr}"
    );
}

/// A `--column-encoding` column that exists on only some selected tables
/// applies there and is skipped elsewhere. Selecting tables that do not
/// share every named column is not an error.
#[test]
fn test_tpcgen_cli_tpcds_parquet_column_encoding_applies_only_where_the_column_exists() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // r_reason_desc only exists on reason, not item.
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("reason,item")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("r_reason_desc=DELTA_LENGTH_BYTE_ARRAY")
        .assert()
        .success();

    let reason_path = temp_dir.path().join("reason.parquet");
    expect_column_encoding(
        &reason_path,
        "r_reason_desc",
        Encoding::DELTA_LENGTH_BYTE_ARRAY,
    );
    assert!(
        temp_dir.path().join("item.parquet").exists(),
        "expected item.parquet to still be generated, just without r_reason_desc applied to it"
    );
}

/// A `--column-encoding` column that matches no selected table (a typo)
/// must fail before any table is written.
#[test]
fn test_tpcgen_cli_tpcds_parquet_column_encoding_typo_fails_before_any_output() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("reason,item")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("r_reason_desc_typo=DELTA_LENGTH_BYTE_ARRAY")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("column 'r_reason_desc_typo'"),
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
fn test_tpcgen_cli_tpcds_parquet_dictionary_encoding_fails_before_any_output() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // r_reason_desc only exists on reason. This must still fail up
    // front, before either table is scheduled.
    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.01")
        .arg("--tables")
        .arg("reason,item")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--column-encoding")
        .arg("r_reason_desc=PLAIN_DICTIONARY")
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

#[test]
fn test_tpcgen_cli_tpcds_parquet_row_group_size_1mb() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("customer")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--row-group-bytes")
        .arg("1000000")
        .assert()
        .success();

    expect_row_group_sizes(
        temp_dir.path(),
        vec![RowGroups {
            table: "customer",
            row_group_bytes: vec![
                1154938, 1153793, 1152175, 1152992, 1152614, 1152066, 1153473, 1152964,
            ],
        }],
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_rejects_zero_row_group_bytes() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--row-group-bytes")
        .arg("0")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert_eq!(
        stderr,
        "error: invalid value '0' for '--row-group-bytes <ROW_GROUP_BYTES>': must be greater than zero\n\nFor more information, try '--help'.\n"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_rejects_zero_num_threads() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--num-threads")
        .arg("0")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("error: invalid value '0' for '--num-threads <NUM_THREADS>'"),
        "Expected --num-threads=0 to be rejected at argument parse time, got stderr: {stderr}"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_unknown_table_error_lists_valid_tables() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("unknown table 'part'. Expected one of: call_center, catalog_page, catalog_returns, catalog_sales, customer, customer_address, customer_demographics, date_dim, household_demographics, income_band, inventory, item, promotion, reason, ship_mode, store, store_returns, store_sales, time_dim, warehouse, web_page, web_returns, web_sales, web_site, dbgen_version"),
        "Expected unknown table error to list valid TPC-DS tables, got stderr: {stderr}"
    );
}

/// Test multiple TPC-DS table selection and the default DAT command form.
#[test]
fn test_tpcgen_cli_tpcds_dat_multiple_table_selection_command_forms() {
    let forms: &[&[&str]] = &[&["tpcds"], &["tpcds", "dat"]];

    for form in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen-cli")
            .args(*form)
            .arg("--scale-factor")
            .arg("0")
            .arg("--tables")
            .arg("reason,ship_mode")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success();

        assert!(temp_dir.path().join("reason.dat").exists());
        assert!(temp_dir.path().join("ship_mode.dat").exists());
        assert_eq!(
            fs::read_dir(temp_dir.path())
                .expect("Failed to read generated output directory")
                .count(),
            2,
            "Expected `tpcgen-cli {}` to produce the selected table output set",
            form.join(" ")
        );
    }
}

/// Repeated selections and both sides of a sales/returns pair should schedule
/// each row generator exactly once for DAT and CSV.
#[test]
fn test_tpcgen_cli_tpcds_row_outputs_deduplicate_selected_tables() {
    let table_orders = [
        "reason,reason,reason,\
         store_sales,store_returns,store_sales,\
         catalog_sales,catalog_returns,catalog_sales,\
         web_sales,web_returns,web_sales",
        "reason,reason,reason,\
         store_returns,store_sales,store_returns,\
         catalog_returns,catalog_sales,catalog_returns,\
         web_returns,web_sales,web_returns",
    ];

    for tables in table_orders {
        for (format, extension) in [("dat", "dat"), ("csv", "csv")] {
            let temp_dir = tempdir().expect("Failed to create temporary directory");

            let assert = cargo_bin_cmd!("tpcgen-cli")
                .arg("tpcds")
                .arg(format)
                .arg("--scale-factor")
                .arg("0")
                .arg("--tables")
                .arg(tables)
                .arg("--output-dir")
                .arg(temp_dir.path())
                .arg("--verbose")
                .assert()
                .success();

            let expected_files = [
                format!("reason.{extension}"),
                format!("store_sales.{extension}"),
                format!("store_returns.{extension}"),
                format!("catalog_sales.{extension}"),
                format!("catalog_returns.{extension}"),
                format!("web_sales.{extension}"),
                format!("web_returns.{extension}"),
            ]
            .into_iter()
            .collect::<BTreeSet<_>>();
            let actual_files = fs::read_dir(temp_dir.path())
                .expect("Failed to read generated output directory")
                .map(|entry| {
                    entry
                        .expect("Failed to read generated output directory entry")
                        .file_name()
                        .into_string()
                        .expect("Generated output file name is not valid UTF-8")
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(actual_files, expected_files);

            // Each selected table is generated exactly once, however many
            // times it was named on the command line. A sales table and its
            // returns table are separate outputs, so both appear.
            let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
            for file in &expected_files {
                let generated = stderr
                    .lines()
                    .filter(|line| line.contains("Generated ") && line.ends_with(file))
                    .count();
                assert_eq!(
                    generated, 1,
                    "Expected {file} to be generated once for {format} with {tables}, got stderr: {stderr}"
                );
            }
        }
    }
}

fn generate_parquet_files(table_args: &[String]) -> BTreeSet<String> {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let mut command = cargo_bin_cmd!("tpcgen-cli");
    command
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0");
    for tables in table_args {
        command.arg("--tables").arg(tables);
    }
    command
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    fs::read_dir(temp_dir.path())
        .expect("Failed to read generated output directory")
        .map(|entry| {
            entry
                .expect("Failed to read generated output directory entry")
                .file_name()
                .into_string()
                .expect("Generated output file name is not valid UTF-8")
        })
        .collect()
}

/// Parquet receives the same exact-value deduplication as other formats,
/// including when values are supplied through repeated `--tables` flags.
#[test]
fn test_tpcgen_cli_tpcds_parquet_deduplicates_repeated_table_selection() {
    let expected = BTreeSet::from(["reason.parquet".to_string()]);
    for table_args in [
        vec!["reason,reason,reason".to_string()],
        vec![
            "reason".to_string(),
            "reason".to_string(),
            "reason".to_string(),
        ],
    ] {
        assert_eq!(generate_parquet_files(&table_args), expected);
    }
}

/// Parquet keeps sales and returns as distinct output selections while still
/// deduplicating repeated occurrences of either table.
#[test]
fn test_tpcgen_cli_tpcds_parquet_preserves_sales_returns_selection_semantics() {
    for (sales, returns) in [
        (Table::CatalogSales, Table::CatalogReturns),
        (Table::StoreSales, Table::StoreReturns),
        (Table::WebSales, Table::WebReturns),
    ] {
        let sales = sales.get_name();
        let returns = returns.get_name();

        assert_eq!(
            generate_parquet_files(&[sales.to_string()]),
            BTreeSet::from([format!("{sales}.parquet")])
        );
        assert_eq!(
            generate_parquet_files(&[returns.to_string()]),
            BTreeSet::from([format!("{returns}.parquet")])
        );

        for tables in [
            format!("{sales},{returns},{sales}"),
            format!("{returns},{sales},{returns}"),
        ] {
            assert_eq!(
                generate_parquet_files(&[tables]),
                BTreeSet::from([format!("{sales}.parquet"), format!("{returns}.parquet"),])
            );
        }
    }
}

/// Test each TPC-DS DAT table can be selected individually and creates output.
#[test]
fn test_tpcgen_cli_tpcds_dat_individual_table_selection_outputs_requested_table() {
    // The CLI accepts only main tables; source/internal tables are rejected by parse_table.
    for table in Table::main_tables() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen-cli")
            .arg("tpcds")
            .arg("dat")
            .arg("--scale-factor")
            .arg("0")
            .arg("--tables")
            .arg(table.get_name())
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success();

        let expected_file = temp_dir.path().join(format!("{}.dat", table.get_name()));
        assert!(
            expected_file.exists(),
            "Expected selecting {table} to create {:?}",
            expected_file
        );
    }
}

/// Test that TPC-DS DAT generation forwards compatibility mode to tpcdsgen.
#[test]
fn test_tpcgen_cli_tpcds_dat_compat_mode() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--compat")
        .arg("c")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.dat")).expect("Failed to read DAT file");
    assert_eq!(
        contents.lines().count(),
        75,
        "Expected C compatibility mode to use C dsdgen reason table cardinality"
    );
}

/// Test that TPC-DS DAT generation forwards the actual command line to dbgen_version.
#[test]
fn test_tpcgen_tpcds_dat_dbgen_version_command_line() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("dbgen_version")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents = fs::read_to_string(temp_dir.path().join("dbgen_version.dat"))
        .expect("Failed to read DAT file");
    let fields: Vec<_> = contents
        .trim_end()
        .trim_end_matches('|')
        .split('|')
        .collect();
    assert_eq!(fields.len(), 4);
    assert!(
        fields[3].contains("tpcds dat --scale-factor 1 --tables dbgen_version --output-dir"),
        "Expected dbgen_version command line to contain the actual TPC-DS invocation, got: {}",
        fields[3]
    );
}

/// Test that default DAT output options generate every main TPC-DS output file.
///
/// This overrides only scale factor and output directory: scale factor 0 keeps
/// the integration test fast, while output directory isolates generated files.
#[test]
fn test_tpcgen_cli_tpcds_dat_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("catalog_sales.dat").exists());
    assert!(temp_dir.path().join("catalog_returns.dat").exists());
    assert!(temp_dir.path().join("reason.dat").exists());
    assert_eq!(
        fs::read_dir(temp_dir.path())
            .expect("Failed to read generated output directory")
            .count(),
        25,
        "Expected default TPC-DS DAT generation to produce every main table"
    );
}

/// Test that TPC-DS CSV generation writes a headered CSV file for one table.
#[test]
fn test_tpcgen_cli_tpcds_csv_single_table() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let csv_file = temp_dir.path().join("reason.csv");
    assert!(csv_file.exists(), "Expected {:?} to exist", csv_file);

    let contents = fs::read_to_string(&csv_file).expect("Failed to read CSV file");
    let lines: Vec<_> = contents.lines().collect();
    assert_eq!(
        lines.first(),
        Some(&"r_reason_sk,r_reason_id,r_reason_desc")
    );
    assert_eq!(
        lines.len(),
        36,
        "Expected CSV header plus 35 reason rows at scale factor 1"
    );
    assert!(
        lines.iter().all(|line| !line.ends_with(',')),
        "Expected CSV rows not to end with a trailing delimiter, got:\n{contents}"
    );
}

/// Test that TPC-DS CSV generation supports a custom delimiter.
#[test]
fn test_tpcgen_cli_tpcds_csv_custom_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("\\t")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.csv")).expect("Failed to read CSV file");
    let first_line = contents.lines().next().expect("CSV output is empty");
    assert_eq!(first_line, "r_reason_sk\tr_reason_id\tr_reason_desc");
    assert!(
        !first_line.contains(','),
        "Expected custom-delimited CSV header not to use commas: {first_line}"
    );
    assert_eq!(
        first_line.matches('\t').count(),
        2,
        "Expected exactly two tab delimiters in the reason header"
    );
}

/// Test that TPC-DS CSV generation escapes headers containing the delimiter.
#[test]
fn test_tpcgen_cli_tpcds_csv_delimiter_in_header_is_escaped() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("_")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.csv")).expect("Failed to read CSV file");
    let first_line = contents.lines().next().expect("CSV output is empty");
    let second_line = contents.lines().nth(1).expect("CSV data row is missing");
    assert_eq!(
        first_line,
        "\"r_reason_sk\"_\"r_reason_id\"_\"r_reason_desc\""
    );
    assert_eq!(
        second_line.split('_').count(),
        3,
        "Expected underscore-delimited data rows to have three fields: {second_line}"
    );
}

/// Test that default CSV output options generate every main TPC-DS output file.
#[test]
fn test_tpcgen_cli_tpcds_csv_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_files: BTreeSet<_> = [
        "call_center.csv",
        "catalog_page.csv",
        "catalog_returns.csv",
        "catalog_sales.csv",
        "customer.csv",
        "customer_address.csv",
        "customer_demographics.csv",
        "date_dim.csv",
        "dbgen_version.csv",
        "household_demographics.csv",
        "income_band.csv",
        "inventory.csv",
        "item.csv",
        "promotion.csv",
        "reason.csv",
        "ship_mode.csv",
        "store.csv",
        "store_returns.csv",
        "store_sales.csv",
        "time_dim.csv",
        "warehouse.csv",
        "web_page.csv",
        "web_returns.csv",
        "web_sales.csv",
        "web_site.csv",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let actual_files = fs::read_dir(temp_dir.path())
        .expect("Failed to read generated output directory")
        .map(|entry| {
            entry
                .expect("Failed to read generated output directory entry")
                .file_name()
                .into_string()
                .expect("Generated output file name is not valid UTF-8")
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_files, expected_files,
        "Expected default TPC-DS CSV generation to produce every main table"
    );
}

/// Test that the TPC-DS CSV subcommand rejects a non-ASCII delimiter at parse time.
#[test]
fn test_tpcgen_cli_tpcds_csv_rejects_non_ascii_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("€")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("ASCII"));
}

/// Session matching the CLI defaults for the given scale factor.
fn test_session(scale_factor: f64) -> Session {
    SessionBuilder::new()
        .with_scale_factor(scale_factor)
        .build()
        .expect("valid session")
}

/// Read a parquet file into a single [`RecordBatch`], also returning the
/// number of row groups in the file.
fn read_concatenated_parquet(path: &Path) -> (RecordBatch, usize) {
    let file = File::open(path).expect("Failed to open Parquet file");
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(file).expect("Failed to read Parquet metadata");
    let num_row_groups = builder.metadata().num_row_groups();
    let schema = builder.schema().clone();
    let batches: Vec<RecordBatch> = builder
        .build()
        .expect("Failed to build Parquet reader")
        .map(|batch| batch.expect("Failed to read Parquet batch"))
        .collect();
    let batch = concat_batches(&schema, &batches).expect("Failed to concatenate batches");
    (batch, num_row_groups)
}

/// Drain a [`RecordBatchReader`] into a single [`RecordBatch`].
fn read_concatenated_reference<R: RecordBatchReader>(mut reader: R) -> RecordBatch {
    let schema = reader.schema();
    let batches: Vec<RecordBatch> = reader
        .by_ref()
        .map(|batch| batch.expect("Failed to generate reference batch"))
        .collect();
    concat_batches(&schema, &batches).expect("Failed to concatenate reference batches")
}

/// Parquet files are generated using multiple source row ranges. Each
/// Row Group comes from a particular row range, potentially encoded in parallel.
///
/// This test ensures that the result of this row range generation is the same
/// as generating the data in a single chunk.
///
/// store_returns is generated from the store_sales generator, so this also
/// verifies that ranging over the *sales* source rows loses or duplicates no
/// return rows at range boundaries.
///
/// Item is an SCD table, so this also verifies that range boundaries preserve
/// the previous revision state needed by continuation rows.
#[test]
fn test_tpcgen_cli_tpcds_parquet_matches_single_pass_generation() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // write parquet data using CLI
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("store_sales,store_returns,item")
        // small row groups to force several source row ranges
        .arg("--row-group-bytes")
        .arg("250KB")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    // Parquet data
    let (store_sales, num_row_groups) =
        read_concatenated_parquet(&temp_dir.path().join("store_sales.parquet"));
    assert_eq!(num_row_groups, 24);
    let expected = read_concatenated_reference(StoreSalesArrow::new(test_session(0.001)));
    assert_eq!(store_sales, expected);

    // regenerate same data directly from arrow generator
    let (store_returns, num_row_groups) =
        read_concatenated_parquet(&temp_dir.path().join("store_returns.parquet"));
    assert_eq!(num_row_groups, 3);
    let expected = read_concatenated_reference(StoreReturnsArrow::new(test_session(0.001)));
    assert_eq!(store_returns, expected);

    let (item, num_row_groups) = read_concatenated_parquet(&temp_dir.path().join("item.parquet"));
    // 2,000 source rows over 2 row groups starts the second range at row 1,001,
    // a continuation revision that copies from row 1,000. Pin both numbers: if
    // either drifts the split can land on a row that starts a new Item, where
    // nothing is copied and the SCD case silently goes untested.
    assert_eq!(num_row_groups, 2);
    assert_eq!(item.num_rows(), 2_000);
    let expected = read_concatenated_reference(ItemArrow::new(test_session(0.001)));
    assert_eq!(item, expected);
}

/// Test that the number of threads does not change the generated files.
#[test]
fn test_tpcgen_cli_tpcds_parquet_num_threads_equivalence() {
    let mut outputs = vec![];
    for num_threads in ["1", "4"] {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen-cli")
            .arg("tpcds")
            .arg("parquet")
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg("store_sales")
            // small row groups so multiple row groups are encoded in parallel
            .arg("--row-group-bytes")
            .arg("1MB")
            .arg("--num-threads")
            .arg(num_threads)
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success();

        let path = temp_dir.path().join("store_sales.parquet");

        // verify multiple row groups were actually created, so the encoding
        // really ran in parallel with --num-threads=4
        let file = File::open(&path).expect("Failed to open Parquet file");
        let mut metadata_reader = ParquetMetaDataReader::new();
        metadata_reader.try_parse(&file).unwrap();
        let num_row_groups = metadata_reader.finish().unwrap().num_row_groups();
        assert_eq!(num_row_groups, 6);

        outputs.push(fs::read(&path).expect("Failed to read Parquet file"));
    }

    assert_eq!(
        outputs[0], outputs[1],
        "Expected --num-threads=1 and --num-threads=4 to produce identical files"
    );
}

/// Test that the Arrow schema is embedded in the Parquet metadata: the
/// dbgen_version dv_create_time column is Time32(Second), which has no exact
/// Parquet equivalent and only survives via the embedded Arrow schema.
#[test]
fn test_tpcgen_cli_tpcds_parquet_preserves_arrow_schema() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("dbgen_version")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let file = File::open(temp_dir.path().join("dbgen_version.parquet"))
        .expect("Failed to open Parquet file");
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(file).expect("Failed to read Parquet metadata");
    let field = builder
        .schema()
        .field_with_name("dv_create_time")
        .expect("dv_create_time field");
    assert_eq!(field.data_type(), &DataType::Time32(TimeUnit::Second));
}

/// Test that `--help` lists each selectable TPC-DS table.
#[test]
fn test_tpcgen_cli_tpcds_help_lists_tables() {
    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("--help")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    for table in Table::main_tables() {
        assert!(
            stdout.contains(&format!("- {}:", table.get_name())),
            "Expected `tpcds --help` to list {table}, got stdout: {stdout}"
        );
    }
}

/// Test that `--part` without `--parts` is rejected with the expected message.
#[test]
fn test_tpcgen_cli_tpcds_dat_part_without_parts_is_rejected() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("1")
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert_eq!(
        stderr,
        "Error: The --part option requires the --parts option to be set\n"
    );
}

/// Test that an invalid `--part`/`--parts` combination is rejected before any
/// output directory is created.
#[test]
fn test_tpcgen_cli_tpcds_rejects_invalid_part_without_creating_output() {
    // (--parts, --part, expected error)
    let cases = [
        (
            "3",
            "4",
            "Invalid --part value '4'. Expected at most the value of --parts (3)",
        ),
        (
            "3",
            "0",
            "Invalid --part value '0'. Expected a number greater than zero",
        ),
        (
            "0",
            "1",
            "Invalid --parts value '0'. Expected a number greater than zero",
        ),
    ];

    for format in ["dat", "csv", "parquet"] {
        for (parts, part, expected_error) in cases {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let output_dir = temp_dir.path().join("output");

            let assert = cargo_bin_cmd!("tpcgen-cli")
                .arg("tpcds")
                .arg(format)
                .arg("--scale-factor")
                .arg("0.001")
                .arg("--tables")
                .arg("reason")
                .arg("--output-dir")
                .arg(&output_dir)
                .arg("--parts")
                .arg(parts)
                .arg("--part")
                .arg(part)
                .assert()
                .failure();

            let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
            assert_eq!(
                stderr,
                format!("Error: {expected_error}\n"),
                "Unexpected error for {format} --parts {parts} --part {part}"
            );

            assert!(
                !output_dir.exists(),
                "Invalid --part must not create output: {format} --parts {parts} --part {part}"
            );
        }
    }
}

/// Test that a non-positive `--parts` is rejected.
#[test]
fn test_tpcgen_cli_tpcds_dat_rejects_non_positive_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--parts")
        .arg("0")
        .assert()
        .failure();
}

/// Test that `--parts` works for small tables.
///
/// The original `dsdgen` has a 1M-row split threshold, so for the 35-row reason
/// table `--parts 4` puts the whole table in a single chunk
#[test]
fn test_tpcgen_cli_tpcds_dat_parts_small_table_stays_in_chunk_one() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--parts")
        .arg("4")
        .assert()
        .success();

    let path = temp_dir.path().join("reason/reason.1.dat");
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("Expected {path:?} to exist: {err}"));
    assert_eq!(contents.lines().count(), 35, "chunk 1 has every row");

    for chunk in 2..=5 {
        let path = temp_dir.path().join(format!("reason/reason.{chunk}.dat"));
        assert!(
            !path.exists(),
            "chunk {chunk} at path {path:?} should not exist"
        );
    }
}

/// Test that `--parts 1` puts the output in the `parts` directory,
#[test]
fn test_tpcgen_cli_tpcds_dat_parts_outputs_directory() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--parts")
        .arg("1")
        .assert()
        .success();

    assert!(
        temp_dir.path().join("reason/reason.1.dat").is_file(),
        "--parts 1 should nest like tpchgen-cli"
    );
    assert!(
        !temp_dir.path().join("reason.dat").exists(),
        "--parts 1 should not also produce a flat file"
    );
}

// ----------------
// Test that concatenating a file created with `--parts`
// exactly reproduces a single-file output
//
// Most of the tables are below the 1M minimum size, so only testing the larger
// tables is sufficient to verify the behavior.
//
// For now, to avoid long test times, we test only a subset of the larger tables.
// testing the whole thing is tracked by
// https://github.com/datafusion-contrib/tpcgen-rs/issues/457
// ----------------

// DAT

#[test]
fn test_tpcgen_cli_tpcds_dat_parts_catalog_page() {
    test_dat_parts("catalog_page", 1.0, 4, &[11_718, 0, 0, 0]);
}

#[test]
fn test_tpcgen_cli_tpcds_dat_parts_customer_demographics() {
    test_dat_parts(
        "customer_demographics",
        1.0,
        4,
        &[480_200, 480_200, 480_200, 480_200],
    );
}

#[test]
fn test_tpcgen_cli_tpcds_dat_parts_inventory() {
    test_dat_parts(
        "inventory",
        1.0,
        4,
        &[2_936_250, 2_936_250, 2_936_250, 2_936_250],
    );
}

/// dbgen_version records the command line that generated it, so a `--parts`
/// run is never byte identical to an unsplit one. Check the part layout and
/// row placement instead: the one row belongs to part 1.
#[test]
fn test_tpcgen_cli_tpcds_dat_parts_dbgen_version() {
    let parts_dir = tempdir().expect("Failed to create temporary directory");
    generate_parts("dat", "dbgen_version", 1.0, 4, parts_dir.path());

    let path = parts_dir.path().join("dbgen_version/dbgen_version.1.dat");
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("Expected {path:?} to exist: {err}"));
    assert_eq!(contents.lines().count(), 1, "chunk 1 holds the single row");
    for chunk in 2..=4 {
        let path = part_path(parts_dir.path(), "dbgen_version", chunk, "dat");
        assert!(
            !path.exists(),
            "chunk {chunk} generates no rows, so it must write no file: {path:?}"
        );
    }
}

// CSV

#[test]
fn test_tpcgen_cli_tpcds_csv_parts_catalog_page() {
    test_csv_parts("catalog_page", 1.0, 4, &[11_718, 0, 0, 0]);
}

#[test]
fn test_tpcgen_cli_tpcds_csv_parts_customer_demographics() {
    test_csv_parts(
        "customer_demographics",
        1.0,
        4,
        &[480_200, 480_200, 480_200, 480_200],
    );
}

// PARQUET

#[test]
fn test_tpcgen_cli_tpcds_parquet_parts_catalog_returns() {
    test_parquet_parts("catalog_returns", 1.0, 4, &[144_067, 0, 0, 0]);
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_parts_customer_demographics() {
    test_parquet_parts(
        "customer_demographics",
        1.0,
        4,
        &[480_200, 480_200, 480_200, 480_200],
    );
}

/// Run the CLI once for `table_name` in `format`, writing a single unsplit
/// file per table into `output_dir`.
fn generate_unsplit(format: &str, table_name: &str, scale_factor: f64, output_dir: &Path) {
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg(format)
        .arg("--scale-factor")
        .arg(scale_factor.to_string())
        .arg("--tables")
        .arg(table_name)
        .arg("--output-dir")
        .arg(output_dir)
        .assert()
        .success();
}

/// Run the CLI once for `table_name` in `format`, writing `parts` numbered
/// files per table into `output_dir/<table>/`.
fn generate_parts(
    format: &str,
    table_name: &str,
    scale_factor: f64,
    parts: usize,
    output_dir: &Path,
) {
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg(format)
        .arg("--scale-factor")
        .arg(scale_factor.to_string())
        .arg("--tables")
        .arg(table_name)
        .arg("--output-dir")
        .arg(output_dir)
        .arg("--parts")
        .arg(parts.to_string())
        .assert()
        .success();
}

/// Assert `table_name`'s `--parts` directory holds exactly one file for each
/// entry in `expected_rows`.
///
/// For example, if `expected_rows` is `[100, 200]` this verifies that there are
/// exactly 2 files in `parts_dir/table_name`.
fn assert_part_file_count(parts_dir: &Path, table_name: &str, expected_rows: &[usize]) {
    let expected_files = expected_rows.iter().filter(|rows| **rows > 0).count();
    let num_files = fs::read_dir(parts_dir.join(table_name))
        .map(|dir| dir.count())
        .unwrap_or(0);
    assert_eq!(
        num_files, expected_files,
        "Expected one --parts output file per non-empty part"
    );
}

/// Path of one `--parts` chunk file for `table_name`.
fn part_path(parts_dir: &Path, table_name: &str, chunk: usize, ext: &str) -> PathBuf {
    parts_dir.join(format!("{table_name}/{table_name}.{chunk}.{ext}"))
}

/// Test that concatenating a file created with `--parts` exactly reproduces a
/// single-file output, and that every part holds the expected number of rows.
///
/// `expected_rows` has one entry per expected part.
///
/// For example, given 4 parts, and `expected_rows` is `[n, 0, 0, 0]` a single
/// file with n rows is expected, and no files for the other three parts should
/// exist.
fn test_dat_parts(table_name: &str, scale_factor: f64, parts: usize, expected_rows: &[usize]) {
    assert_eq!(expected_rows.len(), parts, "one row count per part");

    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    generate_unsplit("dat", table_name, scale_factor, unsplit_dir.path());
    let unsplit = fs::read(unsplit_dir.path().join(format!("{table_name}.dat")))
        .expect("unsplit file exists");

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    generate_parts("dat", table_name, scale_factor, parts, parts_dir.path());

    assert_part_file_count(parts_dir.path(), table_name, expected_rows);
    let mut concatenated = Vec::new();
    for (chunk, expected_rows) in (1..=parts).zip(expected_rows.iter().copied()) {
        let path = part_path(parts_dir.path(), table_name, chunk, "dat");
        if expected_rows == 0 {
            assert!(!path.exists(), "an empty part must write no file: {path:?}");
            continue;
        }
        let contents = fs::read(&path).unwrap_or_else(|err| panic!("{path:?} exists: {err}"));
        assert_eq!(
            contents.iter().filter(|byte| **byte == b'\n').count(),
            expected_rows,
            "Unexpected number of rows in chunk {chunk}"
        );
        concatenated.extend(contents);
    }

    assert_eq!(
        concatenated, unsplit,
        "Expected concatenated chunks are not the same as unsplit"
    );
}

/// Test that concatenating CSV files results in the same output as a single CSV
/// file, and that every part holds the expected number of rows.
///
/// See  [`test_dat_parts`]  for details on `expected_rows`.
fn test_csv_parts(table_name: &str, scale_factor: f64, parts: usize, expected_rows: &[usize]) {
    assert_eq!(expected_rows.len(), parts, "one row count per part");

    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    generate_unsplit("csv", table_name, scale_factor, unsplit_dir.path());
    let unsplit = fs::read_to_string(unsplit_dir.path().join(format!("{table_name}.csv")))
        .expect("unsplit exists");

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    generate_parts("csv", table_name, scale_factor, parts, parts_dir.path());

    assert_part_file_count(parts_dir.path(), table_name, expected_rows);

    let mut lines = unsplit.lines();
    let header = lines.next().expect("unsplit CSV has a header");
    let mut expected = header.to_string();
    expected.push('\n');
    expected.push_str(&lines.map(|line| format!("{line}\n")).collect::<String>());

    let mut reconstructed = String::new();
    for (chunk, expected_rows) in (1..=parts).zip(expected_rows.iter().copied()) {
        let path = part_path(parts_dir.path(), table_name, chunk, "csv");
        if expected_rows == 0 {
            assert!(!path.exists(), "an empty part must write no file: {path:?}");
            continue;
        }
        let contents =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path:?} exists: {err}"));
        let mut chunk_lines = contents.lines();
        let chunk_header = chunk_lines.next().expect("every chunk has its own header");
        assert_eq!(chunk_header, header, "every chunk's header matches");
        if reconstructed.is_empty() {
            reconstructed.push_str(header);
            reconstructed.push('\n');
        }
        let rows: Vec<&str> = chunk_lines.collect();
        assert_eq!(
            rows.len(),
            expected_rows,
            "Unexpected number of rows in chunk {chunk}"
        );
        reconstructed.push_str(
            &rows
                .iter()
                .map(|line| format!("{line}\n"))
                .collect::<String>(),
        );
    }

    assert_eq!(
        reconstructed, expected,
        "Expected --parts chunk row data (headers aside) to reproduce the unsplit CSV output"
    );
}

/// Test that concatenating the `--parts` Parquet files reproduces the unsplit
/// Parquet output, and that every part holds the expected number of rows.
///
/// See  [`test_dat_parts`]  for details on `expected_rows`.
fn test_parquet_parts(table_name: &str, scale_factor: f64, parts: usize, expected_rows: &[usize]) {
    assert_eq!(expected_rows.len(), parts, "one row count per part");

    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    generate_unsplit("parquet", table_name, scale_factor, unsplit_dir.path());
    let (unsplit, _row_groups) =
        read_concatenated_parquet(&unsplit_dir.path().join(format!("{table_name}.parquet")));

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    generate_parts("parquet", table_name, scale_factor, parts, parts_dir.path());

    assert_part_file_count(parts_dir.path(), table_name, expected_rows);

    let mut part_batches = vec![];
    for (chunk, expected_rows) in (1..=parts).zip(expected_rows.iter().copied()) {
        let path = part_path(parts_dir.path(), table_name, chunk, "parquet");
        if expected_rows == 0 {
            assert!(!path.exists(), "an empty part must write no file: {path:?}");
            continue;
        }
        assert!(path.exists(), "Expected {path:?} to exist");
        let (batch, _row_groups) = read_concatenated_parquet(&path);
        assert_eq!(
            batch.num_rows(),
            expected_rows,
            "Unexpected number of rows in chunk {chunk}"
        );
        part_batches.push(batch);
    }
    let reconstructed = concat_batches(&unsplit.schema(), &part_batches)
        .expect("Failed to concatenate part batches");
    assert_eq!(
        reconstructed, unsplit,
        "Expected concatenated --parts Parquet batches to match the unsplit Parquet batch"
    );
}

/// `tpcds --stdout` with no subcommand writes the default DAT output to stdout.
#[test]
fn test_tpcgen_cli_tpcds_stdout_matches_file_output_default() {
    assert_stdout_matches_file_output("tpcds", None, "reason", "dat");
}

#[test]
fn test_tpcgen_cli_tpcds_stdout_matches_file_output_dat() {
    assert_stdout_matches_file_output("tpcds", Some("dat"), "reason", "dat");
}

#[test]
fn test_tpcgen_cli_tpcds_stdout_matches_file_output_csv() {
    assert_stdout_matches_file_output("tpcds", Some("csv"), "reason", "csv");
}

#[test]
fn test_tpcgen_cli_tpcds_stdout_matches_file_output_parquet() {
    assert_stdout_matches_file_output("tpcds", Some("parquet"), "reason", "parquet");
}

#[test]
fn test_tpcgen_cli_tpcds_dat_no_overwrite() {
    assert_tpcds_no_overwrite("dat");
}

#[test]
fn test_tpcgen_cli_tpcds_csv_no_overwrite() {
    assert_tpcds_no_overwrite("csv");
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_no_overwrite() {
    assert_tpcds_no_overwrite("parquet");
}

/// Check that an existing TPC-DS `format` output of the reason table is not
/// overwritten, and a warning is logged instead.
fn assert_tpcds_no_overwrite(format: &str) {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let path = temp_dir.path().join(format!("reason.{format}"));
    fs::write(&path, b"existing output").expect("Failed to seed existing output");

    let output = cargo_bin_cmd!("tpcgen-cli")
        .args([
            "tpcds",
            format,
            "--scale-factor",
            "0.001",
            "--tables",
            "reason",
        ])
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let warning = format!("{} already exists, skipping generation", path.display());
    assert!(
        stderr.contains(&warning),
        "Expected {warning:?}, got stderr: {stderr}"
    );
    assert_eq!(fs::read(&path).unwrap(), b"existing output");
    let mut inprogress_path = path.into_os_string();
    inprogress_path.push(".inprogress");
    assert!(!Path::new(&inprogress_path).exists());
}

/// Test that with `--parts`, only the parts that already exist are skipped:
/// the missing parts are still generated into the table's directory.
#[test]
fn test_tpcgen_cli_tpcds_dat_parts_generates_missing_parts() {
    // customer_demographics is large enough (1.9M rows at any scale factor) to
    // be split into two non-empty parts
    let table_name = "customer_demographics";
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let parts_dir = temp_dir.path().join(table_name);
    let existing = parts_dir.join(format!("{table_name}.1.dat"));
    let missing = parts_dir.join(format!("{table_name}.2.dat"));
    fs::create_dir_all(&parts_dir).expect("Failed to create parts directory");
    fs::write(&existing, b"existing output").expect("Failed to seed existing part");

    let output = cargo_bin_cmd!("tpcgen-cli")
        .args([
            "tpcds",
            "dat",
            "--scale-factor",
            "0.001",
            "--tables",
            table_name,
        ])
        .args(["--parts", "2"])
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    // exactly the existing part is skipped
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    let skipped: Vec<&str> = stderr
        .lines()
        .filter(|line| line.contains("already exists, skipping generation"))
        .collect();
    assert_eq!(skipped.len(), 1, "Expected one skipped part, got: {stderr}");
    assert!(
        skipped[0].contains(&existing.display().to_string()),
        "Expected {existing:?} to be skipped, got: {stderr}"
    );
    assert_eq!(fs::read(&existing).unwrap(), b"existing output");
    assert!(missing.is_file());
}

/// Test that format-specific options are grouped under their own help heading.
#[test]
fn test_tpcgen_cli_tpcds_help_groups_format_specific_options() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "parquet",
            "Parquet Options",
            &["--compression", "--row-group-bytes", "--column-encoding"],
        ),
        ("csv", "CSV Options", &["--delimiter"]),
    ];
    for (format, heading, flags) in cases {
        for help_flag in ["-h", "--help"] {
            let assert = cargo_bin_cmd!("tpcgen-cli")
                .args(["tpcds", format, help_flag])
                .assert()
                .success();
            let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
            assert_flags_under_help_heading(&stdout, heading, flags);
        }
    }
}
