use crate::conversions::{
    bool_to_yn, decimal_to_i128, opt, sk_opt, string_view_array_from_opt_iter,
};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
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

        let mut p_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut p_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_start: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut p_end: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut p_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut p_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut p_response: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut p_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_dmail: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_email: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_catalog: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_tv: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_radio: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_press: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_event: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_demo: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_details: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_purpose: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut p_active: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            p_sk.push(sk_opt(nbm, 0, r.get_p_promo_sk()));
            p_id.push(opt(nbm, 1, r.get_p_promo_id().to_owned()));
            p_start.push(sk_opt(nbm, 2, r.get_p_start_date_id()));
            p_end.push(sk_opt(nbm, 3, r.get_p_end_date_id()));
            p_item.push(sk_opt(nbm, 4, r.get_p_item_sk()));
            p_cost.push(opt(nbm, 5, decimal_to_i128(r.get_p_cost())));
            p_response.push(opt(nbm, 6, r.get_p_response_target()));
            p_name.push(opt(nbm, 7, r.get_p_promo_name().to_owned()));
            p_dmail.push(opt(nbm, 8, bool_to_yn(r.get_p_channel_dmail()).to_owned()));
            p_email.push(opt(nbm, 9, bool_to_yn(r.get_p_channel_email()).to_owned()));
            p_catalog.push(opt(
                nbm,
                10,
                bool_to_yn(r.get_p_channel_catalog()).to_owned(),
            ));
            p_tv.push(opt(nbm, 11, bool_to_yn(r.get_p_channel_tv()).to_owned()));
            p_radio.push(opt(nbm, 12, bool_to_yn(r.get_p_channel_radio()).to_owned()));
            p_press.push(opt(nbm, 13, bool_to_yn(r.get_p_channel_press()).to_owned()));
            p_event.push(opt(nbm, 14, bool_to_yn(r.get_p_channel_event()).to_owned()));
            p_demo.push(opt(nbm, 15, bool_to_yn(r.get_p_channel_demo()).to_owned()));
            p_details.push(opt(nbm, 16, r.get_p_channel_details().to_owned()));
            p_purpose.push(opt(nbm, 17, r.get_p_purpose().to_owned()));
            p_active.push(opt(
                nbm,
                18,
                bool_to_yn(r.get_p_discount_active()).to_owned(),
            ));
        }

        let cost_arr = Decimal128Array::from(p_cost)
            .with_precision_and_scale(38, 2)
            .unwrap();
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(p_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    p_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(p_start)),
                Arc::new(Int64Array::from(p_end)),
                Arc::new(Int64Array::from(p_item)),
                Arc::new(cost_arr),
                Arc::new(Int32Array::from(p_response)),
                Arc::new(string_view_array_from_opt_iter(
                    p_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_dmail.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_email.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_catalog.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_tv.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_radio.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_press.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_event.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_demo.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_details.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_purpose.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    p_active.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("p_promo_sk", DataType::Int64, true),
        Field::new("p_promo_id", DataType::Utf8View, true),
        Field::new("p_start_date_sk", DataType::Int64, true),
        Field::new("p_end_date_sk", DataType::Int64, true),
        Field::new("p_item_sk", DataType::Int64, true),
        Field::new("p_cost", DataType::Decimal128(38, 2), true),
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
