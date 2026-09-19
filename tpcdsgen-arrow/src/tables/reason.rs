use crate::conversions::{integer_sk_opt, opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, ReasonRowGenerator};

pub struct ReasonArrow {
    inner: RowIter<ReasonRowGenerator>,
    batch_size: usize,
}

impl ReasonArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Reason);
        Self {
            inner: RowIter::new(ReasonRowGenerator::new(), session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.inner
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate only source rows `starting_row_number..=ending_row_number`
    /// (1-based, inclusive). The ending row number is clamped to the table's
    /// row count.
    pub fn with_source_row_range(
        mut self,
        starting_row_number: u64,
        ending_row_number: u64,
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

impl RecordBatchReader for ReasonArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for ReasonArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Reason(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut sk: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut id: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut reason_desc: Vec<Option<&str>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            sk.push(integer_sk_opt(nbm, 0, r.get_r_reason_sk()));
            id.push(opt(nbm, 1, r.get_r_reason_id()));
            reason_desc.push(opt(nbm, 2, r.get_r_reason_desc()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int32Array::from(sk)),
                Arc::new(string_view_array_from_opt_iter(id.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(reason_desc.iter().copied())),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("r_reason_sk", DataType::Int32, false),
        Field::new("r_reason_id", DataType::Utf8View, false),
        Field::new("r_reason_desc", DataType::Utf8View, true),
    ]))
}
