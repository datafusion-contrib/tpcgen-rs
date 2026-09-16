use crate::DEFAULT_BATCH_SIZE;
use crate::conversions::{decimal128_array_from_iter, string_view_array_from_display_iter};
use arrow::array::{Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{CustomerGenerator, CustomerGeneratorIterator};

/// Generate [`Customer`]s in [`RecordBatch`] format
///
/// [`Customer`]: tpchgen::generators::Customer
///
/// # Example
/// ```
/// # use tpchgen::generators::{CustomerGenerator};
/// # use tpchgen_arrow::CustomerArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = CustomerGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = CustomerArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///   "+-----------+--------------------+---------------------------------------+-------------+-----------------+-----------+--------------+-------------------------------------------------------------------------------------------------------------------+",
///   "| c_custkey | c_name             | c_address                             | c_nationkey | c_phone         | c_acctbal | c_mktsegment | c_comment                                                                                                         |",
///   "+-----------+--------------------+---------------------------------------+-------------+-----------------+-----------+--------------+-------------------------------------------------------------------------------------------------------------------+",
///   "| 1         | Customer#000000001 | IVhzIApeRb ot,c,E                     | 15          | 25-989-741-2988 | 711.56    | BUILDING     | to the even, regular platelets. regular, ironic epitaphs nag e                                                    |",
///   "| 2         | Customer#000000002 | XSTf4,NCwDVaWNe6tEgvwfmRchLXak        | 13          | 23-768-687-3665 | 121.65    | AUTOMOBILE   | l accounts. blithely ironic theodolites integrate boldly: caref                                                   |",
///   "| 3         | Customer#000000003 | MG9kdTD2WBHm                          | 1           | 11-719-748-3364 | 7498.12   | AUTOMOBILE   |  deposits eat slyly ironic, even instructions. express foxes detect slyly. blithely even accounts abov            |",
///   "| 4         | Customer#000000004 | XxVSJsLAGtn                           | 4           | 14-128-190-5944 | 2866.83   | MACHINERY    |  requests. final, regular ideas sleep final accou                                                                 |",
///   "| 5         | Customer#000000005 | KvpyuHCplrB84WgAiGV6sYpZq7Tj          | 3           | 13-750-942-6364 | 794.47    | HOUSEHOLD    | n accounts will have to unwind. foxes cajole accor                                                                |",
///   "| 6         | Customer#000000006 | sKZz0CsnMD7mp4Xd0YrBvx,LREYKUWAh yVn  | 20          | 30-114-968-4951 | 7638.57   | AUTOMOBILE   | tions. even deposits boost according to the slyly bold packages. final accounts cajole requests. furious          |",
///   "| 7         | Customer#000000007 | TcGe5gaZNgVePxU5kRrvXBfkasDTea        | 18          | 28-190-982-9759 | 9561.95   | AUTOMOBILE   | ainst the ironic, express theodolites. express, even pinto beans among the exp                                    |",
///   "| 8         | Customer#000000008 | I0B10bB0AymmC, 0PrRYBCP1yGJ8xcBPmWhl5 | 17          | 27-147-574-9335 | 6819.74   | BUILDING     | among the slyly regular theodolites kindle blithely courts. carefully even theodolites haggle slyly along the ide |",
///   "| 9         | Customer#000000009 | xKiAFTjUsCuxfeleNqefumTrjS            | 8           | 18-338-906-3675 | 8324.07   | FURNITURE    | r theodolites according to the requests wake thinly excuses: pending requests haggle furiousl                     |",
///   "| 10        | Customer#000000010 | 6LrEaV6KR6PLVcgl2ArL Q3rqzLzcT1 v2    | 5           | 15-741-346-9870 | 2753.54   | HOUSEHOLD    | es regular deposits haggle. fur                                                                                   |",
///   "+-----------+--------------------+---------------------------------------+-------------+-----------------+-----------+--------------+-------------------------------------------------------------------------------------------------------------------+",
///   ]);
/// ```
pub struct CustomerArrow {
    inner: CustomerGeneratorIterator<'static>,
    batch_size: usize,
}

impl CustomerArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&CUSTOMER_SCHEMA)
    }

    pub fn new(generator: CustomerGenerator<'static>) -> Self {
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

impl RecordBatchReader for CustomerArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CustomerArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        let c_custkey = Int64Array::from_iter_values(rows.iter().map(|r| r.c_custkey));
        let c_name = string_view_array_from_display_iter(rows.iter().map(|r| r.c_name));
        let c_address = string_view_array_from_display_iter(rows.iter().map(|r| &r.c_address));
        let c_nationkey = Int64Array::from_iter_values(rows.iter().map(|r| r.c_nationkey));
        let c_phone = string_view_array_from_display_iter(rows.iter().map(|r| &r.c_phone));
        let c_acctbal = decimal128_array_from_iter(rows.iter().map(|r| r.c_acctbal));
        let c_mktsegment = string_view_array_from_display_iter(rows.iter().map(|r| r.c_mktsegment));
        let c_comment = string_view_array_from_display_iter(rows.iter().map(|r| r.c_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(c_custkey),
                Arc::new(c_name),
                Arc::new(c_address),
                Arc::new(c_nationkey),
                Arc::new(c_phone),
                Arc::new(c_acctbal),
                Arc::new(c_mktsegment),
                Arc::new(c_comment),
            ],
        );
        Some(batch)
    }
}

/// Schema for the Customer
static CUSTOMER_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_customer_schema);
fn make_customer_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("c_custkey", DataType::Int64, false),
        Field::new("c_name", DataType::Utf8View, false),
        Field::new("c_address", DataType::Utf8View, false),
        Field::new("c_nationkey", DataType::Int64, false),
        Field::new("c_phone", DataType::Utf8View, false),
        Field::new("c_acctbal", DataType::Decimal128(15, 2), false),
        Field::new("c_mktsegment", DataType::Utf8View, false),
        Field::new("c_comment", DataType::Utf8View, false),
    ]))
}
