use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, TimeDimRowGenerator};

pub struct TimeDimArrow {
    inner: RowIter<TimeDimRowGenerator>,
    batch_size: usize,
}

impl TimeDimArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::TimeDim);
        Self {
            inner: RowIter::new(TimeDimRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for TimeDimArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for TimeDimArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::TimeDim(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut t_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut t_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut t_time: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut t_hour: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut t_minute: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut t_second: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut t_am_pm: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut t_shift: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut t_sub_shift: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut t_meal_time: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            t_sk.push(sk_opt(nbm, 0, r.t_time_sk));
            t_id.push(opt(nbm, 1, r.t_time_id.clone()));
            t_time.push(opt(nbm, 2, r.t_time));
            t_hour.push(opt(nbm, 3, r.t_hour));
            t_minute.push(opt(nbm, 4, r.t_minute));
            t_second.push(opt(nbm, 5, r.t_second));
            t_am_pm.push(opt(nbm, 6, r.t_am_pm.clone()));
            t_shift.push(opt(nbm, 7, r.t_shift.clone()));
            t_sub_shift.push(opt(nbm, 8, r.t_sub_shift.clone()));
            // t_meal_time is an empty string (not null) for hours with no meal,
            // but the pipe-delimited format can't distinguish empty from null,
            // so we map empty -> None to match the .dat file convention.
            t_meal_time.push(if r.t_meal_time.is_empty() {
                None
            } else {
                Some(r.t_meal_time.clone())
            });
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(t_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    t_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(t_time)),
                Arc::new(Int32Array::from(t_hour)),
                Arc::new(Int32Array::from(t_minute)),
                Arc::new(Int32Array::from(t_second)),
                Arc::new(string_view_array_from_opt_iter(
                    t_am_pm.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    t_shift.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    t_sub_shift.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    t_meal_time.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("t_time_sk", DataType::Int64, false),
        Field::new("t_time_id", DataType::Utf8View, false),
        Field::new("t_time", DataType::Int32, false),
        Field::new("t_hour", DataType::Int32, false),
        Field::new("t_minute", DataType::Int32, false),
        Field::new("t_second", DataType::Int32, false),
        Field::new("t_am_pm", DataType::Utf8View, false),
        Field::new("t_shift", DataType::Utf8View, false),
        Field::new("t_sub_shift", DataType::Utf8View, false),
        Field::new("t_meal_time", DataType::Utf8View, true),
    ]))
}
