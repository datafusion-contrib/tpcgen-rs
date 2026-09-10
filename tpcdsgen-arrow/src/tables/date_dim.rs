use crate::conversions::{bool_to_yn, date_to_date32, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Date32Array, Int32Array, Int64Array, RecordBatch, StringViewBuilder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DateDimRowGenerator, GeneratedRow};

pub struct DateDimArrow {
    inner: RowIter<DateDimRowGenerator>,
    batch_size: usize,
}

impl DateDimArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DateDim);
        Self {
            inner: RowIter::new(DateDimRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for DateDimArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for DateDimArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::DateDim(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut d_date_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut d_date_id: Vec<String> = Vec::with_capacity(rows.len());
        let mut d_date: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_month_seq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_week_seq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_quarter_seq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_year: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_dow: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_moy: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_dom: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_qoy: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_fy_year: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_fy_quarter_seq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_fy_week_seq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_day_name: Vec<String> = Vec::with_capacity(rows.len());
        let mut d_quarter_name: Vec<String> = Vec::with_capacity(rows.len());
        let mut d_holiday: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_weekend: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_following_holiday: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_first_dom: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_last_dom: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_same_day_ly: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_same_day_lq: Vec<i32> = Vec::with_capacity(rows.len());
        let mut d_current_day: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_current_week: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_current_month: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_current_quarter: Vec<&'static str> = Vec::with_capacity(rows.len());
        let mut d_current_year: Vec<&'static str> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            d_date_sk.push(sk_opt(nbm, 0, r.d_date_sk));
            d_date_id.push(r.d_date_id.clone());
            d_date.push(date_to_date32(&r.d_date));
            d_month_seq.push(r.d_month_seq);
            d_week_seq.push(r.d_week_seq);
            d_quarter_seq.push(r.d_quarter_seq);
            d_year.push(r.d_year);
            d_dow.push(r.d_dow);
            d_moy.push(r.d_moy);
            d_dom.push(r.d_dom);
            d_qoy.push(r.d_qoy);
            d_fy_year.push(r.d_fy_year);
            d_fy_quarter_seq.push(r.d_fy_quarter_seq);
            d_fy_week_seq.push(r.d_fy_week_seq);
            d_day_name.push(r.d_day_name.clone());
            d_quarter_name.push(r.d_quarter_name.clone());
            d_holiday.push(bool_to_yn(r.d_holiday));
            d_weekend.push(bool_to_yn(r.d_weekend));
            d_following_holiday.push(bool_to_yn(r.d_following_holiday));
            d_first_dom.push(r.d_first_dom);
            d_last_dom.push(r.d_last_dom);
            d_same_day_ly.push(r.d_same_day_ly);
            d_same_day_lq.push(r.d_same_day_lq);
            d_current_day.push(bool_to_yn(r.d_current_day));
            d_current_week.push(bool_to_yn(r.d_current_week));
            d_current_month.push(bool_to_yn(r.d_current_month));
            d_current_quarter.push(bool_to_yn(r.d_current_quarter));
            d_current_year.push(bool_to_yn(r.d_current_year));
        }

        let mut id_b = StringViewBuilder::with_capacity(d_date_id.len());
        for s in &d_date_id {
            id_b.append_value(s);
        }
        let mut day_name_b = StringViewBuilder::with_capacity(d_day_name.len());
        for s in &d_day_name {
            day_name_b.append_value(s);
        }
        let mut quarter_name_b = StringViewBuilder::with_capacity(d_quarter_name.len());
        for s in &d_quarter_name {
            quarter_name_b.append_value(s);
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(d_date_sk)),
                Arc::new(id_b.finish()),
                Arc::new(Date32Array::from_iter_values(d_date)),
                Arc::new(Int32Array::from_iter_values(d_month_seq)),
                Arc::new(Int32Array::from_iter_values(d_week_seq)),
                Arc::new(Int32Array::from_iter_values(d_quarter_seq)),
                Arc::new(Int32Array::from_iter_values(d_year)),
                Arc::new(Int32Array::from_iter_values(d_dow)),
                Arc::new(Int32Array::from_iter_values(d_moy)),
                Arc::new(Int32Array::from_iter_values(d_dom)),
                Arc::new(Int32Array::from_iter_values(d_qoy)),
                Arc::new(Int32Array::from_iter_values(d_fy_year)),
                Arc::new(Int32Array::from_iter_values(d_fy_quarter_seq)),
                Arc::new(Int32Array::from_iter_values(d_fy_week_seq)),
                Arc::new(day_name_b.finish()),
                Arc::new(quarter_name_b.finish()),
                Arc::new(string_view_array_from_opt_iter(
                    d_holiday.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_weekend.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_following_holiday.iter().map(|s| Some(*s)),
                )),
                Arc::new(Int32Array::from_iter_values(d_first_dom)),
                Arc::new(Int32Array::from_iter_values(d_last_dom)),
                Arc::new(Int32Array::from_iter_values(d_same_day_ly)),
                Arc::new(Int32Array::from_iter_values(d_same_day_lq)),
                Arc::new(string_view_array_from_opt_iter(
                    d_current_day.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_current_week.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_current_month.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_current_quarter.iter().map(|s| Some(*s)),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    d_current_year.iter().map(|s| Some(*s)),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("d_date_sk", DataType::Int64, true),
        Field::new("d_date_id", DataType::Utf8View, false),
        Field::new("d_date", DataType::Date32, false),
        Field::new("d_month_seq", DataType::Int32, false),
        Field::new("d_week_seq", DataType::Int32, false),
        Field::new("d_quarter_seq", DataType::Int32, false),
        Field::new("d_year", DataType::Int32, false),
        Field::new("d_dow", DataType::Int32, false),
        Field::new("d_moy", DataType::Int32, false),
        Field::new("d_dom", DataType::Int32, false),
        Field::new("d_qoy", DataType::Int32, false),
        Field::new("d_fy_year", DataType::Int32, false),
        Field::new("d_fy_quarter_seq", DataType::Int32, false),
        Field::new("d_fy_week_seq", DataType::Int32, false),
        Field::new("d_day_name", DataType::Utf8View, false),
        Field::new("d_quarter_name", DataType::Utf8View, false),
        Field::new("d_holiday", DataType::Utf8View, false),
        Field::new("d_weekend", DataType::Utf8View, false),
        Field::new("d_following_holiday", DataType::Utf8View, false),
        Field::new("d_first_dom", DataType::Int32, false),
        Field::new("d_last_dom", DataType::Int32, false),
        Field::new("d_same_day_ly", DataType::Int32, false),
        Field::new("d_same_day_lq", DataType::Int32, false),
        Field::new("d_current_day", DataType::Utf8View, false),
        Field::new("d_current_week", DataType::Utf8View, false),
        Field::new("d_current_month", DataType::Utf8View, false),
        Field::new("d_current_quarter", DataType::Utf8View, false),
        Field::new("d_current_year", DataType::Utf8View, false),
    ]))
}
