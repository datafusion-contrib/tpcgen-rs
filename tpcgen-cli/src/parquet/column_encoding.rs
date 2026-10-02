//! Temporary preflight checks for Parquet encoders that panic on incompatible types.
//!
//! Replace this module with the upstream fallible API once available:
//! <https://github.com/apache/arrow-rs/issues/10964>.
//! Flag parsing and writer configuration remain in the parent module.
//!
//! # `--column-encoding` compatibility
//!
//! This matrix covers explicit overrides, not automatic writer encodings.
//! Verified with Parquet 59.3.0 and 60.0.0: both have the same matrix.
//! ✅ = nullable values round-trip with the requested data-page encoding.
//! ❌ = rejected as an explicit override; our preflight returns `InvalidInput`.
//!
//! | Encoding | BOOLEAN | INT32 | INT64 | FLOAT | DOUBLE | BYTE_ARRAY | FIXED_LEN_BYTE_ARRAY |
//! |---|---|---|---|---|---|---|---|
//! | `PLAIN` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
//! | `RLE` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
//! | `DELTA_BINARY_PACKED` | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
//! | `DELTA_LENGTH_BYTE_ARRAY` | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
//! | `DELTA_BYTE_ARRAY` | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
//! | `BYTE_STREAM_SPLIT` | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
//! | `PLAIN_DICTIONARY` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
//! | `RLE_DICTIONARY` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
//! | `BIT_PACKED` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
//!
//! Tests cover all 63 cells: 18 successful pairs, 24 incompatible pairs that panic
//! in the unguarded writer, and 21 unsupported-override rejections.
//!
//! `PLAIN_DICTIONARY` and `RLE_DICTIONARY` cannot be configured with
//! `--column-encoding`; dictionary encoding is already the writer's default.
//! `BIT_PACKED` is unsupported for writing.

use super::reject_unsupported_encoding;
use arrow::datatypes::{Schema, SchemaRef};
use parquet::arrow::ArrowSchemaConverter;
use parquet::basic::{Encoding, Type as PhysicalType};
use parquet::file::properties::DEFAULT_COERCE_TYPES;
use parquet::schema::types::SchemaDescPtr;
use std::collections::HashSet;
use std::io;
use std::sync::Arc;

/// Rejects unsupported encodings in every entry, then keeps only the last
/// override for each column, in the order of those final occurrences.
pub(super) fn effective_column_encodings(
    encodings: &[(String, Encoding)],
) -> io::Result<Vec<&(String, Encoding)>> {
    for (column, encoding) in encodings {
        reject_unsupported_encoding(*encoding)
            .map_err(|err| io::Error::new(err.kind(), format!("column '{column}': {err}")))?;
    }
    let mut seen = HashSet::new();
    let mut effective = encodings
        .iter()
        .rev()
        .filter(|(column, _)| seen.insert(column))
        .collect::<Vec<_>>();
    effective.reverse();
    Ok(effective)
}

/// Mirrors the checks made when Parquet encoders are built or receive values.
pub(super) fn check_encoding_supports_type(
    column: &str,
    encoding: Encoding,
    physical_type: PhysicalType,
) -> io::Result<()> {
    reject_unsupported_encoding(encoding)?;
    let supported: &[PhysicalType] = match encoding {
        Encoding::PLAIN => return Ok(()),
        Encoding::RLE => &[PhysicalType::BOOLEAN],
        Encoding::DELTA_BINARY_PACKED => &[PhysicalType::INT32, PhysicalType::INT64],
        Encoding::DELTA_LENGTH_BYTE_ARRAY => &[PhysicalType::BYTE_ARRAY],
        Encoding::DELTA_BYTE_ARRAY => {
            &[PhysicalType::BYTE_ARRAY, PhysicalType::FIXED_LEN_BYTE_ARRAY]
        }
        Encoding::BYTE_STREAM_SPLIT => &[
            PhysicalType::FLOAT,
            PhysicalType::DOUBLE,
            PhysicalType::INT32,
            PhysicalType::INT64,
            PhysicalType::FIXED_LEN_BYTE_ARRAY,
        ],
        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("encoding {other} is not supported for Parquet writing"),
            ))
        }
    };
    if supported.contains(&physical_type) {
        return Ok(());
    }
    let supported = supported
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    Err(io::Error::new(io::ErrorKind::InvalidInput, format!(
        "encoding {encoding} cannot encode column '{column}' of type {physical_type}; {encoding} supports {supported}"
    )))
}

/// Uses the writer's default coercion so validation sees the same physical types.
pub(super) fn parquet_schema(schema: &Schema) -> io::Result<SchemaDescPtr> {
    ArrowSchemaConverter::new()
        .with_coerce_types(DEFAULT_COERCE_TYPES)
        .convert(schema)
        .map(Arc::new)
        .map_err(|err| {
            io::Error::other(format!("failed to convert Arrow schema to Parquet: {err}"))
        })
}

/// Validates effective overrides across all selected tables, converting each
/// schema only once. Columns present on only some selected tables are allowed.
pub(crate) fn validate_column_encodings<'a>(
    tables: impl Iterator<Item = (&'a str, SchemaRef)>,
    encodings: &[(String, Encoding)],
) -> io::Result<()> {
    let encodings = effective_column_encodings(encodings)?;
    if encodings.is_empty() {
        return Ok(());
    }
    let tables = tables
        .map(|(name, schema)| {
            parquet_schema(&schema)
                .map(|schema| (name, schema))
                .map_err(|err| io::Error::new(err.kind(), format!("table '{name}': {err}")))
        })
        .collect::<io::Result<Vec<_>>>()?;
    for (col, enc) in encodings {
        let mut matches_any_table = false;
        for (table, schema) in &tables {
            if let Some(descr) = schema.columns().iter().find(|d| d.name() == col) {
                matches_any_table = true;
                check_encoding_supports_type(col, *enc, descr.physical_type())
                    .map_err(|err| io::Error::new(err.kind(), format!("table '{table}': {err}")))?;
            }
        }
        if !matches_any_table {
            let names = tables
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "column '{col}' for --column-encoding not found in any selected table ({names})"
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{generate_parquet, schema_with_field_ids};
    use super::*;
    use crate::progress::ProgressHandle;
    use arrow::array::{
        ArrayRef, BooleanArray, Decimal128Array, Float32Array, Float64Array, Int32Array,
        Int64Array, StringArray,
    };
    use arrow::datatypes::{DataType, Field};
    use arrow::record_batch::{RecordBatch, RecordBatchIterator};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::basic::Compression;
    use parquet::column::page::Page;
    use parquet::file::properties::WriterProperties;
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;

    #[test]
    fn effective_overrides_keep_final_occurrence_order() {
        let encodings = [
            ("first".to_string(), Encoding::RLE),
            ("second".to_string(), Encoding::PLAIN),
            ("first".to_string(), Encoding::DELTA_BINARY_PACKED),
            ("third".to_string(), Encoding::PLAIN),
        ];
        assert_eq!(
            effective_column_encodings(&encodings).unwrap(),
            vec![&encodings[1], &encodings[2], &encodings[3]],
        );
    }

    #[test]
    fn preflight_checks_all_matching_tables() {
        let tables = [
            (
                "integers",
                Arc::new(Schema::new(vec![Field::new(
                    "value",
                    DataType::Int64,
                    true,
                )])),
            ),
            (
                "strings",
                Arc::new(Schema::new(vec![Field::new("value", DataType::Utf8, true)])),
            ),
        ];
        let err = validate_column_encodings(
            tables.into_iter(),
            &[("value".to_string(), Encoding::DELTA_BINARY_PACKED)],
        )
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        for context in [
            "table 'strings'",
            "'value'",
            "BYTE_ARRAY",
            "DELTA_BINARY_PACKED",
        ] {
            assert!(err.to_string().contains(context), "{err}");
        }
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn unsupported_encodings_are_rejected_for_every_type() {
        let output_dir = tempfile::tempdir().unwrap();
        let mut combinations = 0;
        for (array, physical_type, _) in encoding_test_cases() {
            let batch = encoding_test_batch(array);
            for encoding in [
                Encoding::PLAIN_DICTIONARY,
                Encoding::RLE_DICTIONARY,
                Encoding::BIT_PACKED,
            ] {
                let overrides = [
                    ("value".to_string(), encoding),
                    ("value".to_string(), Encoding::PLAIN),
                ];
                // Reject unsupported names even when a later valid override replaces them.
                for encodings in [&overrides[..1], overrides.as_slice()] {
                    let preflight = validate_column_encodings(
                        [("sample", batch.schema())].into_iter(),
                        encodings,
                    )
                    .unwrap_err();
                    let path = output_dir
                        .path()
                        .join(format!("{physical_type}_{encoding}.parquet"));
                    let write = write_encoding_test_batch(&path, &batch, encodings)
                        .await
                        .unwrap_err();
                    for err in [preflight, write] {
                        assert_eq!(
                            err.kind(),
                            io::ErrorKind::InvalidInput,
                            "{physical_type}: {encoding}: {err}"
                        );
                        assert!(
                            err.to_string().contains(&encoding.to_string()),
                            "{physical_type}: {encoding}: {err}"
                        );
                    }
                    assert_eq!(std::fs::metadata(path).unwrap().len(), 0);
                }
                combinations += 1;
            }
        }
        assert_eq!(combinations, 21);
    }

    #[tokio::test]
    async fn schema_conversion_errors_propagate_before_writing() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "unsupported",
            DataType::Struct(Default::default()),
            true,
        )]));
        let encodings = [("unsupported".to_string(), Encoding::PLAIN)];
        let err = validate_column_encodings([("sample", schema.clone())].into_iter(), &encodings)
            .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("table 'sample'"), "{err}");
        assert!(
            message.contains("failed to convert Arrow schema to Parquet"),
            "{err}"
        );
        assert!(message.contains("empty structs"), "{err}");
        assert!(!message.contains("not found"), "{err}");

        let output_dir = tempfile::tempdir().unwrap();
        // Conversion must also be fallible without any column overrides.
        for encodings in [None, Some(encodings.as_slice())] {
            let path = output_dir.path().join("unsupported.parquet");
            let source = RecordBatchIterator::new(std::iter::empty(), schema.clone());
            let err = generate_parquet(
                BufWriter::new(File::create(&path).unwrap()),
                std::iter::once(source),
                1,
                Compression::UNCOMPRESSED,
                encodings,
                ProgressHandle::new(|_, _| {}),
            )
            .await
            .unwrap_err();
            assert!(
                err.to_string()
                    .contains("failed to convert Arrow schema to Parquet"),
                "{err}"
            );
            assert_eq!(std::fs::metadata(path).unwrap().len(), 0);
        }
    }

    fn encoding_test_cases() -> Vec<(ArrayRef, PhysicalType, &'static [Encoding])> {
        use Encoding::*;
        vec![
            (
                Arc::new(BooleanArray::from(vec![
                    Some(true),
                    None,
                    Some(false),
                    Some(true),
                ])),
                PhysicalType::BOOLEAN,
                &[PLAIN, RLE],
            ),
            (
                Arc::new(Int32Array::from(vec![
                    Some(-42),
                    None,
                    Some(0),
                    Some(i32::MAX),
                ])),
                PhysicalType::INT32,
                &[PLAIN, DELTA_BINARY_PACKED, BYTE_STREAM_SPLIT],
            ),
            (
                Arc::new(Int64Array::from(vec![
                    Some(-42),
                    None,
                    Some(0),
                    Some(i64::MAX),
                ])),
                PhysicalType::INT64,
                &[PLAIN, DELTA_BINARY_PACKED, BYTE_STREAM_SPLIT],
            ),
            (
                Arc::new(Float32Array::from(vec![
                    Some(-1.25),
                    None,
                    Some(0.0),
                    Some(3.5),
                ])),
                PhysicalType::FLOAT,
                &[PLAIN, BYTE_STREAM_SPLIT],
            ),
            (
                Arc::new(Float64Array::from(vec![
                    Some(-1.25),
                    None,
                    Some(0.0),
                    Some(3.5),
                ])),
                PhysicalType::DOUBLE,
                &[PLAIN, BYTE_STREAM_SPLIT],
            ),
            (
                Arc::new(StringArray::from(vec![
                    Some("abc"),
                    None,
                    Some(""),
                    Some("abd"),
                ])),
                PhysicalType::BYTE_ARRAY,
                &[PLAIN, DELTA_LENGTH_BYTE_ARRAY, DELTA_BYTE_ARRAY],
            ),
            (
                Arc::new(
                    Decimal128Array::from(vec![Some(-123), None, Some(0), Some(10_i128.pow(30))])
                        .with_precision_and_scale(38, 2)
                        .unwrap(),
                ),
                PhysicalType::FIXED_LEN_BYTE_ARRAY,
                &[PLAIN, DELTA_BYTE_ARRAY, BYTE_STREAM_SPLIT],
            ),
        ]
    }

    fn encoding_test_batch(array: ArrayRef) -> RecordBatch {
        RecordBatch::try_from_iter_with_nullable([
            ("value", array, true),
            (
                "untouched",
                Arc::new(StringArray::from(vec![
                    Some("repeat"),
                    None,
                    Some("repeat"),
                    Some("repeat"),
                ])),
                true,
            ),
        ])
        .unwrap()
    }

    async fn write_encoding_test_batch(
        path: &Path,
        batch: &RecordBatch,
        overrides: &[(String, Encoding)],
    ) -> io::Result<()> {
        let source = RecordBatchIterator::new(vec![Ok(batch.clone())], batch.schema());
        generate_parquet(
            BufWriter::new(File::create(path).unwrap()),
            std::iter::once(source),
            1,
            Compression::UNCOMPRESSED,
            Some(overrides),
            ProgressHandle::new(|_, _| {}),
        )
        .await
    }

    fn assert_encoding_roundtrip(
        path: &Path,
        batch: &RecordBatch,
        physical_type: PhysicalType,
        encoding: Encoding,
    ) {
        let file = SerializedFileReader::new(File::open(path).unwrap()).unwrap();
        assert_eq!(file.metadata().num_row_groups(), 1);
        let metadata = file.metadata().row_group(0);
        assert_eq!(metadata.column(0).column_type(), physical_type);
        assert!(metadata.column(0).dictionary_page_offset().is_none());
        // Footer RLE may only describe null levels; inspect actual data pages.
        let mut pages = file
            .get_row_group(0)
            .unwrap()
            .get_column_page_reader(0)
            .unwrap();
        let mut data_pages = 0;
        while let Some(page) = pages.get_next_page().unwrap() {
            match page {
                Page::DataPage {
                    encoding: actual, ..
                }
                | Page::DataPageV2 {
                    encoding: actual, ..
                } => {
                    assert_eq!(actual, encoding, "{physical_type}");
                    data_pages += 1;
                }
                Page::DictionaryPage { .. } => panic!("overridden column has a dictionary page"),
            }
        }
        assert!(data_pages > 0);
        let untouched = metadata.column(1);
        assert!(untouched.dictionary_page_offset().is_some());
        assert!(untouched
            .encodings()
            .any(|actual| actual == Encoding::RLE_DICTIONARY));
        let mut reader = ParquetRecordBatchReaderBuilder::try_new(File::open(path).unwrap())
            .unwrap()
            .build()
            .unwrap();
        let expected_batch = RecordBatch::try_new(
            Arc::new(schema_with_field_ids(&batch.schema())),
            batch.columns().to_vec(),
        )
        .unwrap();
        assert_eq!(
            reader.next().unwrap().unwrap(),
            expected_batch,
            "{physical_type}: {encoding}"
        );
        assert!(reader.next().is_none());
    }

    fn incompatible_encoding(physical_type: PhysicalType) -> Encoding {
        if physical_type == PhysicalType::BOOLEAN {
            Encoding::DELTA_BINARY_PACKED
        } else {
            Encoding::RLE
        }
    }

    #[tokio::test]
    async fn allowed_column_encodings_roundtrip_nullable_values() {
        let output_dir = tempfile::tempdir().unwrap();
        let mut combinations = 0;
        for (array, physical_type, encodings) in encoding_test_cases() {
            let batch = encoding_test_batch(array);
            for encoding in encodings {
                // The first override is incompatible, but the final value wins
                // in both benchmark preflight and the standalone writer.
                let mut overrides = [
                    ("value".to_string(), incompatible_encoding(physical_type)),
                    ("value".to_string(), *encoding),
                ];
                validate_column_encodings([("sample", batch.schema())].into_iter(), &overrides)
                    .unwrap();
                let path = output_dir
                    .path()
                    .join(format!("{physical_type}_{encoding}.parquet"));
                write_encoding_test_batch(&path, &batch, &overrides)
                    .await
                    .unwrap();
                assert_encoding_roundtrip(&path, &batch, physical_type, *encoding);

                overrides.reverse();
                let err =
                    validate_column_encodings([("sample", batch.schema())].into_iter(), &overrides)
                        .unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
                let path = output_dir.path().join("invalid.parquet");
                let err = write_encoding_test_batch(&path, &batch, &overrides)
                    .await
                    .unwrap_err();
                assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
                assert_eq!(std::fs::metadata(path).unwrap().len(), 0);
                combinations += 1;
            }
        }
        assert_eq!(combinations, 18);
    }

    #[tokio::test]
    async fn incompatible_pairs_panic_without_preflight_but_return_errors_with_it() {
        use parquet::arrow::ArrowWriter;
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let output_dir = tempfile::tempdir().unwrap();
        let mut combinations = 0;
        for (array, physical_type, allowed) in encoding_test_cases() {
            let batch = encoding_test_batch(array);
            for encoding in [
                Encoding::PLAIN,
                Encoding::RLE,
                Encoding::DELTA_BINARY_PACKED,
                Encoding::DELTA_LENGTH_BYTE_ARRAY,
                Encoding::DELTA_BYTE_ARRAY,
                Encoding::BYTE_STREAM_SPLIT,
            ] {
                if allowed.contains(&encoding) {
                    continue;
                }
                // Bypass our guard. Do not unwrap errors inside catch_unwind:
                // only a panic from the upstream writer counts as evidence.
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let properties = WriterProperties::builder()
                        .set_column_dictionary_enabled("value".into(), false)
                        .set_column_encoding("value".into(), encoding)
                        .build();
                    let mut writer =
                        ArrowWriter::try_new(Vec::new(), batch.schema(), Some(properties))?;
                    writer.write(&batch)?;
                    writer.close().map(|_| ())
                }));
                assert!(result.is_err(), "{physical_type}: {encoding}: {result:?}");

                let overrides = [("value".to_string(), encoding)];
                let preflight =
                    validate_column_encodings([("sample", batch.schema())].into_iter(), &overrides)
                        .unwrap_err();
                assert!(preflight.to_string().contains("table 'sample'"));
                let path = output_dir.path().join("invalid.parquet");
                let write = write_encoding_test_batch(&path, &batch, &overrides)
                    .await
                    .unwrap_err();
                for err in [preflight, write] {
                    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
                    assert!(
                        err.to_string().contains(&format!(
                        "encoding {encoding} cannot encode column 'value' of type {physical_type}"
                    )),
                        "{err}"
                    );
                }
                assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
                combinations += 1;
            }
        }
        assert_eq!(combinations, 24);
    }
}
