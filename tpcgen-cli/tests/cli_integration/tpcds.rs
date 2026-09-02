use super::test_helpers::{expect_row_group_sizes, RowGroups};
use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use arrow::datatypes::{DataType, TimeUnit};
use arrow::record_batch::RecordBatchReader;
use assert_cmd::cargo::cargo_bin_cmd;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::metadata::ParquetMetaDataReader;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;
use tpcdsgen::config::{Session, SessionBuilder, Table};
use tpcdsgen_arrow::{StoreReturnsArrow, StoreSalesArrow};

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
        stderr.contains("Generating reason..."),
        "Expected TPC-DS table start log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("Generated reason: 1 rows ->"),
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
                1074775, 1073988, 1073405, 1071992, 1073785, 1072613, 1072227, 1073264, 1073338,
                1072748,
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

            let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
            assert_eq!(stderr.matches("Generating reason...").count(), 1);
            for pair in ["store", "catalog", "web"] {
                assert_eq!(
                    stderr
                        .matches(&format!("Generating {pair}_sales + {pair}_returns..."))
                        .count(),
                    1,
                    "Expected the {pair} sales/returns generator to run once for {format} with {tables}, got stderr: {stderr}"
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
        Some(&"r_reason_sk,r_reason_id,r_reason_description")
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
    assert_eq!(first_line, "r_reason_sk\tr_reason_id\tr_reason_description");
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
        "\"r_reason_sk\"_\"r_reason_id\"_\"r_reason_description\""
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
        .arg("store_sales,store_returns")
        // small row groups to force several source row ranges
        .arg("--row-group-bytes")
        .arg("1000000")
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
            .arg("1000000")
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
        assert_eq!(num_row_groups, 24);

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

/// Test that `--parts` on a table well under dsdgen's 1M-row split threshold
/// (`reason`) puts the whole table in chunk 1 and generates empty files for
/// every other chunk, following dsdgen's own small-table semantics.
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

    let chunk_one =
        fs::read_to_string(temp_dir.path().join("reason/reason.1.dat")).expect("chunk 1 exists");
    assert_eq!(chunk_one.lines().count(), 35, "chunk 1 has every row");

    for chunk in 2..=4 {
        let path = temp_dir.path().join(format!("reason/reason.{chunk}.dat"));
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Expected {path:?} to exist: {err}"));
        assert!(
            contents.is_empty(),
            "Expected chunk {chunk} of `reason` to be empty, got: {contents:?}"
        );
    }
}

/// Test that concatenating every `--parts` chunk of a DAT table, in order,
/// reproduces exactly the unsplit single-file output (dsdgen's chunks are a
/// position-independent partition of the same row sequence).
#[test]
fn test_tpcgen_cli_tpcds_dat_parts_concatenation_matches_unsplit_output() {
    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(unsplit_dir.path())
        .assert()
        .success();
    let unsplit =
        fs::read(unsplit_dir.path().join("call_center.dat")).expect("unsplit file exists");

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(parts_dir.path())
        .arg("--parts")
        .arg("4")
        .assert()
        .success();

    let mut concatenated = Vec::new();
    for chunk in 1..=4 {
        let path = parts_dir
            .path()
            .join(format!("call_center/call_center.{chunk}.dat"));
        concatenated.extend(fs::read(&path).unwrap_or_else(|err| panic!("{path:?} exists: {err}")));
    }

    assert_eq!(
        concatenated, unsplit,
        "Expected concatenated --parts chunks to reproduce the unsplit DAT output"
    );
}

/// Test the same concatenation property for CSV, ignoring each part's own
/// repeated header line (every part is an independently valid CSV file with
/// its own header).
#[test]
fn test_tpcgen_cli_tpcds_csv_parts_concatenation_matches_unsplit_output() {
    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(unsplit_dir.path())
        .assert()
        .success();
    let unsplit =
        fs::read_to_string(unsplit_dir.path().join("call_center.csv")).expect("unsplit exists");

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(parts_dir.path())
        .arg("--parts")
        .arg("4")
        .assert()
        .success();

    let mut lines = unsplit.lines();
    let header = lines.next().expect("unsplit CSV has a header");
    let mut expected = header.to_string();
    expected.push('\n');
    expected.push_str(&lines.map(|line| format!("{line}\n")).collect::<String>());

    let mut reconstructed = String::new();
    for chunk in 1..=4 {
        let path = parts_dir
            .path()
            .join(format!("call_center/call_center.{chunk}.csv"));
        let contents =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path:?} exists: {err}"));
        let mut chunk_lines = contents.lines();
        let chunk_header = chunk_lines.next().expect("every chunk has its own header");
        assert_eq!(chunk_header, header, "every chunk's header matches");
        if chunk == 1 {
            reconstructed.push_str(header);
            reconstructed.push('\n');
        }
        reconstructed.push_str(
            &chunk_lines
                .map(|line| format!("{line}\n"))
                .collect::<String>(),
        );
    }

    assert_eq!(
        reconstructed, expected,
        "Expected --parts chunk row data (headers aside) to reproduce the unsplit CSV output"
    );
}

/// Test that `--parts` on Parquet output produces one file per part whose
/// row counts sum to the unsplit file's row count.
#[test]
fn test_tpcgen_cli_tpcds_parquet_parts_row_counts_sum_to_unsplit() {
    let unsplit_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(unsplit_dir.path())
        .assert()
        .success();
    let unsplit_row_count = parquet_row_count(&unsplit_dir.path().join("call_center.parquet"));

    let parts_dir = tempdir().expect("Failed to create temporary directory");
    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("call_center")
        .arg("--output-dir")
        .arg(parts_dir.path())
        .arg("--parts")
        .arg("4")
        .assert()
        .success();

    let mut total = 0usize;
    for chunk in 1..=4 {
        let path = parts_dir
            .path()
            .join(format!("call_center/call_center.{chunk}.parquet"));
        assert!(path.exists(), "Expected {path:?} to exist");
        total += parquet_row_count(&path);
    }

    assert_eq!(
        total, unsplit_row_count,
        "Expected summed --parts row counts to match the unsplit Parquet row count"
    );
}

/// Return the total row count across all row groups of a Parquet file.
fn parquet_row_count(path: &Path) -> usize {
    let file = File::open(path).unwrap_or_else(|err| panic!("Failed to open {path:?}: {err}"));
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(file).expect("Failed to read Parquet metadata");
    builder
        .build()
        .expect("Failed to build Parquet reader")
        .map(|batch| batch.expect("Failed to read Parquet batch").num_rows())
        .sum()
}
