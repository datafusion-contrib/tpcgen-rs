use crate::DEFAULT_BATCH_SIZE;
use crate::conversions::{decimal128_array_from_iter, string_view_array_from_display_iter};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpchgen::generators::{PartSuppGenerator, PartSuppGeneratorIterator};

/// Generate [`PartSupp`]s in [`RecordBatch`] format
///
/// [`PartSupp`]: tpchgen::generators::PartSupp
///
/// # Example
/// ```
/// # use tpchgen::generators::{PartSuppGenerator};
/// # use tpchgen_arrow::PartSuppArrow;
///
/// // Create a SF=1.0 generator and wrap it in an Arrow generator
/// let generator = PartSuppGenerator::new(1.0, 1, 1);
/// let mut arrow_generator = PartSuppArrow::new(generator)
///   .with_batch_size(10);
/// // Read the first 10 batches
/// let batch = arrow_generator.next().unwrap().unwrap();
/// // compare the output by pretty printing it
/// let formatted_batches = arrow::util::pretty::pretty_format_batches(&[batch])
///   .unwrap()
///   .to_string();
/// let lines = formatted_batches.lines().collect::<Vec<_>>();
/// assert_eq!(lines, vec![
///   "+------------+------------+-------------+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
///   "| ps_partkey | ps_suppkey | ps_availqty | ps_supplycost | ps_comment                                                                                                                                                                                         |",
///   "+------------+------------+-------------+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+",
///   "| 1          | 2          | 3325        | 771.64        | , even theodolites. regular, final theodolites eat after the carefully pending foxes. furiously regular deposits sleep slyly. carefully bold realms above the ironic dependencies haggle careful   |",
///   "| 1          | 2502       | 8076        | 993.49        | ven ideas. quickly even packages print. pending multipliers must have to are fluff                                                                                                                 |",
///   "| 1          | 5002       | 3956        | 337.09        | after the fluffily ironic deposits? blithely special dependencies integrate furiously even excuses. blithely silent theodolites could have to haggle pending, express requests; fu                 |",
///   "| 1          | 7502       | 4069        | 357.84        | al, regular dependencies serve carefully after the quickly final pinto beans. furiously even deposits sleep quickly final, silent pinto beans. fluffily reg                                        |",
///   "| 2          | 3          | 8895        | 378.49        | nic accounts. final accounts sleep furiously about the ironic, bold packages. regular, regular accounts                                                                                            |",
///   "| 2          | 2503       | 4969        | 915.27        | ptotes. quickly pending dependencies integrate furiously. fluffily ironic ideas impress blithely above the express accounts. furiously even epitaphs need to wak                                   |",
///   "| 2          | 5003       | 8539        | 438.37        | blithely bold ideas. furiously stealthy packages sleep fluffily. slyly special deposits snooze furiously carefully regular accounts. regular deposits according to the accounts nag carefully slyl |",
///   "| 2          | 7503       | 3025        | 306.39        | olites. deposits wake carefully. even, express requests cajole. carefully regular ex                                                                                                               |",
///   "| 3          | 4          | 4651        | 920.92        | ilent foxes affix furiously quickly unusual requests. even packages across the carefully even theodolites nag above the sp                                                                         |",
///   "| 3          | 2504       | 4093        | 498.13        | ending dependencies haggle fluffily. regular deposits boost quickly carefully regular requests. deposits affix furiously around the pinto beans. ironic, unusual platelets across the p            |",
///   "+------------+------------+-------------+---------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+"
/// ]);
/// ```
pub struct PartSuppArrow {
    inner: PartSuppGeneratorIterator<'static>,
    batch_size: usize,
}

impl PartSuppArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&PARTSUPP_SCHEMA)
    }

    pub fn new(generator: PartSuppGenerator<'static>) -> Self {
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

impl RecordBatchReader for PartSuppArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for PartSuppArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get next rows to convert
        let rows: Vec<_> = self.inner.by_ref().take(self.batch_size).collect();
        if rows.is_empty() {
            return None;
        }

        let ps_partkey = Int64Array::from_iter_values(rows.iter().map(|r| r.ps_partkey));
        let ps_suppkey = Int64Array::from_iter_values(rows.iter().map(|r| r.ps_suppkey));
        let ps_availqty = Int32Array::from_iter_values(rows.iter().map(|r| r.ps_availqty));
        let ps_supplycost = decimal128_array_from_iter(rows.iter().map(|r| r.ps_supplycost));
        let ps_comment = string_view_array_from_display_iter(rows.iter().map(|r| r.ps_comment));

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(ps_partkey),
                Arc::new(ps_suppkey),
                Arc::new(ps_availqty),
                Arc::new(ps_supplycost),
                Arc::new(ps_comment),
            ],
        );
        Some(batch)
    }
}

/// Schema for the PartSupp
static PARTSUPP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_partsupp_schema);
fn make_partsupp_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ps_partkey", DataType::Int64, false),
        Field::new("ps_suppkey", DataType::Int64, false),
        Field::new("ps_availqty", DataType::Int32, false),
        Field::new("ps_supplycost", DataType::Decimal128(15, 2), false),
        Field::new("ps_comment", DataType::Utf8View, false),
    ]))
}
