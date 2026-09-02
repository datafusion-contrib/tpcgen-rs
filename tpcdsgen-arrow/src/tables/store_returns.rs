use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, StoreSalesRowGenerator};

pub struct StoreReturnsArrow {
    inner: RowIter<StoreSalesRowGenerator>,
    batch_size: usize,
}

impl StoreReturnsArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::StoreSales);
        Self {
            inner: RowIter::new(StoreSalesRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for StoreReturnsArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for StoreReturnsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .filter_map(|g| {
                if let GeneratedRow::StoreReturns(r) = g {
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

        let mut sr_returned_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_returned_time: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_store: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_reason: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_ticket: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut sr_quantity: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut sr_return_amt: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_return_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_return_amt_inc_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_fee: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_return_ship_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_refunded_cash: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_reversed_charge: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_store_credit: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut sr_net_loss: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            let p = r.get_sr_pricing();
            sr_returned_date.push(sk_opt(nbm, 0, r.get_sr_returned_date_sk()));
            sr_returned_time.push(sk_opt(nbm, 1, r.get_sr_returned_time_sk()));
            sr_item.push(sk_opt(nbm, 2, r.get_sr_item_sk()));
            sr_customer.push(sk_opt(nbm, 3, r.get_sr_customer_sk()));
            sr_cdemo.push(sk_opt(nbm, 4, r.get_sr_cdemo_sk()));
            sr_hdemo.push(sk_opt(nbm, 5, r.get_sr_hdemo_sk()));
            sr_addr.push(sk_opt(nbm, 6, r.get_sr_addr_sk()));
            sr_store.push(sk_opt(nbm, 7, r.get_sr_store_sk()));
            sr_reason.push(sk_opt(nbm, 8, r.get_sr_reason_sk()));
            sr_ticket.push(sk_opt(nbm, 9, r.get_sr_ticket_number()));
            sr_quantity.push(opt(nbm, 10, p.get_quantity()));
            sr_return_amt.push(opt(nbm, 11, decimal_to_i128(p.get_net_paid())));
            sr_return_tax.push(opt(nbm, 12, decimal_to_i128(p.get_ext_tax())));
            sr_return_amt_inc_tax.push(opt(
                nbm,
                13,
                decimal_to_i128(p.get_net_paid_including_tax()),
            ));
            sr_fee.push(opt(nbm, 14, decimal_to_i128(p.get_fee())));
            sr_return_ship_cost.push(opt(nbm, 15, decimal_to_i128(p.get_ext_ship_cost())));
            sr_refunded_cash.push(opt(nbm, 16, decimal_to_i128(p.get_refunded_cash())));
            sr_reversed_charge.push(opt(nbm, 17, decimal_to_i128(p.get_reversed_charge())));
            sr_store_credit.push(opt(nbm, 18, decimal_to_i128(p.get_store_credit())));
            sr_net_loss.push(opt(nbm, 19, decimal_to_i128(p.get_net_loss())));
        }

        let dec = |v: Vec<Option<i128>>| {
            Decimal128Array::from(v)
                .with_precision_and_scale(38, 2)
                .unwrap()
        };
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(sr_returned_date)),
                Arc::new(Int64Array::from(sr_returned_time)),
                Arc::new(Int64Array::from(sr_item)),
                Arc::new(Int64Array::from(sr_customer)),
                Arc::new(Int64Array::from(sr_cdemo)),
                Arc::new(Int64Array::from(sr_hdemo)),
                Arc::new(Int64Array::from(sr_addr)),
                Arc::new(Int64Array::from(sr_store)),
                Arc::new(Int64Array::from(sr_reason)),
                Arc::new(Int64Array::from(sr_ticket)),
                Arc::new(Int32Array::from(sr_quantity)),
                Arc::new(dec(sr_return_amt)),
                Arc::new(dec(sr_return_tax)),
                Arc::new(dec(sr_return_amt_inc_tax)),
                Arc::new(dec(sr_fee)),
                Arc::new(dec(sr_return_ship_cost)),
                Arc::new(dec(sr_refunded_cash)),
                Arc::new(dec(sr_reversed_charge)),
                Arc::new(dec(sr_store_credit)),
                Arc::new(dec(sr_net_loss)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sr_returned_date_sk", DataType::Int64, true),
        Field::new("sr_return_time_sk", DataType::Int64, true),
        Field::new("sr_item_sk", DataType::Int64, true),
        Field::new("sr_customer_sk", DataType::Int64, true),
        Field::new("sr_cdemo_sk", DataType::Int64, true),
        Field::new("sr_hdemo_sk", DataType::Int64, true),
        Field::new("sr_addr_sk", DataType::Int64, true),
        Field::new("sr_store_sk", DataType::Int64, true),
        Field::new("sr_reason_sk", DataType::Int64, true),
        Field::new("sr_ticket_number", DataType::Int64, true),
        Field::new("sr_return_quantity", DataType::Int32, true),
        Field::new("sr_return_amt", DataType::Decimal128(38, 2), true),
        Field::new("sr_return_tax", DataType::Decimal128(38, 2), true),
        Field::new("sr_return_amt_inc_tax", DataType::Decimal128(38, 2), true),
        Field::new("sr_fee", DataType::Decimal128(38, 2), true),
        Field::new("sr_return_ship_cost", DataType::Decimal128(38, 2), true),
        Field::new("sr_refunded_cash", DataType::Decimal128(38, 2), true),
        Field::new("sr_reversed_charge", DataType::Decimal128(38, 2), true),
        Field::new("sr_store_credit", DataType::Decimal128(38, 2), true),
        Field::new("sr_net_loss", DataType::Decimal128(38, 2), true),
    ]))
}
