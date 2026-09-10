use crate::DEFAULT_BATCH_SIZE;
use crate::conversions::{decimal128_array_from_iter, to_arrow_date32};
use arrow::array::{
    Date32Array, Decimal128Array, Int32Array, Int64Array, RecordBatch, StringViewArray,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{LineItemGenerator, LineItemGeneratorIterator};

/// Generate  [`LineItem`]s in [`RecordBatch`] format
///
/// [`LineItem`]: tpchgen::generators::LineItem
///
/// # Example
/// ```
/// # use tpchgen::generators::LineItemGenerator;
/// # use tpchgen_arrow::LineItemArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = LineItemGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = LineItemArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+",
///   "| l_orderkey | l_partkey | l_suppkey | l_linenumber | l_quantity | l_extendedprice | l_discount | l_tax | l_returnflag | l_linestatus | l_shipdate | l_commitdate | l_receiptdate | l_shipinstruct    | l_shipmode | l_comment                           |",
///   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+",
///   "| 1          | 155190    | 7706      | 1            | 17.00      | 21168.23        | 0.04       | 0.02  | N            | O            | 1996-03-13 | 1996-02-12   | 1996-03-22    | DELIVER IN PERSON | TRUCK      | egular courts above the             |",
///   "| 1          | 67310     | 7311      | 2            | 36.00      | 45983.16        | 0.09       | 0.06  | N            | O            | 1996-04-12 | 1996-02-28   | 1996-04-20    | TAKE BACK RETURN  | MAIL       | ly final dependencies: slyly bold   |",
///   "| 1          | 63700     | 3701      | 3            | 8.00       | 13309.60        | 0.10       | 0.02  | N            | O            | 1996-01-29 | 1996-03-05   | 1996-01-31    | TAKE BACK RETURN  | REG AIR    | riously. regular, express dep       |",
///   "| 1          | 2132      | 4633      | 4            | 28.00      | 28955.64        | 0.09       | 0.06  | N            | O            | 1996-04-21 | 1996-03-30   | 1996-05-16    | NONE              | AIR        | lites. fluffily even de             |",
///   "| 1          | 24027     | 1534      | 5            | 24.00      | 22824.48        | 0.10       | 0.04  | N            | O            | 1996-03-30 | 1996-03-14   | 1996-04-01    | NONE              | FOB        |  pending foxes. slyly re            |",
///   "| 1          | 15635     | 638       | 6            | 32.00      | 49620.16        | 0.07       | 0.02  | N            | O            | 1996-01-30 | 1996-02-07   | 1996-02-03    | DELIVER IN PERSON | MAIL       | arefully slyly ex                   |",
///   "| 2          | 106170    | 1191      | 1            | 38.00      | 44694.46        | 0.00       | 0.05  | N            | O            | 1997-01-28 | 1997-01-14   | 1997-02-02    | TAKE BACK RETURN  | RAIL       | ven requests. deposits breach a     |",
///   "| 3          | 4297      | 1798      | 1            | 45.00      | 54058.05        | 0.06       | 0.00  | R            | F            | 1994-02-02 | 1994-01-04   | 1994-02-23    | NONE              | AIR        | ongside of the furiously brave acco |",
///   "| 3          | 19036     | 6540      | 2            | 49.00      | 46796.47        | 0.10       | 0.00  | R            | F            | 1993-11-09 | 1993-12-20   | 1993-11-24    | TAKE BACK RETURN  | RAIL       |  unusual accounts. eve              |",
///   "| 3          | 128449    | 3474      | 3            | 27.00      | 39890.88        | 0.06       | 0.07  | A            | F            | 1994-01-16 | 1993-11-22   | 1994-01-23    | DELIVER IN PERSON | SHIP       | nal foxes wake.                     |",
///   "+------------+-----------+-----------+--------------+------------+-----------------+------------+-------+--------------+--------------+------------+--------------+---------------+-------------------+------------+-------------------------------------+"
/// ]);
/// ```
// # TODOs:
// 1. create individual column iterators to avoid a copy into rows
// 2. Maybe Recycle buffers (don't reallocate new ones all the time) :thinking:
// Based off code / types from DataFusion
// https://github.com/apache/datafusion/blob/a1ae15826245097e7c12d4f0ed3425b25af6c431/benchmarks/src/tpch/mod.rs#L104-L103
pub struct LineItemArrow {
    inner: LineItemGeneratorIterator<'static>,
    batch_size: usize,
}

impl LineItemArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&LINEITEM_SCHEMA)
    }

    pub fn new(generator: LineItemGenerator<'static>) -> Self {
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

impl RecordBatchReader for LineItemArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for LineItemArrow {
    type Item = Result<RecordBatch, ArrowError>;

    /// Generate the next batch of data, if there is one
    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        // Convert column by column
        let l_orderkey = Int64Array::from_iter_values(rows.iter().map(|row| row.l_orderkey));
        let l_partkey = Int64Array::from_iter_values(rows.iter().map(|row| row.l_partkey));
        let l_suppkey = Int64Array::from_iter_values(rows.iter().map(|row| row.l_suppkey));
        let l_linenumber = Int32Array::from_iter_values(rows.iter().map(|row| row.l_linenumber));
        let l_quantity = Decimal128Array::from_iter_values(rows.iter().map(|row| {
            // Convert the i64 to Arrow Decimal(15,2)
            // TODO it is supposed to be decimal in the spec
            (row.l_quantity as i128) * 100
        }))
        .with_precision_and_scale(15, 2)
        .unwrap();
        let l_extended_price =
            decimal128_array_from_iter(rows.iter().map(|row| row.l_extendedprice));
        let l_discount = decimal128_array_from_iter(rows.iter().map(|row| row.l_discount));
        let l_tax = decimal128_array_from_iter(rows.iter().map(|row| row.l_tax));
        let l_returnflag =
            StringViewArray::from_iter_values(rows.iter().map(|row| row.l_returnflag));
        let l_linestatus =
            StringViewArray::from_iter_values(rows.iter().map(|row| row.l_linestatus));
        let l_shipdate = Date32Array::from_iter_values(
            rows.iter().map(|row| row.l_shipdate).map(to_arrow_date32),
        );
        let l_commitdate = Date32Array::from_iter_values(
            rows.iter().map(|row| row.l_commitdate).map(to_arrow_date32),
        );
        let l_receiptdate = Date32Array::from_iter_values(
            rows.iter()
                .map(|row| row.l_receiptdate)
                .map(to_arrow_date32),
        );
        let l_shipinstruct =
            StringViewArray::from_iter_values(rows.iter().map(|row| row.l_shipinstruct));
        let l_shipmode = StringViewArray::from_iter_values(rows.iter().map(|row| row.l_shipmode));
        let l_comment = StringViewArray::from_iter_values(rows.iter().map(|row| row.l_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(l_orderkey),
                Arc::new(l_partkey),
                Arc::new(l_suppkey),
                Arc::new(l_linenumber),
                Arc::new(l_quantity),
                Arc::new(l_extended_price),
                Arc::new(l_discount),
                Arc::new(l_tax),
                Arc::new(l_returnflag),
                Arc::new(l_linestatus),
                Arc::new(l_shipdate),
                Arc::new(l_commitdate),
                Arc::new(l_receiptdate),
                Arc::new(l_shipinstruct),
                Arc::new(l_shipmode),
                Arc::new(l_comment),
            ],
        );
        Some(batch)
    }
}

/// Schema for the LineItem table
static LINEITEM_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_lineitem_schema);

fn make_lineitem_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("l_orderkey", DataType::Int64, false),
        Field::new("l_partkey", DataType::Int64, false),
        Field::new("l_suppkey", DataType::Int64, false),
        Field::new("l_linenumber", DataType::Int32, false),
        Field::new("l_quantity", DataType::Decimal128(15, 2), false),
        Field::new("l_extendedprice", DataType::Decimal128(15, 2), false),
        Field::new("l_discount", DataType::Decimal128(15, 2), false),
        Field::new("l_tax", DataType::Decimal128(15, 2), false),
        Field::new("l_returnflag", DataType::Utf8View, false),
        Field::new("l_linestatus", DataType::Utf8View, false),
        Field::new("l_shipdate", DataType::Date32, false),
        Field::new("l_commitdate", DataType::Date32, false),
        Field::new("l_receiptdate", DataType::Date32, false),
        Field::new("l_shipinstruct", DataType::Utf8View, false),
        Field::new("l_shipmode", DataType::Utf8View, false),
        Field::new("l_comment", DataType::Utf8View, false),
    ]))
}
