use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogSalesRowGenerator, GeneratedRow};

pub struct CatalogReturnsArrow {
    inner: RowIter<CatalogSalesRowGenerator>,
    batch_size: usize,
}

impl CatalogReturnsArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogSales);
        Self {
            inner: RowIter::new(CatalogSalesRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CatalogReturnsArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for CatalogReturnsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .filter_map(|g| {
                if let GeneratedRow::CatalogReturns(r) = g {
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

        let mut cr_returned_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_returned_time: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_refunded_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_refunded_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_refunded_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_refunded_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_returning_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_returning_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_returning_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_returning_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_call_center: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_catalog_page: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_ship_mode: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_warehouse: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_reason: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_order_number: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cr_quantity: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cr_return_amount: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_return_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_return_amount_inc_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_fee: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_return_ship_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_refunded_cash: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_reversed_charge: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_store_credit: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cr_net_loss: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            let p = r.get_cr_pricing();
            cr_returned_date.push(sk_opt(nbm, 0, r.get_cr_returned_date_sk()));
            cr_returned_time.push(sk_opt(nbm, 1, r.get_cr_returned_time_sk()));
            cr_item.push(sk_opt(nbm, 2, r.get_cr_item_sk()));
            cr_refunded_customer.push(sk_opt(nbm, 3, r.get_cr_refunded_customer_sk()));
            cr_refunded_cdemo.push(sk_opt(nbm, 4, r.get_cr_refunded_cdemo_sk()));
            cr_refunded_hdemo.push(sk_opt(nbm, 5, r.get_cr_refunded_hdemo_sk()));
            cr_refunded_addr.push(sk_opt(nbm, 6, r.get_cr_refunded_addr_sk()));
            cr_returning_customer.push(sk_opt(nbm, 7, r.get_cr_returning_customer_sk()));
            cr_returning_cdemo.push(sk_opt(nbm, 8, r.get_cr_returning_cdemo_sk()));
            cr_returning_hdemo.push(sk_opt(nbm, 9, r.get_cr_returning_hdemo_sk()));
            cr_returning_addr.push(sk_opt(nbm, 10, r.get_cr_returning_addr_sk()));
            cr_call_center.push(sk_opt(nbm, 11, r.get_cr_call_center_sk()));
            cr_catalog_page.push(sk_opt(nbm, 12, r.get_cr_catalog_page_sk()));
            cr_ship_mode.push(sk_opt(nbm, 13, r.get_cr_ship_mode_sk()));
            cr_warehouse.push(sk_opt(nbm, 14, r.get_cr_warehouse_sk()));
            cr_reason.push(sk_opt(nbm, 15, r.get_cr_reason_sk()));
            cr_order_number.push(opt(nbm, 16, r.get_cr_order_number()));
            cr_quantity.push(opt(nbm, 17, p.get_quantity()));
            cr_return_amount.push(opt(nbm, 18, decimal_to_i128(p.get_net_paid())));
            cr_return_tax.push(opt(nbm, 19, decimal_to_i128(p.get_ext_tax())));
            cr_return_amount_inc_tax.push(opt(
                nbm,
                20,
                decimal_to_i128(p.get_net_paid_including_tax()),
            ));
            cr_fee.push(opt(nbm, 21, decimal_to_i128(p.get_fee())));
            cr_return_ship_cost.push(opt(nbm, 22, decimal_to_i128(p.get_ext_ship_cost())));
            cr_refunded_cash.push(opt(nbm, 23, decimal_to_i128(p.get_refunded_cash())));
            cr_reversed_charge.push(opt(nbm, 24, decimal_to_i128(p.get_reversed_charge())));
            cr_store_credit.push(opt(nbm, 25, decimal_to_i128(p.get_store_credit())));
            cr_net_loss.push(opt(nbm, 26, decimal_to_i128(p.get_net_loss())));
        }

        let dec = |v: Vec<Option<i128>>| {
            Decimal128Array::from(v)
                .with_precision_and_scale(38, 2)
                .unwrap()
        };
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(cr_returned_date)),
                Arc::new(Int64Array::from(cr_returned_time)),
                Arc::new(Int64Array::from(cr_item)),
                Arc::new(Int64Array::from(cr_refunded_customer)),
                Arc::new(Int64Array::from(cr_refunded_cdemo)),
                Arc::new(Int64Array::from(cr_refunded_hdemo)),
                Arc::new(Int64Array::from(cr_refunded_addr)),
                Arc::new(Int64Array::from(cr_returning_customer)),
                Arc::new(Int64Array::from(cr_returning_cdemo)),
                Arc::new(Int64Array::from(cr_returning_hdemo)),
                Arc::new(Int64Array::from(cr_returning_addr)),
                Arc::new(Int64Array::from(cr_call_center)),
                Arc::new(Int64Array::from(cr_catalog_page)),
                Arc::new(Int64Array::from(cr_ship_mode)),
                Arc::new(Int64Array::from(cr_warehouse)),
                Arc::new(Int64Array::from(cr_reason)),
                Arc::new(Int64Array::from(cr_order_number)),
                Arc::new(Int32Array::from(cr_quantity)),
                Arc::new(dec(cr_return_amount)),
                Arc::new(dec(cr_return_tax)),
                Arc::new(dec(cr_return_amount_inc_tax)),
                Arc::new(dec(cr_fee)),
                Arc::new(dec(cr_return_ship_cost)),
                Arc::new(dec(cr_refunded_cash)),
                Arc::new(dec(cr_reversed_charge)),
                Arc::new(dec(cr_store_credit)),
                Arc::new(dec(cr_net_loss)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cr_returned_date_sk", DataType::Int64, true),
        Field::new("cr_returned_time_sk", DataType::Int64, true),
        Field::new("cr_item_sk", DataType::Int64, true),
        Field::new("cr_refunded_customer_sk", DataType::Int64, true),
        Field::new("cr_refunded_cdemo_sk", DataType::Int64, true),
        Field::new("cr_refunded_hdemo_sk", DataType::Int64, true),
        Field::new("cr_refunded_addr_sk", DataType::Int64, true),
        Field::new("cr_returning_customer_sk", DataType::Int64, true),
        Field::new("cr_returning_cdemo_sk", DataType::Int64, true),
        Field::new("cr_returning_hdemo_sk", DataType::Int64, true),
        Field::new("cr_returning_addr_sk", DataType::Int64, true),
        Field::new("cr_call_center_sk", DataType::Int64, true),
        Field::new("cr_catalog_page_sk", DataType::Int64, true),
        Field::new("cr_ship_mode_sk", DataType::Int64, true),
        Field::new("cr_warehouse_sk", DataType::Int64, true),
        Field::new("cr_reason_sk", DataType::Int64, true),
        Field::new("cr_order_number", DataType::Int64, true),
        Field::new("cr_return_quantity", DataType::Int32, true),
        Field::new("cr_return_amount", DataType::Decimal128(38, 2), true),
        Field::new("cr_return_tax", DataType::Decimal128(38, 2), true),
        Field::new(
            "cr_return_amount_inc_tax",
            DataType::Decimal128(38, 2),
            true,
        ),
        Field::new("cr_fee", DataType::Decimal128(38, 2), true),
        Field::new("cr_return_ship_cost", DataType::Decimal128(38, 2), true),
        Field::new("cr_refunded_cash", DataType::Decimal128(38, 2), true),
        Field::new("cr_reversed_charge", DataType::Decimal128(38, 2), true),
        Field::new("cr_store_credit", DataType::Decimal128(38, 2), true),
        Field::new("cr_net_loss", DataType::Decimal128(38, 2), true),
    ]))
}
