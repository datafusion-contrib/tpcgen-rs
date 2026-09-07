use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, WebSalesRowGenerator};

pub struct WebReturnsArrow {
    inner: RowIter<WebSalesRowGenerator>,
    batch_size: usize,
}

impl WebReturnsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSales);
        Self {
            inner: RowIter::new(WebSalesRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for WebReturnsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WebReturnsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .filter_map(|g| {
                if let GeneratedRow::WebReturns(r) = g {
                    Some(r)
                } else {
                    None
                }
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut wr_returned_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_returned_time: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_refunded_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_refunded_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_refunded_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_refunded_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_returning_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_returning_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_returning_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_returning_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_web_page: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_reason: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_order_number: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut wr_quantity: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut wr_return_amt: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_return_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_return_amt_inc_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_fee: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_return_ship_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_refunded_cash: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_reversed_charge: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_store_credit: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut wr_net_loss: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            let p = r.get_wr_pricing();
            wr_returned_date.push(sk_opt(nbm, 0, r.get_wr_returned_date_sk()));
            wr_returned_time.push(sk_opt(nbm, 1, r.get_wr_returned_time_sk()));
            wr_item.push(sk_opt(nbm, 2, r.get_wr_item_sk()));
            wr_refunded_customer.push(sk_opt(nbm, 3, r.get_wr_refunded_customer_sk()));
            wr_refunded_cdemo.push(sk_opt(nbm, 4, r.get_wr_refunded_cdemo_sk()));
            wr_refunded_hdemo.push(sk_opt(nbm, 5, r.get_wr_refunded_hdemo_sk()));
            wr_refunded_addr.push(sk_opt(nbm, 6, r.get_wr_refunded_addr_sk()));
            wr_returning_customer.push(sk_opt(nbm, 7, r.get_wr_returning_customer_sk()));
            wr_returning_cdemo.push(sk_opt(nbm, 8, r.get_wr_returning_cdemo_sk()));
            wr_returning_hdemo.push(sk_opt(nbm, 9, r.get_wr_returning_hdemo_sk()));
            wr_returning_addr.push(sk_opt(nbm, 10, r.get_wr_returning_addr_sk()));
            wr_web_page.push(sk_opt(nbm, 11, r.get_wr_web_page_sk()));
            wr_reason.push(sk_opt(nbm, 12, r.get_wr_reason_sk()));
            wr_order_number.push(opt(nbm, 13, r.get_wr_order_number()));
            wr_quantity.push(opt(nbm, 14, p.get_quantity()));
            wr_return_amt.push(opt(nbm, 15, decimal_to_i128(p.get_net_paid())));
            wr_return_tax.push(opt(nbm, 16, decimal_to_i128(p.get_ext_tax())));
            wr_return_amt_inc_tax.push(opt(
                nbm,
                17,
                decimal_to_i128(p.get_net_paid_including_tax()),
            ));
            wr_fee.push(opt(nbm, 18, decimal_to_i128(p.get_fee())));
            wr_return_ship_cost.push(opt(nbm, 19, decimal_to_i128(p.get_ext_ship_cost())));
            wr_refunded_cash.push(opt(nbm, 20, decimal_to_i128(p.get_refunded_cash())));
            wr_reversed_charge.push(opt(nbm, 21, decimal_to_i128(p.get_reversed_charge())));
            wr_store_credit.push(opt(nbm, 22, decimal_to_i128(p.get_store_credit())));
            wr_net_loss.push(opt(nbm, 23, decimal_to_i128(p.get_net_loss())));
        }

        let dec = |v: Vec<Option<i128>>| {
            Decimal128Array::from(v)
                .with_precision_and_scale(38, 2)
                .unwrap()
        };
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(wr_returned_date)),
                Arc::new(Int64Array::from(wr_returned_time)),
                Arc::new(Int64Array::from(wr_item)),
                Arc::new(Int64Array::from(wr_refunded_customer)),
                Arc::new(Int64Array::from(wr_refunded_cdemo)),
                Arc::new(Int64Array::from(wr_refunded_hdemo)),
                Arc::new(Int64Array::from(wr_refunded_addr)),
                Arc::new(Int64Array::from(wr_returning_customer)),
                Arc::new(Int64Array::from(wr_returning_cdemo)),
                Arc::new(Int64Array::from(wr_returning_hdemo)),
                Arc::new(Int64Array::from(wr_returning_addr)),
                Arc::new(Int64Array::from(wr_web_page)),
                Arc::new(Int64Array::from(wr_reason)),
                Arc::new(Int64Array::from(wr_order_number)),
                Arc::new(Int32Array::from(wr_quantity)),
                Arc::new(dec(wr_return_amt)),
                Arc::new(dec(wr_return_tax)),
                Arc::new(dec(wr_return_amt_inc_tax)),
                Arc::new(dec(wr_fee)),
                Arc::new(dec(wr_return_ship_cost)),
                Arc::new(dec(wr_refunded_cash)),
                Arc::new(dec(wr_reversed_charge)),
                Arc::new(dec(wr_store_credit)),
                Arc::new(dec(wr_net_loss)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("wr_returned_date_sk", DataType::Int64, true),
        Field::new("wr_returned_time_sk", DataType::Int64, true),
        Field::new("wr_item_sk", DataType::Int64, true),
        Field::new("wr_refunded_customer_sk", DataType::Int64, true),
        Field::new("wr_refunded_cdemo_sk", DataType::Int64, true),
        Field::new("wr_refunded_hdemo_sk", DataType::Int64, true),
        Field::new("wr_refunded_addr_sk", DataType::Int64, true),
        Field::new("wr_returning_customer_sk", DataType::Int64, true),
        Field::new("wr_returning_cdemo_sk", DataType::Int64, true),
        Field::new("wr_returning_hdemo_sk", DataType::Int64, true),
        Field::new("wr_returning_addr_sk", DataType::Int64, true),
        Field::new("wr_web_page_sk", DataType::Int64, true),
        Field::new("wr_reason_sk", DataType::Int64, true),
        Field::new("wr_order_number", DataType::Int64, true),
        Field::new("wr_return_quantity", DataType::Int32, true),
        Field::new("wr_return_amt", DataType::Decimal128(38, 2), true),
        Field::new("wr_return_tax", DataType::Decimal128(38, 2), true),
        Field::new("wr_return_amt_inc_tax", DataType::Decimal128(38, 2), true),
        Field::new("wr_fee", DataType::Decimal128(38, 2), true),
        Field::new("wr_return_ship_cost", DataType::Decimal128(38, 2), true),
        Field::new("wr_refunded_cash", DataType::Decimal128(38, 2), true),
        Field::new("wr_reversed_charge", DataType::Decimal128(38, 2), true),
        Field::new("wr_store_credit", DataType::Decimal128(38, 2), true),
        Field::new("wr_net_loss", DataType::Decimal128(38, 2), true),
    ]))
}
