use crate::DEFAULT_BATCH_SIZE;
use crate::conversions::{decimal128_array_from_iter, string_view_array_from_display_iter};
use arrow::array::{Int32Array, Int64Array, RecordBatch, StringViewArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{PartGenerator, PartGeneratorIterator};

/// Generate [`Part`]s in [`RecordBatch`] format
///
/// [`Part`]: tpchgen::generators::Part
///
/// # Example
/// ```
/// # use tpchgen::generators::{PartGenerator};
/// # use tpchgen_arrow::PartArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = PartGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = PartArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///  "+-----------+------------------------------------------+----------------+----------+-------------------------+--------+-------------+---------------+----------------------+",
///   "| p_partkey | p_name                                   | p_mfgr         | p_brand  | p_type                  | p_size | p_container | p_retailprice | p_comment            |",
///   "+-----------+------------------------------------------+----------------+----------+-------------------------+--------+-------------+---------------+----------------------+",
///   "| 1         | goldenrod lavender spring chocolate lace | Manufacturer#1 | Brand#13 | PROMO BURNISHED COPPER  | 7      | JUMBO PKG   | 901.00        | ly. slyly ironi      |",
///   "| 2         | blush thistle blue yellow saddle         | Manufacturer#1 | Brand#13 | LARGE BRUSHED BRASS     | 1      | LG CASE     | 902.00        | lar accounts amo     |",
///   "| 3         | spring green yellow purple cornsilk      | Manufacturer#4 | Brand#42 | STANDARD POLISHED BRASS | 21     | WRAP CASE   | 903.00        | egular deposits hag  |",
///   "| 4         | cornflower chocolate smoke green pink    | Manufacturer#3 | Brand#34 | SMALL PLATED BRASS      | 14     | MED DRUM    | 904.00        | p furiously r        |",
///   "| 5         | forest brown coral puff cream            | Manufacturer#3 | Brand#32 | STANDARD POLISHED TIN   | 15     | SM PKG      | 905.00        |  wake carefully      |",
///   "| 6         | bisque cornflower lawn forest magenta    | Manufacturer#2 | Brand#24 | PROMO PLATED STEEL      | 4      | MED BAG     | 906.00        | sual a               |",
///   "| 7         | moccasin green thistle khaki floral      | Manufacturer#1 | Brand#11 | SMALL PLATED COPPER     | 45     | SM BAG      | 907.00        | lyly. ex             |",
///   "| 8         | misty lace thistle snow royal            | Manufacturer#4 | Brand#44 | PROMO BURNISHED TIN     | 41     | LG DRUM     | 908.00        | eposi                |",
///   "| 9         | thistle dim navajo dark gainsboro        | Manufacturer#4 | Brand#43 | SMALL BURNISHED STEEL   | 12     | WRAP CASE   | 909.00        | ironic foxe          |",
///   "| 10        | linen pink saddle puff powder            | Manufacturer#5 | Brand#54 | LARGE BURNISHED STEEL   | 44     | LG CAN      | 910.01        | ithely final deposit |",
///   "+-----------+------------------------------------------+----------------+----------+-------------------------+--------+-------------+---------------+----------------------+"
/// ]);
/// ```
pub struct PartArrow {
    inner: PartGeneratorIterator<'static>,
    batch_size: usize,
}

impl PartArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&PART_SCHEMA)
    }

    pub fn new(generator: PartGenerator<'static>) -> Self {
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

impl RecordBatchReader for PartArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for PartArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        let p_partkey = Int64Array::from_iter_values(rows.iter().map(|r| r.p_partkey));
        let p_name = string_view_array_from_display_iter(rows.iter().map(|r| &r.p_name));
        let p_mfgr = string_view_array_from_display_iter(rows.iter().map(|r| r.p_mfgr));
        let p_brand = string_view_array_from_display_iter(rows.iter().map(|r| r.p_brand));
        let p_type = StringViewArray::from_iter_values(rows.iter().map(|r| r.p_type));
        let p_size = Int32Array::from_iter_values(rows.iter().map(|r| r.p_size));
        let p_container = StringViewArray::from_iter_values(rows.iter().map(|r| r.p_container));
        let p_retailprice = decimal128_array_from_iter(rows.iter().map(|r| r.p_retailprice));
        let p_comment = StringViewArray::from_iter_values(rows.iter().map(|r| r.p_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(p_partkey),
                Arc::new(p_name),
                Arc::new(p_mfgr),
                Arc::new(p_brand),
                Arc::new(p_type),
                Arc::new(p_size),
                Arc::new(p_container),
                Arc::new(p_retailprice),
                Arc::new(p_comment),
            ],
        );
        Some(batch)
    }
}

/// Schema for the Part
static PART_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_part_schema);
fn make_part_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("p_partkey", DataType::Int64, false),
        Field::new("p_name", DataType::Utf8View, false),
        Field::new("p_mfgr", DataType::Utf8View, false),
        Field::new("p_brand", DataType::Utf8View, false),
        Field::new("p_type", DataType::Utf8View, false),
        Field::new("p_size", DataType::Int32, false),
        Field::new("p_container", DataType::Utf8View, false),
        Field::new("p_retailprice", DataType::Decimal128(15, 2), false),
        Field::new("p_comment", DataType::Utf8View, false),
    ]))
}
