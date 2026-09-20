use crate::DEFAULT_BATCH_SIZE;
use crate::conversions::{
    decimal128_array_from_iter, string_view_array_from_display_iter, to_arrow_date32,
};
use arrow::array::{Date32Array, Int32Array, Int64Array, RecordBatch, StringViewArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{OrderGenerator, OrderGeneratorIterator};

/// Generate [`Order`]s in [`RecordBatch`] format
///
/// [`Order`]: tpchgen::generators::Order
///
/// # Example
/// ```
/// # use tpchgen::generators::{OrderGenerator};
/// # use tpchgen_arrow::OrderArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = OrderGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = OrderArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///   "+------------+-----------+---------------+--------------+-------------+-----------------+-----------------+----------------+---------------------------------------------------------------------------+",
///   "| o_orderkey | o_custkey | o_orderstatus | o_totalprice | o_orderdate | o_orderpriority | o_clerk         | o_shippriority | o_comment                                                                 |",
///   "+------------+-----------+---------------+--------------+-------------+-----------------+-----------------+----------------+---------------------------------------------------------------------------+",
///   "| 1          | 36901     | O             | 173665.47    | 1996-01-02  | 5-LOW           | Clerk#000000951 | 0              | nstructions sleep furiously among                                         |",
///   "| 2          | 78002     | O             | 46929.18     | 1996-12-01  | 1-URGENT        | Clerk#000000880 | 0              |  foxes. pending accounts at the pending, silent asymptot                  |",
///   "| 3          | 123314    | F             | 193846.25    | 1993-10-14  | 5-LOW           | Clerk#000000955 | 0              | sly final accounts boost. carefully regular ideas cajole carefully. depos |",
///   "| 4          | 136777    | O             | 32151.78     | 1995-10-11  | 5-LOW           | Clerk#000000124 | 0              | sits. slyly regular warthogs cajole. regular, regular theodolites acro    |",
///   "| 5          | 44485     | F             | 144659.20    | 1994-07-30  | 5-LOW           | Clerk#000000925 | 0              | quickly. bold deposits sleep slyly. packages use slyly                    |",
///   "| 6          | 55624     | F             | 58749.59     | 1992-02-21  | 4-NOT SPECIFIED | Clerk#000000058 | 0              | ggle. special, final requests are against the furiously specia            |",
///   "| 7          | 39136     | O             | 252004.18    | 1996-01-10  | 2-HIGH          | Clerk#000000470 | 0              | ly special requests                                                       |",
///   "| 32         | 130057    | O             | 208660.75    | 1995-07-16  | 2-HIGH          | Clerk#000000616 | 0              | ise blithely bold, regular requests. quickly unusual dep                  |",
///   "| 33         | 66958     | F             | 163243.98    | 1993-10-27  | 3-MEDIUM        | Clerk#000000409 | 0              | uriously. furiously final request                                         |",
///   "| 34         | 61001     | O             | 58949.67     | 1998-07-21  | 3-MEDIUM        | Clerk#000000223 | 0              | ly final packages. fluffily final deposits wake blithely ideas. spe       |",
///   "+------------+-----------+---------------+--------------+-------------+-----------------+-----------------+----------------+---------------------------------------------------------------------------+"
/// ]);
/// ```
pub struct OrderArrow {
    inner: OrderGeneratorIterator<'static>,
    batch_size: usize,
}

impl OrderArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&ORDER_SCHEMA)
    }

    pub fn new(generator: OrderGenerator<'static>) -> Self {
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

impl RecordBatchReader for OrderArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for OrderArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        let o_orderkey = Int64Array::from_iter_values(rows.iter().map(|r| r.o_orderkey));
        let o_custkey = Int64Array::from_iter_values(rows.iter().map(|r| r.o_custkey));
        let o_orderstatus =
            string_view_array_from_display_iter(rows.iter().map(|r| r.o_orderstatus));
        let o_totalprice = decimal128_array_from_iter(rows.iter().map(|r| r.o_totalprice));
        let o_orderdate =
            Date32Array::from_iter_values(rows.iter().map(|r| r.o_orderdate).map(to_arrow_date32));
        let o_orderpriority =
            StringViewArray::from_iter_values(rows.iter().map(|r| r.o_orderpriority));
        let o_clerk = string_view_array_from_display_iter(rows.iter().map(|r| r.o_clerk));
        let o_shippriority = Int32Array::from_iter_values(rows.iter().map(|r| r.o_shippriority));
        let o_comment = StringViewArray::from_iter_values(rows.iter().map(|r| r.o_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(o_orderkey),
                Arc::new(o_custkey),
                Arc::new(o_orderstatus),
                Arc::new(o_totalprice),
                Arc::new(o_orderdate),
                Arc::new(o_orderpriority),
                Arc::new(o_clerk),
                Arc::new(o_shippriority),
                Arc::new(o_comment),
            ],
        );
        Some(batch)
    }
}

/// Schema for the Order
static ORDER_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_order_schema);
fn make_order_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("o_orderkey", DataType::Int64, false),
        Field::new("o_custkey", DataType::Int64, false),
        Field::new("o_orderstatus", DataType::Utf8View, false),
        Field::new("o_totalprice", DataType::Decimal128(15, 2), false),
        Field::new("o_orderdate", DataType::Date32, false),
        Field::new("o_orderpriority", DataType::Utf8View, false),
        Field::new("o_clerk", DataType::Utf8View, false),
        Field::new("o_shippriority", DataType::Int32, false),
        Field::new("o_comment", DataType::Utf8View, false),
    ]))
}
