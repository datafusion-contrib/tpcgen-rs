use assert_cmd::{cargo::cargo_bin_cmd, Command};
use parquet::basic::Encoding;
use parquet::file::metadata::ParquetMetaDataReader;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

struct Benchmark {
    name: &'static str,
    tables: &'static str,
    overridden_table: &'static str,
    integer_column: &'static str,
    integer_type: &'static str,
    string_column: &'static str,
}

const BENCHMARKS: [Benchmark; 2] = [
    Benchmark {
        name: "tpch",
        tables: "nation,region",
        overridden_table: "region",
        integer_column: "r_regionkey",
        integer_type: "INT64",
        string_column: "r_name",
    },
    Benchmark {
        name: "tpcds",
        tables: "ship_mode,reason",
        overridden_table: "reason",
        integer_column: "r_reason_sk",
        integer_type: "INT32",
        string_column: "r_reason_desc",
    },
];

impl Benchmark {
    fn command(&self, output_dir: &Path) -> Command {
        let mut command = cargo_bin_cmd!("tpcgen-cli");
        command
            .args([
                self.name,
                "parquet",
                "--scale-factor",
                "0.001",
                "--tables",
                self.tables,
                "--num-threads",
                "1",
                "--quiet",
            ])
            .arg("--output-dir")
            .arg(output_dir)
            .timeout(Duration::from_secs(60));
        command
    }

    fn columns(&self) -> [(&str, &str, Encoding, Encoding); 2] {
        [
            (
                self.integer_column,
                self.integer_type,
                Encoding::DELTA_BINARY_PACKED,
                Encoding::DELTA_LENGTH_BYTE_ARRAY,
            ),
            (
                self.string_column,
                "BYTE_ARRAY",
                Encoding::DELTA_LENGTH_BYTE_ARRAY,
                Encoding::DELTA_BINARY_PACKED,
            ),
        ]
    }

    fn expect_encodings(&self, output_dir: &Path) {
        let path = output_dir.join(format!("{}.parquet", self.overridden_table));
        for (column, _, encoding, _) in self.columns() {
            super::test_helpers::expect_column_encoding(&path, column, encoding);
        }
        for table in self.tables.split(',') {
            let path = output_dir.join(format!("{table}.parquet"));
            let file = File::open(&path).expect("Failed to open parquet file");
            let mut reader = ParquetMetaDataReader::new();
            reader.try_parse(&file).unwrap();
            let metadata = reader.finish().unwrap();
            assert!(metadata.file_metadata().num_rows() > 0, "{path:?} is empty");
            for row_group in metadata.row_groups() {
                for column in row_group.columns() {
                    let name = column.column_path().string();
                    let overridden = name == self.integer_column || name == self.string_column;
                    assert_eq!(
                        column.dictionary_page_offset().is_some(),
                        !overridden,
                        "{table}.{name}"
                    );
                }
            }
        }
    }
}

fn add_overrides(command: &mut Command, entries: &[String], comma_separated: bool) {
    if comma_separated {
        command.arg("--column-encoding").arg(entries.join(","));
    } else {
        for entry in entries {
            command.arg("--column-encoding").arg(entry);
        }
    }
}

fn expect_preflight_failure(mut command: Command, output_dir: &Path, diagnostics: &[&str]) {
    let result = command.assert().failure().stdout("");
    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    for diagnostic in diagnostics {
        assert!(
            stderr.contains(diagnostic),
            "expected {diagnostic:?} in stderr: {stderr}"
        );
    }
    assert!(!stderr.contains("panicked"), "unexpected panic: {stderr}");
    assert!(!output_dir.exists(), "validation created {output_dir:?}");
    assert!(
        !output_dir.parent().unwrap().exists(),
        "validation created an ancestor of {output_dir:?}"
    );
}

#[test]
fn unknown_columns_fail_before_creating_output_directory() {
    for benchmark in &BENCHMARKS {
        let root = tempfile::tempdir_in(".").unwrap();
        let output_dir = root.path().join("nested/output");
        let column = format!("{}_typo", benchmark.string_column);
        let mut command = benchmark.command(&output_dir);
        command
            .arg("--column-encoding")
            .arg(format!("{column}=DELTA_LENGTH_BYTE_ARRAY"));
        expect_preflight_failure(
            command,
            &output_dir,
            &[
                &format!("column '{column}'"),
                "not found in any selected table",
            ],
        );
    }
}

#[test]
fn tpcds_partitioned_invalid_encodings_leave_outputs_untouched() {
    let benchmark = &BENCHMARKS[1];
    for (encoding, diagnostic) in [
        ("missing=PLAIN", "not found in any selected table"),
        ("r_reason_sk=RLE_DICTIONARY", "dictionary encoding"),
        ("r_reason_sk=BIT_PACKED", "not supported"),
        ("r_reason_sk=RLE", "type INT32"),
    ] {
        for stdout in [false, true] {
            for existing in [false, true] {
                let root = tempfile::tempdir_in(".").unwrap();
                let output_dir = root.path().join("nested/output");
                let mut preserved = Vec::new();
                if existing {
                    for table in benchmark.tables.split(',') {
                        let table_dir = output_dir.join(table);
                        std::fs::create_dir_all(&table_dir).unwrap();
                        for suffix in ["parquet", "parquet.inprogress"] {
                            let path = table_dir.join(format!("{table}.1.{suffix}"));
                            let contents = format!("existing {table} {suffix}").into_bytes();
                            std::fs::write(&path, &contents).unwrap();
                            preserved.push((path, contents));
                        }
                    }
                }
                let mut command = benchmark.command(&output_dir);
                command.args(["--parts", "1", "--overwrite", "--column-encoding", encoding]);
                if stdout {
                    command.arg("--stdout");
                }
                let result = command.assert().failure().stdout("");
                let stderr = String::from_utf8_lossy(&result.get_output().stderr);
                assert!(stderr.contains(diagnostic), "{encoding}: {stderr}");
                assert!(!stderr.contains("panicked"), "{stderr}");

                if existing {
                    for (path, contents) in preserved {
                        assert_eq!(std::fs::read(&path).unwrap(), contents, "{path:?}");
                    }
                    assert_eq!(std::fs::read_dir(&output_dir).unwrap().count(), 2);
                    for table in benchmark.tables.split(',') {
                        assert_eq!(
                            std::fs::read_dir(output_dir.join(table)).unwrap().count(),
                            2
                        );
                    }
                } else {
                    assert!(!output_dir.exists(), "validation created {output_dir:?}");
                    assert!(!output_dir.parent().unwrap().exists());
                }
            }
        }
    }
}

#[test]
fn incompatible_encoding_fails_before_stdout() {
    for benchmark in &BENCHMARKS {
        let root = tempfile::tempdir_in(".").unwrap();
        let output_dir = root.path().join("nested/output");
        let mut command = benchmark.command(&output_dir);
        command
            .args(["--stdout", "--column-encoding"])
            .arg(format!("{}=RLE", benchmark.integer_column));
        expect_preflight_failure(
            command,
            &output_dir,
            &[
                &format!("table '{}'", benchmark.overridden_table),
                benchmark.integer_column,
                benchmark.integer_type,
                "RLE",
            ],
        );
    }
}

#[test]
fn final_compatible_overrides_win_and_preserve_other_columns_dictionary_encoding() {
    for benchmark in &BENCHMARKS {
        for comma_separated in [false, true] {
            let root = tempfile::tempdir_in(".").unwrap();
            let output_dir = root.path().join("nested/output");
            let mut command = benchmark.command(&output_dir);
            let entries: Vec<_> = benchmark
                .columns()
                .iter()
                .flat_map(|(column, _, compatible, incompatible)| {
                    [
                        format!("{column}={incompatible}"),
                        format!("{column}={compatible}"),
                    ]
                })
                .collect();
            add_overrides(&mut command, &entries, comma_separated);
            command.assert().success().stdout("").stderr("");
            benchmark.expect_encodings(&output_dir);
        }
    }
}

#[test]
fn final_incompatible_overrides_fail_before_creating_output_directory() {
    for benchmark in &BENCHMARKS {
        for comma_separated in [false, true] {
            for (column, physical_type, compatible, incompatible) in benchmark.columns() {
                let root = tempfile::tempdir_in(".").unwrap();
                let output_dir = root.path().join("nested/output");
                let mut command = benchmark.command(&output_dir);
                add_overrides(
                    &mut command,
                    &[
                        format!("{column}={compatible}"),
                        format!("{column}={incompatible}"),
                    ],
                    comma_separated,
                );
                expect_preflight_failure(
                    command,
                    &output_dir,
                    &[
                        &format!("table '{}'", benchmark.overridden_table),
                        &format!("column '{column}'"),
                        physical_type,
                        &incompatible.to_string(),
                    ],
                );
            }
        }
    }
}

#[test]
fn invalid_earlier_entries_cannot_be_overridden() {
    for benchmark in &BENCHMARKS {
        let column = benchmark.string_column;
        for comma_separated in [false, true] {
            for (invalid, diagnostic) in [
                (column.to_string(), "expected COLUMN=ENCODING"),
                (format!("{column}="), "expected COLUMN=ENCODING"),
                ("=PLAIN".to_string(), "expected COLUMN=ENCODING"),
                (format!("{column}=NOT_AN_ENCODING"), "invalid value"),
                (format!("{column}=PLAIN_DICTIONARY"), "PLAIN_DICTIONARY"),
                (format!("{column}=RLE_DICTIONARY"), "RLE_DICTIONARY"),
                (format!("{column}=BIT_PACKED"), "BIT_PACKED"),
            ] {
                let root = tempfile::tempdir_in(".").unwrap();
                let output_dir = root.path().join("nested/output");
                let mut command = benchmark.command(&output_dir);
                add_overrides(
                    &mut command,
                    &[invalid, format!("{column}=DELTA_LENGTH_BYTE_ARRAY")],
                    comma_separated,
                );
                expect_preflight_failure(command, &output_dir, &[diagnostic]);
            }
        }
    }
}
