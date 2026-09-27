use crate::conversions::{
    bool_to_yn, decimal128_15_2_array, decimal_to_i128, integer_sk_opt, opt,
    string_view_array_from_opt_iter,
};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, PromotionRowGenerator};

pub struct PromotionArrow {
    inner: RowIter<PromotionRowGenerator>,
    batch_size: usize,
}

impl PromotionArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Promotion);
        Self {
            inner: RowIter::new(PromotionRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for PromotionArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for PromotionArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Promotion(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut p_sk: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_id: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_start: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_end: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_item: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut p_response: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_name: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_dmail: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_email: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_catalog: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_tv: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_radio: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_press: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_event: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_demo: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_details: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_purpose: Vec<Option<&str>> = Vec::with_capacity(rows.len());
        let mut p_active: Vec<Option<&str>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            p_sk.push(integer_sk_opt(nbm, 0, r.get_p_promo_sk()));
            p_id.push(opt(nbm, 1, r.get_p_promo_id()));
            p_start.push(integer_sk_opt(nbm, 2, r.get_p_start_date_id()));
            p_end.push(integer_sk_opt(nbm, 3, r.get_p_end_date_id()));
            p_item.push(integer_sk_opt(nbm, 4, r.get_p_item_sk()));
            p_cost.push(opt(nbm, 5, decimal_to_i128(r.get_p_cost())));
            p_response.push(opt(nbm, 6, r.get_p_response_target()));
            p_name.push(opt(nbm, 7, r.get_p_promo_name()));
            p_dmail.push(opt(nbm, 8, bool_to_yn(r.get_p_channel_dmail())));
            p_email.push(opt(nbm, 9, bool_to_yn(r.get_p_channel_email())));
            p_catalog.push(opt(nbm, 10, bool_to_yn(r.get_p_channel_catalog())));
            p_tv.push(opt(nbm, 11, bool_to_yn(r.get_p_channel_tv())));
            p_radio.push(opt(nbm, 12, bool_to_yn(r.get_p_channel_radio())));
            p_press.push(opt(nbm, 13, bool_to_yn(r.get_p_channel_press())));
            p_event.push(opt(nbm, 14, bool_to_yn(r.get_p_channel_event())));
            p_demo.push(opt(nbm, 15, bool_to_yn(r.get_p_channel_demo())));
            p_details.push(opt(nbm, 16, r.get_p_channel_details()));
            p_purpose.push(opt(nbm, 17, r.get_p_purpose()));
            p_active.push(opt(nbm, 18, bool_to_yn(r.get_p_discount_active())));
        }

        let cost_arr = decimal128_15_2_array(p_cost);
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int32Array::from(p_sk)),
                Arc::new(string_view_array_from_opt_iter(p_id.iter().copied())),
                Arc::new(Int32Array::from(p_start)),
                Arc::new(Int32Array::from(p_end)),
                Arc::new(Int32Array::from(p_item)),
                Arc::new(cost_arr),
                Arc::new(Int32Array::from(p_response)),
                Arc::new(string_view_array_from_opt_iter(p_name.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_dmail.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_email.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_catalog.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_tv.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_radio.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_press.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_event.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_demo.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_details.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_purpose.iter().copied())),
                Arc::new(string_view_array_from_opt_iter(p_active.iter().copied())),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("p_promo_sk", DataType::Int32, false),
        Field::new("p_promo_id", DataType::Utf8View, false),
        Field::new("p_start_date_sk", DataType::Int32, true),
        Field::new("p_end_date_sk", DataType::Int32, true),
        Field::new("p_item_sk", DataType::Int32, true),
        Field::new("p_cost", DataType::Decimal128(15, 2), true),
        Field::new("p_response_target", DataType::Int32, true),
        Field::new("p_promo_name", DataType::Utf8View, true),
        Field::new("p_channel_dmail", DataType::Utf8View, true),
        Field::new("p_channel_email", DataType::Utf8View, true),
        Field::new("p_channel_catalog", DataType::Utf8View, true),
        Field::new("p_channel_tv", DataType::Utf8View, true),
        Field::new("p_channel_radio", DataType::Utf8View, true),
        Field::new("p_channel_press", DataType::Utf8View, true),
        Field::new("p_channel_event", DataType::Utf8View, true),
        Field::new("p_channel_demo", DataType::Utf8View, true),
        Field::new("p_channel_details", DataType::Utf8View, true),
        Field::new("p_purpose", DataType::Utf8View, true),
        Field::new("p_discount_active", DataType::Utf8View, true),
    ]))
}
