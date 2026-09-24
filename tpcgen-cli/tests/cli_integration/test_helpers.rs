use assert_cmd::cargo::cargo_bin_cmd;
use parquet::basic::Encoding;
use parquet::file::metadata::ParquetMetaDataReader;
use std::fs;
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;

#[derive(Debug, PartialEq)]
pub(crate) struct RowGroups {
    pub(crate) table: &'static str,
    /// total bytes in each row group
    pub(crate) row_group_bytes: Vec<i64>,
}

/// For each table in tables, check that the parquet file in output_dir has
/// a file with the expected row group sizes.
pub(crate) fn expect_row_group_sizes(output_dir: &Path, expected_row_groups: Vec<RowGroups>) {
    let mut actual_row_groups = vec![];
    for table in &expected_row_groups {
        let output_path = output_dir.join(format!("{}.parquet", table.table));
        assert!(
            output_path.exists(),
            "Expected parquet file {:?} to exist",
            output_path
        );
        // read the metadata to get the row group size
        let file = File::open(&output_path).expect("Failed to open parquet file");
        let mut metadata_reader = ParquetMetaDataReader::new();
        metadata_reader.try_parse(&file).unwrap();
        let metadata = metadata_reader.finish().unwrap();
        let row_groups = metadata.row_groups();
        let actual_row_group_bytes: Vec<_> =
            row_groups.iter().map(|rg| rg.total_byte_size()).collect();
        actual_row_groups.push(RowGroups {
            table: table.table,
            row_group_bytes: actual_row_group_bytes,
        })
    }
    // compare the expected and actual row groups debug print actual on failure
    // for better output / easier comparison
    let expected_row_groups = format!("{expected_row_groups:#?}");
    let actual_row_groups = format!("{actual_row_groups:#?}");
    assert_eq!(actual_row_groups, expected_row_groups);
}

/// Asserts `column` uses `expected` as one of its encodings in *every* row
/// group of the file at `path` (not just the first row group that happens to
/// contain it), so a regression that only affects later row groups (e.g. a
/// dictionary-fallback threshold silently reverting to a different encoding
/// partway through the file) doesn't go unnoticed.
pub(crate) fn expect_column_encoding(path: &Path, column: &str, expected: Encoding) {
    let file = File::open(path).expect("Failed to open parquet file");
    let mut metadata_reader = ParquetMetaDataReader::new();
    metadata_reader.try_parse(&file).unwrap();
    let metadata = metadata_reader.finish().unwrap();
    let mut found_in_any_row_group = false;
    for (row_group_idx, row_group) in metadata.row_groups().iter().enumerate() {
        for col in row_group.columns() {
            if col.column_path().string() == column {
                found_in_any_row_group = true;
                let encodings: Vec<Encoding> = col.encodings().collect();
                assert!(
                    encodings.contains(&expected),
                    "expected {column} to use {expected:?} in row group {row_group_idx}, encodings: {encodings:?}"
                );
            }
        }
    }
    assert!(
        found_in_any_row_group,
        "column {column} not found in {}",
        path.display()
    );
}

/// Generate `table` from `benchmark` (`tpch` or `tpcds`) with `subcommand`
/// (the benchmark's default output format when `None`), once to a file and
/// once with `--stdout`, and assert the bytes written to stdout are exactly
/// the bytes of the generated file. Also check the reported byte counts.
pub(crate) fn assert_stdout_matches_file_output(
    benchmark: &str,
    subcommand: Option<&str>,
    table: &str,
    extension: &str,
) {
    let command = |output_dir: &Path| {
        let mut command = cargo_bin_cmd!("tpcgen-cli");
        command.arg(benchmark);
        if let Some(subcommand) = subcommand {
            command.arg(subcommand);
        }
        command
            .env("RUST_LOG", "debug")
            .arg("--no-progress")
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg(table)
            .arg("--output-dir")
            .arg(output_dir);
        command
    };

    let file_dir = tempdir().expect("Failed to create temporary directory");
    let file_assert = command(file_dir.path()).assert().success().stdout("");
    let expected = fs::read(file_dir.path().join(format!("{table}.{extension}")))
        .expect("Failed to read generated file");

    let stdout_dir = tempdir().expect("Failed to create temporary directory");
    // run the --stdout version with a directory that doesn't exist
    let unused_dir = stdout_dir.path().join("unused");
    let assert = command(&unused_dir).arg("--stdout").assert().success();

    assert_eq!(
        assert.get_output().stdout,
        expected,
        "Expected --stdout output to match the generated {table}.{extension}"
    );
    assert!(
        !unused_dir.exists(),
        "Expected --stdout to write no files, but {unused_dir:?} was created"
    );

    for output in [file_assert, assert] {
        output.stderr(predicates::str::contains(format!(
            "Wrote {} bytes in ",
            expected.len()
        )));
    }
}

/// Assert that `help` lists each of `flags` only under the `heading` section.
pub fn assert_flags_under_help_heading(help: &str, heading: &str, flags: &[&str]) {
    let (before, section) = help
        .split_once(&format!("\n{heading}:\n"))
        .unwrap_or_else(|| panic!("Expected `{heading}:` heading in help output: {help}"));
    // The section ends at the next unindented line (another heading).
    let section = section
        .match_indices('\n')
        .find(|(index, _)| {
            section[index + 1..]
                .chars()
                .next()
                .is_some_and(|c| !c.is_whitespace())
        })
        .map_or(section, |(index, _)| &section[..index]);
    for flag in flags {
        assert!(
            section.contains(flag),
            "Expected {flag} under `{heading}:`, got help output: {help}"
        );
        assert!(
            !before.contains(flag),
            "Expected {flag} only under `{heading}:`, got help output: {help}"
        );
    }
}
