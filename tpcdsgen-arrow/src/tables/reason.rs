use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int64Array, RecordBatch};
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

        let mut sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut desc: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            sk.push(sk_opt(nbm, 0, r.get_r_reason_sk()));
            id.push(opt(nbm, 1, r.get_r_reason_id().to_owned()));
            desc.push(opt(nbm, 2, r.get_r_reason_description().to_owned()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(sk)),
                Arc::new(string_view_array_from_opt_iter(
                    id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    desc.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("r_reason_sk", DataType::Int64, true),
        Field::new("r_reason_id", DataType::Utf8View, true),
        Field::new("r_reason_description", DataType::Utf8View, true),
    ]))
}
