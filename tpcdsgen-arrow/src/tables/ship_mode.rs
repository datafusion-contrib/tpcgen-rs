use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, ShipModeRowGenerator};

pub struct ShipModeArrow {
    inner: RowIter<ShipModeRowGenerator>,
    batch_size: usize,
}

impl ShipModeArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::ShipMode);
        Self {
            inner: RowIter::new(ShipModeRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for ShipModeArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for ShipModeArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::ShipMode(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut sm_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sm_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut sm_type: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut sm_code: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut sm_carrier: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut sm_contract: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            sm_sk.push(sk_opt(nbm, 0, r.get_sm_ship_mode_sk()));
            sm_id.push(opt(nbm, 1, r.get_sm_ship_mode_id().to_owned()));
            sm_type.push(opt(nbm, 2, r.get_sm_type().to_owned()));
            sm_code.push(opt(nbm, 3, r.get_sm_code().to_owned()));
            sm_carrier.push(opt(nbm, 4, r.get_sm_carrier().to_owned()));
            sm_contract.push(opt(nbm, 5, r.get_sm_contract().to_owned()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(sm_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    sm_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    sm_type.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    sm_code.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    sm_carrier.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    sm_contract.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sm_ship_mode_sk", DataType::Int64, true),
        Field::new("sm_ship_mode_id", DataType::Utf8View, true),
        Field::new("sm_type", DataType::Utf8View, true),
        Field::new("sm_code", DataType::Utf8View, true),
        Field::new("sm_carrier", DataType::Utf8View, true),
        Field::new("sm_contract", DataType::Utf8View, true),
    ]))
}
