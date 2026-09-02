use crate::conversions::{
    address_columns, decimal_to_i128, julian_to_date32, opt, sk_opt,
    string_view_array_from_opt_iter,
};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Date32Array, Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, WebSiteRowGenerator};

pub struct WebSiteArrow {
    inner: RowIter<WebSiteRowGenerator>,
    batch_size: usize,
}

impl WebSiteArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSite);
        Self {
            inner: RowIter::new(WebSiteRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for WebSiteArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for WebSiteArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::WebSite(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut web_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut web_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_rec_start: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut web_rec_end: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut web_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_open_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut web_close_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut web_class: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_market_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut web_market_class: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_market_desc: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_market_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut web_company_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut web_company_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut addr_rows: Vec<(tpcdsgen::types::Address, i64, u32)> =
            Vec::with_capacity(rows.len());
        let mut web_tax_pct: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            web_sk.push(sk_opt(nbm, 0, r.get_web_site_sk()));
            web_id.push(opt(nbm, 1, r.get_web_site_id().to_owned()));
            web_rec_start.push(julian_to_date32(r.get_web_rec_start_date_id()));
            web_rec_end.push(julian_to_date32(r.get_web_rec_end_date_id()));
            web_name.push(opt(nbm, 4, r.web_name().to_owned()));
            web_open_date.push(sk_opt(nbm, 5, r.web_open_date()));
            web_close_date.push(sk_opt(nbm, 6, r.web_close_date()));
            web_class.push(opt(nbm, 7, r.web_class().to_owned()));
            web_manager.push(opt(nbm, 8, r.web_manager().to_owned()));
            web_market_id.push(opt(nbm, 9, r.web_market_id()));
            web_market_class.push(opt(nbm, 10, r.web_market_class().to_owned()));
            web_market_desc.push(opt(nbm, 11, r.web_market_desc().to_owned()));
            web_market_manager.push(opt(nbm, 12, r.web_market_manager().to_owned()));
            web_company_id.push(opt(nbm, 13, r.web_company_id()));
            web_company_name.push(opt(nbm, 14, r.web_company_name().to_owned()));
            addr_rows.push((r.web_address().clone(), nbm, 15));
            web_tax_pct.push(opt(nbm, 25, decimal_to_i128(*r.web_tax_percentage())));
        }

        let (
            street_number,
            street_name,
            street_type,
            suite_number,
            city,
            county,
            state,
            zip,
            country,
            gmt_offset,
        ) = address_columns(addr_rows.iter().map(|(a, nbm, base)| (a, *nbm, *base)));

        let tax_arr = Decimal128Array::from(web_tax_pct)
            .with_precision_and_scale(38, 2)
            .unwrap();

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(web_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    web_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Date32Array::from(web_rec_start)),
                Arc::new(Date32Array::from(web_rec_end)),
                Arc::new(string_view_array_from_opt_iter(
                    web_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(web_open_date)),
                Arc::new(Int64Array::from(web_close_date)),
                Arc::new(string_view_array_from_opt_iter(
                    web_class.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    web_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(web_market_id)),
                Arc::new(string_view_array_from_opt_iter(
                    web_market_class.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    web_market_desc.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    web_market_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(web_company_id)),
                Arc::new(string_view_array_from_opt_iter(
                    web_company_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(street_number),
                Arc::new(street_name),
                Arc::new(street_type),
                Arc::new(suite_number),
                Arc::new(city),
                Arc::new(county),
                Arc::new(state),
                Arc::new(zip),
                Arc::new(country),
                Arc::new(gmt_offset),
                Arc::new(tax_arr),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("web_site_sk", DataType::Int64, true),
        Field::new("web_site_id", DataType::Utf8View, true),
        Field::new("web_rec_start_date", DataType::Date32, true),
        Field::new("web_rec_end_date", DataType::Date32, true),
        Field::new("web_name", DataType::Utf8View, true),
        Field::new("web_open_date_sk", DataType::Int64, true),
        Field::new("web_close_date_sk", DataType::Int64, true),
        Field::new("web_class", DataType::Utf8View, true),
        Field::new("web_manager", DataType::Utf8View, true),
        Field::new("web_mkt_id", DataType::Int32, true),
        Field::new("web_mkt_class", DataType::Utf8View, true),
        Field::new("web_mkt_desc", DataType::Utf8View, true),
        Field::new("web_market_manager", DataType::Utf8View, true),
        Field::new("web_company_id", DataType::Int32, true),
        Field::new("web_company_name", DataType::Utf8View, true),
        Field::new("web_street_number", DataType::Int32, true),
        Field::new("web_street_name", DataType::Utf8View, true),
        Field::new("web_street_type", DataType::Utf8View, true),
        Field::new("web_suite_number", DataType::Utf8View, true),
        Field::new("web_city", DataType::Utf8View, true),
        Field::new("web_county", DataType::Utf8View, true),
        Field::new("web_state", DataType::Utf8View, true),
        Field::new("web_zip", DataType::Utf8View, true),
        Field::new("web_country", DataType::Utf8View, true),
        Field::new("web_gmt_offset", DataType::Int32, true),
        Field::new("web_tax_percentage", DataType::Decimal128(38, 2), true),
    ]))
}
