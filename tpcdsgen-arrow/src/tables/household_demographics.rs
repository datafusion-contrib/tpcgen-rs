use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, HouseholdDemographicsRowGenerator};

pub struct HouseholdDemographicsArrow {
    inner: RowIter<HouseholdDemographicsRowGenerator>,
    batch_size: usize,
}

impl HouseholdDemographicsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session
            .get_scaling()
            .get_row_count(Table::HouseholdDemographics);
        Self {
            inner: RowIter::new(HouseholdDemographicsRowGenerator::new(), session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: i64) {
        self.inner
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate only source rows `starting_row_number..=ending_row_number`
    /// (1-based, inclusive). The ending row number is clamped to the table's
    /// row count.
    pub fn with_source_row_range(
        mut self,
        starting_row_number: i64,
        ending_row_number: i64,
    ) -> Self {
        self.inner
            .set_source_row_range(starting_row_number, ending_row_number);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchReader for HouseholdDemographicsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for HouseholdDemographicsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::HouseholdDemographics(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut demo_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut income_band_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut buy_potential: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut dep_count: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut vehicle_count: Vec<Option<i32>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            demo_sk.push(sk_opt(nbm, 0, r.get_hd_demo_sk()));
            income_band_sk.push(sk_opt(nbm, 1, r.get_hd_income_band_sk()));
            buy_potential.push(opt(nbm, 2, r.get_hd_buy_potential().to_owned()));
            dep_count.push(opt(nbm, 3, r.get_hd_dep_count()));
            vehicle_count.push(opt(nbm, 4, r.get_hd_vehicle_count()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(demo_sk)),
                Arc::new(Int64Array::from(income_band_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    buy_potential.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(dep_count)),
                Arc::new(Int32Array::from(vehicle_count)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("hd_demo_sk", DataType::Int64, true),
        Field::new("hd_income_band_sk", DataType::Int64, true),
        Field::new("hd_buy_potential", DataType::Utf8View, true),
        Field::new("hd_dep_count", DataType::Int32, true),
        Field::new("hd_vehicle_count", DataType::Int32, true),
    ]))
}
