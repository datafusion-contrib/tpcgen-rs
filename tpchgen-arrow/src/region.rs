use crate::DEFAULT_BATCH_SIZE;
use arrow::array::{Int64Array, RecordBatch, StringViewArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{RegionGenerator, RegionGeneratorIterator};

/// Generate  [`Region`]s in [`RecordBatch`] format
///
/// [`Region`]: tpchgen::generators::Region
///
/// # Example
/// ```
/// # use tpchgen::generators::{RegionGenerator};
/// # use tpchgen_arrow::RegionArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = RegionGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = RegionArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///   "+-------------+-------------+---------------------------------------------------------------------------------------------------------------------+",
///   "| r_regionkey | r_name      | r_comment                                                                                                           |",
///   "+-------------+-------------+---------------------------------------------------------------------------------------------------------------------+",
///   "| 0           | AFRICA      | lar deposits. blithely final packages cajole. regular waters are final requests. regular accounts are according to  |",
///   "| 1           | AMERICA     | hs use ironic, even requests. s                                                                                     |",
///   "| 2           | ASIA        | ges. thinly even pinto beans ca                                                                                     |",
///   "| 3           | EUROPE      | ly final courts cajole furiously final excuse                                                                       |",
///   "| 4           | MIDDLE EAST | uickly special accounts cajole carefully blithely close requests. carefully final asymptotes haggle furiousl        |",
///   "+-------------+-------------+---------------------------------------------------------------------------------------------------------------------+"
/// ]);
/// ```
pub struct RegionArrow {
    inner: RegionGeneratorIterator<'static>,
    batch_size: usize,
}

impl RegionArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&REGION_SCHEMA)
    }

    pub fn new(generator: RegionGenerator<'static>) -> Self {
        Self {
            inner: generator.iter(),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }

    /// Set the batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchReader for RegionArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for RegionArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        let r_regionkey = Int64Array::from_iter_values(rows.iter().map(|r| r.r_regionkey));
        let r_name = StringViewArray::from_iter_values(rows.iter().map(|r| r.r_name));
        let r_comment = StringViewArray::from_iter_values(rows.iter().map(|r| r.r_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![Arc::new(r_regionkey), Arc::new(r_name), Arc::new(r_comment)],
        );
        Some(batch)
    }
}

/// Schema for the Region
static REGION_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_region_schema);
fn make_region_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("r_regionkey", DataType::Int64, false),
        Field::new("r_name", DataType::Utf8View, false),
        Field::new("r_comment", DataType::Utf8View, false),
    ]))
}
