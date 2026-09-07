use crate::conversions::opt;
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, IncomeBandRowGenerator};

pub struct IncomeBandArrow {
    inner: RowIter<IncomeBandRowGenerator>,
    batch_size: usize,
}

impl IncomeBandArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::IncomeBand);
        Self {
            inner: RowIter::new(IncomeBandRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for IncomeBandArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for IncomeBandArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::IncomeBand(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut band_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut lower: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut upper: Vec<Option<i32>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            band_id.push(opt(nbm, 0, r.get_ib_income_band_id()));
            lower.push(opt(nbm, 1, r.get_ib_lower_bound()));
            upper.push(opt(nbm, 2, r.get_ib_upper_bound()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int32Array::from(band_id)),
                Arc::new(Int32Array::from(lower)),
                Arc::new(Int32Array::from(upper)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ib_income_band_id", DataType::Int32, false),
        Field::new("ib_lower_bound", DataType::Int32, false),
        Field::new("ib_upper_bound", DataType::Int32, false),
    ]))
}
