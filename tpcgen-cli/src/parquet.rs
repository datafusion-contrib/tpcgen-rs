//! Shared Parquet output helpers.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatchReader;
use futures::StreamExt;
use log::debug;
use parquet::arrow::arrow_writer::{compute_leaves, ArrowColumnChunk};
use parquet::arrow::{add_encoded_arrow_schema_to_metadata, ArrowSchemaConverter};
use parquet::basic::{Compression, Encoding};
use parquet::file::properties::{WriterProperties, WriterPropertiesBuilder, DEFAULT_COERCE_TYPES};
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::types::SchemaDescPtr;
use std::io;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::progress::ProgressHandle;
use crate::tpch_cli::statistics::WriteStatistics;

pub trait IntoSize {
    /// Convert the object into a size
    fn into_size(self) -> Result<usize, io::Error>;
}

pub(crate) fn parse_column_encoding_pair(s: &str) -> Result<(String, Encoding), String> {
    let Some((name, encoding)) = s.split_once('=') else {
        return Err(format!("expected COLUMN=ENCODING, got: '{s}'"));
    };
    let name = name.trim();
    let encoding = encoding.trim();
    if name.is_empty() || encoding.is_empty() {
        return Err(format!("expected COLUMN=ENCODING, got: '{s}'"));
    }
    let encoding = Encoding::from_str(encoding).map_err(|e| e.to_string())?;
    Ok((name.to_string(), encoding))
}

/// Rejects an encoding `--column-encoding` cannot use.
///
/// Dictionary encoding is a writer setting, not a column encoding.
/// `BIT_PACKED` is not supported for writing in this parquet version.
pub(crate) fn reject_unsupported_encoding(encoding: Encoding) -> io::Result<()> {
    match encoding {
        Encoding::PLAIN_DICTIONARY | Encoding::RLE_DICTIONARY => Err(io::Error::other(format!(
            "encoding {encoding} cannot be set with --column-encoding; dictionary encoding is the writer default. Use a non-dictionary encoding such as PLAIN or DELTA_LENGTH_BYTE_ARRAY"
        ))),
        #[allow(deprecated)]
        Encoding::BIT_PACKED => Err(io::Error::other(
            "encoding BIT_PACKED is not supported for Parquet writing",
        )),
        _ => Ok(()),
    }
}

/// Applies `encodings` to `builder`.
///
/// Does not check an encoding against the column's physical type (RLE
/// needs a boolean column, for example). The parquet writer checks this
/// itself, but panics instead of returning an error. Not yet filed
/// upstream in apache/arrow-rs.
///
/// So a bad match only fails once its table starts generating, unlike an
/// unknown column or a rejected encoding. In a multi-table run, another
/// table can finish first.
fn apply_column_encodings(
    mut builder: WriterPropertiesBuilder,
    parquet_schema: &SchemaDescPtr,
    encodings: &[(String, Encoding)],
) -> io::Result<WriterPropertiesBuilder> {
    for (col, enc) in encodings {
        reject_unsupported_encoding(*enc)?;
        let Some(descr) = parquet_schema
            .columns()
            .iter()
            .find(|d| d.name() == col.as_str())
        else {
            return Err(io::Error::other(format!(
                "unknown column '{col}' for --column-encoding"
            )));
        };
        let path = descr.path().clone();
        builder = builder
            .set_column_encoding(path.clone(), *enc)
            .set_column_dictionary_enabled(path, false);
    }
    Ok(builder)
}

/// Converts a set of RecordBatchReaders into a Parquet file.
///
/// Uses num_threads to generate the data in parallel.
///
/// Note the input is an iterator of [`RecordBatchReader`]s; the batches
/// produced by each iterator are encoded as their own row group.
pub async fn generate_parquet<W, I>(
    writer: W,
    iter_iter: I,
    num_threads: usize,
    parquet_compression: Compression,
    column_encodings: Option<&[(String, Encoding)]>,
    progress: ProgressHandle,
) -> Result<(), io::Error>
where
    W: Write + Send + IntoSize + 'static,
    I: Iterator + 'static,
    I::Item: RecordBatchReader + Send,
{
    debug!(
        "Generating Parquet with {num_threads} threads, using {parquet_compression} compression"
    );
    // Based on example in https://docs.rs/parquet/latest/parquet/arrow/arrow_writer/struct.ArrowColumnWriter.html
    let mut iter_iter = iter_iter.peekable();
    let Some(first_iter) = iter_iter.peek() else {
        return Ok(()); // no data
    };
    let schema = first_iter.schema();

    // Compute the parquet schema first. apply_column_encodings needs it to
    // map column names to a ColumnPath and check they exist. Nothing here
    // sets coerce_types, so use the default constant instead of building a
    // WriterProperties just to read it back.
    let parquet_schema = Arc::new(
        ArrowSchemaConverter::new()
            .with_coerce_types(DEFAULT_COERCE_TYPES)
            .convert(&schema)
            .unwrap(),
    );

    let mut builder = WriterProperties::builder().set_compression(parquet_compression);
    if let Some(encodings) = column_encodings {
        builder = apply_column_encodings(builder, &parquet_schema, encodings)?;
    }
    let mut writer_properties = builder.build();
    // Embed the Arrow schema in the Parquet metadata (as ArrowWriter does) so
    // readers recover Arrow types with no exact Parquet equivalent (e.g. the
    // Time32(seconds) column in the TPC-DS dbgen_version table)
    add_encoded_arrow_schema_to_metadata(&schema, &mut writer_properties);
    let writer_properties = Arc::new(writer_properties);

    // create a stream that computes the data for each row group
    let mut row_group_stream = futures::stream::iter(iter_iter)
        .map(async |iter| {
            let parquet_schema = Arc::clone(&parquet_schema);
            let writer_properties = Arc::clone(&writer_properties);
            let schema = Arc::clone(&schema);
            // run on a separate thread
            tokio::task::spawn(async move {
                encode_row_group(parquet_schema, writer_properties, schema, iter)
            })
            .await
            .map_err(|e| io::Error::other(format!("Inner task panicked: {e}")))?
        })
        .buffered(num_threads); // generate row groups in parallel

    let mut statistics = WriteStatistics::new("row groups");

    // A blocking task that writes the row groups to the file
    // done in a blocking task to avoid having a thread waiting on IO
    // Now, read each completed row group and write it to the file
    let root_schema = parquet_schema.root_schema_ptr();
    let writer_properties_captured = Arc::clone(&writer_properties);
    let (tx, mut rx): (
        Sender<Vec<ArrowColumnChunk>>,
        Receiver<Vec<ArrowColumnChunk>>,
    ) = tokio::sync::mpsc::channel(num_threads);
    let writer_task = tokio::task::spawn_blocking(move || {
        // Create parquet writer
        let mut writer =
            SerializedFileWriter::new(writer, root_schema, writer_properties_captured).unwrap();

        while let Some(column_chunks) = rx.blocking_recv() {
            // Start row group
            let mut row_group_writer = writer.next_row_group().unwrap();

            // Slap the chunks into the row group
            for column_chunk in column_chunks {
                column_chunk
                    .append_to_row_group(&mut row_group_writer)
                    .unwrap();
            }
            row_group_writer.close().unwrap();
            statistics.increment_chunks(1);
            progress.increment(1);
        }
        let size = writer.into_inner()?.into_size()?;
        statistics.increment_bytes(size);
        Ok(()) as Result<(), io::Error>
    });

    // now, drive the input stream and send results to the writer task
    while let Some(column_chunks) = row_group_stream.next().await {
        let column_chunks = column_chunks?;
        // send the chunks to the writer task
        if let Err(e) = tx.send(column_chunks).await {
            debug!("Error sending row group to writer: {e}");
            break; // stop early
        }
    }
    // signal the writer task that we are done
    drop(tx);

    // Wait for the writer task to finish
    writer_task.await??;

    Ok(())
}

/// Creates the data for a particular row group.
///
/// Note at the moment it does not use multiple tasks/threads but it could
/// potentially encode multiple columns with different threads.
///
/// Returns an array of [`ArrowColumnChunk`].
fn encode_row_group<I>(
    parquet_schema: SchemaDescPtr,
    writer_properties: Arc<WriterProperties>,
    schema: SchemaRef,
    iter: I,
) -> Result<Vec<ArrowColumnChunk>, io::Error>
where
    I: RecordBatchReader,
{
    // Create writers for each of the leaf columns
    #[allow(deprecated)]
    let mut col_writers = parquet::arrow::arrow_writer::get_column_writers(
        &parquet_schema,
        &writer_properties,
        &schema,
    )
    .map_err(io::Error::other)?;

    // generate the data and send it to the tasks (via the sender channels)
    for batch in iter {
        let batch = batch.map_err(io::Error::other)?;
        let columns = batch.columns().iter();
        let col_writers = col_writers.iter_mut();
        let fields = schema.fields().iter();

        for ((col_writer, field), arr) in col_writers.zip(fields).zip(columns) {
            for leaves in compute_leaves(field.as_ref(), arr).map_err(io::Error::other)? {
                col_writer.write(&leaves).map_err(io::Error::other)?;
            }
        }
    }
    // finish the writers and create the column chunks
    let column_chunks = col_writers
        .into_iter()
        .map(|col_writer| col_writer.close().map_err(io::Error::other))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(column_chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::{ProgressHandle, ProgressTracker};
    use std::fs::File;
    use std::io::BufWriter;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    use tpchgen::generators::RegionGenerator;
    use tpchgen_arrow::RegionArrow;

    #[test]
    fn reject_unsupported_encoding_rejects_dictionary_and_bit_packed() {
        assert!(reject_unsupported_encoding(Encoding::PLAIN_DICTIONARY).is_err());
        assert!(reject_unsupported_encoding(Encoding::RLE_DICTIONARY).is_err());
        #[allow(deprecated)]
        {
            assert!(reject_unsupported_encoding(Encoding::BIT_PACKED).is_err());
        }
        assert!(reject_unsupported_encoding(Encoding::PLAIN).is_ok());
    }

    #[derive(Debug, Default)]
    struct CountingProgress {
        increments: AtomicU64,
    }

    impl ProgressTracker for CountingProgress {
        fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
            ProgressHandle::new(move |row_groups| {
                self.increments.fetch_add(row_groups, Ordering::Relaxed);
            })
        }
    }

    fn region_source() -> RegionArrow {
        RegionArrow::new(RegionGenerator::default()).with_batch_size(5)
    }

    #[tokio::test]
    async fn progress_counts_written_row_groups() {
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("progress.parquet");
        let writer = BufWriter::new(File::create(&output_path).unwrap());

        let tracker = Arc::new(CountingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let progress = progress.register("ignored", 2);

        generate_parquet(
            writer,
            vec![region_source(), region_source()].into_iter(),
            1,
            Compression::UNCOMPRESSED,
            None,
            progress,
        )
        .await
        .unwrap();

        assert_eq!(tracker.increments.load(Ordering::Relaxed), 2);
        assert!(std::fs::metadata(output_path).unwrap().len() > 0);
    }

    async fn write_region(
        encodings: Option<&[(String, Encoding)]>,
        output_path: &std::path::Path,
    ) -> io::Result<()> {
        let writer = BufWriter::new(File::create(output_path).unwrap());
        let tracker = Arc::new(CountingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker;
        let progress = progress.register("region", 1);
        generate_parquet(
            writer,
            vec![region_source()].into_iter(),
            1,
            Compression::UNCOMPRESSED,
            encodings,
            progress,
        )
        .await
    }

    #[tokio::test]
    async fn unknown_column_encoding_returns_error() {
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("unknown.parquet");
        let err = write_region(
            Some(&[(
                "not_a_column".to_string(),
                Encoding::DELTA_LENGTH_BYTE_ARRAY,
            )]),
            &output_path,
        )
        .await
        .unwrap_err();
        assert!(
            err.to_string().contains("unknown column 'not_a_column'"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn dictionary_column_encodings_return_error() {
        let output_dir = tempfile::tempdir().unwrap();
        for encoding in [Encoding::PLAIN_DICTIONARY, Encoding::RLE_DICTIONARY] {
            let output_path = output_dir.path().join(format!("{encoding}.parquet"));
            let err = write_region(Some(&[("r_name".to_string(), encoding)]), &output_path)
                .await
                .unwrap_err();
            let message = err.to_string();
            assert!(
                message.contains("cannot be set with --column-encoding"),
                "encoding {encoding}: {message}"
            );
        }
    }

    #[tokio::test]
    async fn bit_packed_column_encoding_returns_error() {
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("bit_packed.parquet");
        #[allow(deprecated)]
        let err = write_region(
            Some(&[("r_regionkey".to_string(), Encoding::BIT_PACKED)]),
            &output_path,
        )
        .await
        .unwrap_err();
        assert!(
            err.to_string().contains("BIT_PACKED is not supported"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn encoding_incompatible_with_column_type_errors_instead_of_crashing() {
        // We do not check the encoding against the column type here (see
        // `apply_column_encodings`). The parquet writer panics on a bad
        // match instead. We only check that the panic comes back as an
        // `Err`, not the exact message.
        //
        // r_regionkey is INT64, not BOOLEAN. RLE needs a boolean column.
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("regionkey_rle.parquet");
        assert!(write_region(
            Some(&[("r_regionkey".to_string(), Encoding::RLE)]),
            &output_path
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn encoding_compatible_with_column_type_succeeds() {
        let output_dir = tempfile::tempdir().unwrap();

        let output_path = output_dir.path().join("regionkey_delta.parquet");
        write_region(
            Some(&[("r_regionkey".to_string(), Encoding::DELTA_BINARY_PACKED)]),
            &output_path,
        )
        .await
        .unwrap();
        assert!(std::fs::metadata(&output_path).unwrap().len() > 0);

        let output_path = output_dir.path().join("name_delta_length.parquet");
        write_region(
            Some(&[("r_name".to_string(), Encoding::DELTA_LENGTH_BYTE_ARRAY)]),
            &output_path,
        )
        .await
        .unwrap();
        assert!(std::fs::metadata(&output_path).unwrap().len() > 0);
    }
}
