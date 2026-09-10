use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogSalesRowGenerator, GeneratedRow};

pub struct CatalogSalesArrow {
    inner: RowIter<CatalogSalesRowGenerator>,
    batch_size: usize,
}

impl CatalogSalesArrow {
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

impl RecordBatchReader for CatalogSalesArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for CatalogSalesArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .filter_map(|g| {
                if let GeneratedRow::CatalogSales(r) = g {
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

        let mut cs_sold_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_sold_time: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_bill_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_bill_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_bill_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_bill_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_customer: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_cdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_hdemo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_addr: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_call_center: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_catalog_page: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_ship_mode: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_warehouse: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_promo: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_order_number: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cs_quantity: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cs_wholesale_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_list_price: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_sales_price: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_discount_amt: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_sales_price: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_wholesale_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_list_price: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_coupon_amt: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_ext_ship_cost: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_net_paid: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_net_paid_inc_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_net_paid_inc_ship: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_net_paid_inc_ship_tax: Vec<Option<i128>> = Vec::with_capacity(rows.len());
        let mut cs_net_profit: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            let p = r.get_cs_pricing();
            cs_sold_date.push(sk_opt(nbm, 0, r.get_cs_sold_date_sk()));
            cs_sold_time.push(sk_opt(nbm, 1, r.get_cs_sold_time_sk()));
            cs_ship_date.push(sk_opt(nbm, 2, r.get_cs_ship_date_sk()));
            cs_bill_customer.push(sk_opt(nbm, 3, r.get_cs_bill_customer_sk()));
            cs_bill_cdemo.push(sk_opt(nbm, 4, r.get_cs_bill_cdemo_sk()));
            cs_bill_hdemo.push(sk_opt(nbm, 5, r.get_cs_bill_hdemo_sk()));
            cs_bill_addr.push(sk_opt(nbm, 6, r.get_cs_bill_addr_sk()));
            cs_ship_customer.push(sk_opt(nbm, 7, r.get_cs_ship_customer_sk()));
            cs_ship_cdemo.push(sk_opt(nbm, 8, r.get_cs_ship_cdemo_sk()));
            cs_ship_hdemo.push(sk_opt(nbm, 9, r.get_cs_ship_hdemo_sk()));
            cs_ship_addr.push(sk_opt(nbm, 10, r.get_cs_ship_addr_sk()));
            cs_call_center.push(sk_opt(nbm, 11, r.get_cs_call_center_sk()));
            cs_catalog_page.push(sk_opt(nbm, 12, r.get_cs_catalog_page_sk()));
            cs_ship_mode.push(sk_opt(nbm, 13, r.get_cs_ship_mode_sk()));
            cs_warehouse.push(sk_opt(nbm, 14, r.get_cs_warehouse_sk()));
            cs_item.push(sk_opt(nbm, 15, r.get_cs_sold_item_sk()));
            cs_promo.push(sk_opt(nbm, 16, r.get_cs_promo_sk()));
            cs_order_number.push(opt(nbm, 17, r.get_cs_order_number()));
            cs_quantity.push(opt(nbm, 18, p.get_quantity()));
            cs_wholesale_cost.push(opt(nbm, 19, decimal_to_i128(p.get_wholesale_cost())));
            cs_list_price.push(opt(nbm, 20, decimal_to_i128(p.get_list_price())));
            cs_sales_price.push(opt(nbm, 21, decimal_to_i128(p.get_sales_price())));
            cs_ext_discount_amt.push(opt(nbm, 24, decimal_to_i128(p.get_ext_discount_amount())));
            cs_ext_sales_price.push(opt(nbm, 23, decimal_to_i128(p.get_ext_sales_price())));
            cs_ext_wholesale_cost.push(opt(nbm, 25, decimal_to_i128(p.get_ext_wholesale_cost())));
            cs_ext_list_price.push(opt(nbm, 26, decimal_to_i128(p.get_ext_list_price())));
            cs_ext_tax.push(opt(nbm, 27, decimal_to_i128(p.get_ext_tax())));
            cs_coupon_amt.push(opt(nbm, 22, decimal_to_i128(p.get_coupon_amount())));
            cs_ext_ship_cost.push(opt(nbm, 28, decimal_to_i128(p.get_ext_ship_cost())));
            cs_net_paid.push(opt(nbm, 29, decimal_to_i128(p.get_net_paid())));
            cs_net_paid_inc_tax.push(opt(
                nbm,
                30,
                decimal_to_i128(p.get_net_paid_including_tax()),
            ));
            cs_net_paid_inc_ship.push(opt(
                nbm,
                31,
                decimal_to_i128(p.get_net_paid_including_shipping()),
            ));
            cs_net_paid_inc_ship_tax.push(opt(
                nbm,
                32,
                decimal_to_i128(p.get_net_paid_including_shipping_and_tax()),
            ));
            cs_net_profit.push(opt(nbm, 33, decimal_to_i128(p.get_net_profit())));
        }

        let dec = |v: Vec<Option<i128>>| {
            Decimal128Array::from(v)
                .with_precision_and_scale(38, 2)
                .unwrap()
        };
        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(cs_sold_date)),
                Arc::new(Int64Array::from(cs_sold_time)),
                Arc::new(Int64Array::from(cs_ship_date)),
                Arc::new(Int64Array::from(cs_bill_customer)),
                Arc::new(Int64Array::from(cs_bill_cdemo)),
                Arc::new(Int64Array::from(cs_bill_hdemo)),
                Arc::new(Int64Array::from(cs_bill_addr)),
                Arc::new(Int64Array::from(cs_ship_customer)),
                Arc::new(Int64Array::from(cs_ship_cdemo)),
                Arc::new(Int64Array::from(cs_ship_hdemo)),
                Arc::new(Int64Array::from(cs_ship_addr)),
                Arc::new(Int64Array::from(cs_call_center)),
                Arc::new(Int64Array::from(cs_catalog_page)),
                Arc::new(Int64Array::from(cs_ship_mode)),
                Arc::new(Int64Array::from(cs_warehouse)),
                Arc::new(Int64Array::from(cs_item)),
                Arc::new(Int64Array::from(cs_promo)),
                Arc::new(Int64Array::from(cs_order_number)),
                Arc::new(Int32Array::from(cs_quantity)),
                Arc::new(dec(cs_wholesale_cost)),
                Arc::new(dec(cs_list_price)),
                Arc::new(dec(cs_sales_price)),
                Arc::new(dec(cs_ext_discount_amt)),
                Arc::new(dec(cs_ext_sales_price)),
                Arc::new(dec(cs_ext_wholesale_cost)),
                Arc::new(dec(cs_ext_list_price)),
                Arc::new(dec(cs_ext_tax)),
                Arc::new(dec(cs_coupon_amt)),
                Arc::new(dec(cs_ext_ship_cost)),
                Arc::new(dec(cs_net_paid)),
                Arc::new(dec(cs_net_paid_inc_tax)),
                Arc::new(dec(cs_net_paid_inc_ship)),
                Arc::new(dec(cs_net_paid_inc_ship_tax)),
                Arc::new(dec(cs_net_profit)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cs_sold_date_sk", DataType::Int64, true),
        Field::new("cs_sold_time_sk", DataType::Int64, true),
        Field::new("cs_ship_date_sk", DataType::Int64, true),
        Field::new("cs_bill_customer_sk", DataType::Int64, true),
        Field::new("cs_bill_cdemo_sk", DataType::Int64, true),
        Field::new("cs_bill_hdemo_sk", DataType::Int64, true),
        Field::new("cs_bill_addr_sk", DataType::Int64, true),
        Field::new("cs_ship_customer_sk", DataType::Int64, true),
        Field::new("cs_ship_cdemo_sk", DataType::Int64, true),
        Field::new("cs_ship_hdemo_sk", DataType::Int64, true),
        Field::new("cs_ship_addr_sk", DataType::Int64, true),
        Field::new("cs_call_center_sk", DataType::Int64, true),
        Field::new("cs_catalog_page_sk", DataType::Int64, true),
        Field::new("cs_ship_mode_sk", DataType::Int64, true),
        Field::new("cs_warehouse_sk", DataType::Int64, true),
        Field::new("cs_item_sk", DataType::Int64, true),
        Field::new("cs_promo_sk", DataType::Int64, true),
        Field::new("cs_order_number", DataType::Int64, true),
        Field::new("cs_quantity", DataType::Int32, true),
        Field::new("cs_wholesale_cost", DataType::Decimal128(38, 2), true),
        Field::new("cs_list_price", DataType::Decimal128(38, 2), true),
        Field::new("cs_sales_price", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_discount_amt", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_sales_price", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_wholesale_cost", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_list_price", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_tax", DataType::Decimal128(38, 2), true),
        Field::new("cs_coupon_amt", DataType::Decimal128(38, 2), true),
        Field::new("cs_ext_ship_cost", DataType::Decimal128(38, 2), true),
        Field::new("cs_net_paid", DataType::Decimal128(38, 2), true),
        Field::new("cs_net_paid_inc_tax", DataType::Decimal128(38, 2), true),
        Field::new("cs_net_paid_inc_ship", DataType::Decimal128(38, 2), true),
        Field::new(
            "cs_net_paid_inc_ship_tax",
            DataType::Decimal128(38, 2),
            true,
        ),
        Field::new("cs_net_profit", DataType::Decimal128(38, 2), true),
    ]))
}
