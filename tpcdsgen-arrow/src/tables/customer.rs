use crate::conversions::{bool_to_yn, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerRowGenerator, GeneratedRow};

pub struct CustomerArrow {
    inner: RowIter<CustomerRowGenerator>,
    batch_size: usize,
}

impl CustomerArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Customer);
        Self {
            inner: RowIter::new(CustomerRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CustomerArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for CustomerArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Customer(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut c_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut c_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_cdemo_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut c_hdemo_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut c_addr_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut c_shipto_date: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut c_sales_date: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut c_salutation: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_first_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_last_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_pref_flag: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_birth_day: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut c_birth_month: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut c_birth_year: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut c_birth_country: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_login: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_email: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut c_last_review: Vec<Option<i32>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            c_sk.push(sk_opt(nbm, 0, r.get_c_customer_sk()));
            c_id.push(opt(nbm, 1, r.get_c_customer_id().to_owned()));
            c_cdemo_sk.push(sk_opt(nbm, 2, r.get_c_current_cdemo_sk()));
            c_hdemo_sk.push(sk_opt(nbm, 3, r.get_c_current_hdemo_sk()));
            c_addr_sk.push(sk_opt(nbm, 4, r.get_c_current_addr_sk()));
            c_shipto_date.push(opt(nbm, 5, r.get_c_first_shipto_date_id()));
            c_sales_date.push(opt(nbm, 6, r.get_c_first_sales_date_id()));
            c_salutation.push(opt(nbm, 7, r.get_c_salutation().to_owned()));
            c_first_name.push(opt(nbm, 8, r.get_c_first_name().to_owned()));
            c_last_name.push(opt(nbm, 9, r.get_c_last_name().to_owned()));
            c_pref_flag.push(opt(
                nbm,
                10,
                bool_to_yn(r.get_c_preferred_cust_flag()).to_owned(),
            ));
            c_birth_day.push(opt(nbm, 11, r.get_c_birth_day()));
            c_birth_month.push(opt(nbm, 12, r.get_c_birth_month()));
            c_birth_year.push(opt(nbm, 13, r.get_c_birth_year()));
            c_birth_country.push(opt(nbm, 14, r.get_c_birth_country().to_owned()));
            c_login.push(None); // always null per TPC-DS spec
            c_email.push(opt(nbm, 16, r.get_c_email_address().to_owned()));
            c_last_review.push(opt(nbm, 17, r.get_c_last_review_date()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(c_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    c_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(c_cdemo_sk)),
                Arc::new(Int64Array::from(c_hdemo_sk)),
                Arc::new(Int64Array::from(c_addr_sk)),
                Arc::new(Int32Array::from(c_shipto_date)),
                Arc::new(Int32Array::from(c_sales_date)),
                Arc::new(string_view_array_from_opt_iter(
                    c_salutation.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    c_first_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    c_last_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    c_pref_flag.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(c_birth_day)),
                Arc::new(Int32Array::from(c_birth_month)),
                Arc::new(Int32Array::from(c_birth_year)),
                Arc::new(string_view_array_from_opt_iter(
                    c_birth_country.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    c_login.iter().map(|_| None::<&str>),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    c_email.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(c_last_review)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("c_customer_sk", DataType::Int64, true),
        Field::new("c_customer_id", DataType::Utf8View, true),
        Field::new("c_current_cdemo_sk", DataType::Int64, true),
        Field::new("c_current_hdemo_sk", DataType::Int64, true),
        Field::new("c_current_addr_sk", DataType::Int64, true),
        Field::new("c_first_shipto_date_sk", DataType::Int32, true),
        Field::new("c_first_sales_date_sk", DataType::Int32, true),
        Field::new("c_salutation", DataType::Utf8View, true),
        Field::new("c_first_name", DataType::Utf8View, true),
        Field::new("c_last_name", DataType::Utf8View, true),
        Field::new("c_preferred_cust_flag", DataType::Utf8View, true),
        Field::new("c_birth_day", DataType::Int32, true),
        Field::new("c_birth_month", DataType::Int32, true),
        Field::new("c_birth_year", DataType::Int32, true),
        Field::new("c_birth_country", DataType::Utf8View, true),
        Field::new("c_login", DataType::Utf8View, true),
        Field::new("c_email_address", DataType::Utf8View, true),
        Field::new("c_last_review_date_sk", DataType::Int32, true),
    ]))
}
