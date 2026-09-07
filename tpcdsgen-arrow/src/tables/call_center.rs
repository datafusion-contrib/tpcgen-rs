use crate::conversions::{
    address_columns, decimal_to_i128, is_null, julian_to_date32, opt, sk_opt,
    string_view_array_from_opt_iter,
};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Date32Array, Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CallCenterRowGenerator, GeneratedRow};

pub struct CallCenterArrow {
    inner: RowIter<CallCenterRowGenerator>,
    batch_size: usize,
}

impl CallCenterArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CallCenter);
        Self {
            inner: RowIter::new(CallCenterRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CallCenterArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CallCenterArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::CallCenter(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut cc_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cc_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_rec_start: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_rec_end: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_closed_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cc_open_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cc_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_class: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_employees: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_sq_ft: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_hours: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_market_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_market_class: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_market_desc: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_market_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_division_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_division_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cc_company: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cc_company_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut addr_rows: Vec<(tpcdsgen::types::Address, i64, u32)> =
            Vec::with_capacity(rows.len());
        let mut cc_tax_pct: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.get_null_bit_map();
            cc_sk.push(sk_opt(nbm, 0, r.get_cc_call_center_sk()));
            cc_id.push(opt(nbm, 1, r.get_cc_call_center_id().to_owned()));
            cc_rec_start.push(if is_null(nbm, 2) {
                None
            } else {
                julian_to_date32(r.get_cc_rec_start_date_id())
            });
            cc_rec_end.push(if is_null(nbm, 3) {
                None
            } else {
                julian_to_date32(r.get_cc_rec_end_date_id())
            });
            cc_closed_date.push(sk_opt(nbm, 4, r.get_cc_closed_date_id()));
            cc_open_date.push(sk_opt(nbm, 5, r.get_cc_open_date_id()));
            cc_name.push(opt(nbm, 6, r.get_cc_name().to_owned()));
            cc_class.push(opt(nbm, 7, r.get_cc_class().to_owned()));
            cc_employees.push(opt(nbm, 8, r.get_cc_employees()));
            cc_sq_ft.push(opt(nbm, 9, r.get_cc_sq_ft()));
            cc_hours.push(opt(nbm, 10, r.get_cc_hours().to_owned()));
            cc_manager.push(opt(nbm, 11, r.get_cc_manager().to_owned()));
            cc_market_id.push(opt(nbm, 12, r.get_cc_market_id()));
            cc_market_class.push(opt(nbm, 13, r.get_cc_market_class().to_owned()));
            cc_market_desc.push(opt(nbm, 14, r.get_cc_market_desc().to_owned()));
            cc_market_manager.push(opt(nbm, 15, r.get_cc_market_manager().to_owned()));
            cc_division_id.push(opt(nbm, 16, r.get_cc_division_id()));
            cc_division_name.push(opt(nbm, 17, r.get_cc_division_name().to_owned()));
            cc_company.push(opt(nbm, 18, r.get_cc_company()));
            cc_company_name.push(opt(nbm, 19, r.get_cc_company_name().to_owned()));
            addr_rows.push((r.get_cc_address().clone(), nbm, 20));
            cc_tax_pct.push(opt(nbm, 30, decimal_to_i128(*r.get_cc_tax_percentage())));
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

        let tax_arr = Decimal128Array::from(cc_tax_pct)
            .with_precision_and_scale(38, 2)
            .unwrap();

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(cc_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Date32Array::from(cc_rec_start)),
                Arc::new(Date32Array::from(cc_rec_end)),
                Arc::new(Int64Array::from(cc_closed_date)),
                Arc::new(Int64Array::from(cc_open_date)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    cc_class.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(cc_employees)),
                Arc::new(Int32Array::from(cc_sq_ft)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_hours.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    cc_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(cc_market_id)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_market_class.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    cc_market_desc.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    cc_market_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(cc_division_id)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_division_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(cc_company)),
                Arc::new(string_view_array_from_opt_iter(
                    cc_company_name.iter().map(|s| s.as_deref()),
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
        Field::new("cc_call_center_sk", DataType::Int64, true),
        Field::new("cc_call_center_id", DataType::Utf8View, true),
        Field::new("cc_rec_start_date", DataType::Date32, true),
        Field::new("cc_rec_end_date", DataType::Date32, true),
        Field::new("cc_closed_date_sk", DataType::Int64, true),
        Field::new("cc_open_date_sk", DataType::Int64, true),
        Field::new("cc_name", DataType::Utf8View, true),
        Field::new("cc_class", DataType::Utf8View, true),
        Field::new("cc_employees", DataType::Int32, true),
        Field::new("cc_sq_ft", DataType::Int32, true),
        Field::new("cc_hours", DataType::Utf8View, true),
        Field::new("cc_manager", DataType::Utf8View, true),
        Field::new("cc_mkt_id", DataType::Int32, true),
        Field::new("cc_mkt_class", DataType::Utf8View, true),
        Field::new("cc_mkt_desc", DataType::Utf8View, true),
        Field::new("cc_market_manager", DataType::Utf8View, true),
        Field::new("cc_division", DataType::Int32, true),
        Field::new("cc_division_name", DataType::Utf8View, true),
        Field::new("cc_company", DataType::Int32, true),
        Field::new("cc_company_name", DataType::Utf8View, true),
        Field::new("cc_street_number", DataType::Int32, true),
        Field::new("cc_street_name", DataType::Utf8View, true),
        Field::new("cc_street_type", DataType::Utf8View, true),
        Field::new("cc_suite_number", DataType::Utf8View, true),
        Field::new("cc_city", DataType::Utf8View, true),
        Field::new("cc_county", DataType::Utf8View, true),
        Field::new("cc_state", DataType::Utf8View, true),
        Field::new("cc_zip", DataType::Utf8View, true),
        Field::new("cc_country", DataType::Utf8View, true),
        Field::new("cc_gmt_offset", DataType::Int32, true),
        Field::new("cc_tax_percentage", DataType::Decimal128(38, 2), true),
    ]))
}
