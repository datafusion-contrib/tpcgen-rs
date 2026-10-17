use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogReturnsRow, CatalogReturnsRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`CatalogReturnsRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CatalogReturnsArrow {
    inner: CatalogReturnsRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CatalogReturnsRow>,
    cr_returned_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_returned_time_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_item_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_refunded_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_refunded_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_refunded_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_refunded_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_returning_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_returning_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_returning_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_returning_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_call_center_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_catalog_page_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_ship_mode_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_warehouse_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_reason_sk: PrimitiveArrayRecycler<Int32Type>,
    cr_order_number: PrimitiveArrayRecycler<Int64Type>,
    cr_return_quantity: PrimitiveArrayRecycler<Int32Type>,
    cr_return_amount: PrimitiveArrayRecycler<Decimal128Type>,
    cr_return_tax: PrimitiveArrayRecycler<Decimal128Type>,
    cr_return_amt_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    cr_fee: PrimitiveArrayRecycler<Decimal128Type>,
    cr_return_ship_cost: PrimitiveArrayRecycler<Decimal128Type>,
    cr_refunded_cash: PrimitiveArrayRecycler<Decimal128Type>,
    cr_reversed_charge: PrimitiveArrayRecycler<Decimal128Type>,
    cr_store_credit: PrimitiveArrayRecycler<Decimal128Type>,
    cr_net_loss: PrimitiveArrayRecycler<Decimal128Type>,
}

impl CatalogReturnsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogReturns);
        Self {
            inner: CatalogReturnsRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            cr_returned_date_sk: PrimitiveArrayRecycler::new(),
            cr_returned_time_sk: PrimitiveArrayRecycler::new(),
            cr_item_sk: PrimitiveArrayRecycler::new(),
            cr_refunded_customer_sk: PrimitiveArrayRecycler::new(),
            cr_refunded_cdemo_sk: PrimitiveArrayRecycler::new(),
            cr_refunded_hdemo_sk: PrimitiveArrayRecycler::new(),
            cr_refunded_addr_sk: PrimitiveArrayRecycler::new(),
            cr_returning_customer_sk: PrimitiveArrayRecycler::new(),
            cr_returning_cdemo_sk: PrimitiveArrayRecycler::new(),
            cr_returning_hdemo_sk: PrimitiveArrayRecycler::new(),
            cr_returning_addr_sk: PrimitiveArrayRecycler::new(),
            cr_call_center_sk: PrimitiveArrayRecycler::new(),
            cr_catalog_page_sk: PrimitiveArrayRecycler::new(),
            cr_ship_mode_sk: PrimitiveArrayRecycler::new(),
            cr_warehouse_sk: PrimitiveArrayRecycler::new(),
            cr_reason_sk: PrimitiveArrayRecycler::new(),
            cr_order_number: PrimitiveArrayRecycler::new(),
            cr_return_quantity: PrimitiveArrayRecycler::new(),
            cr_return_amount: PrimitiveArrayRecycler::new(),
            cr_return_tax: PrimitiveArrayRecycler::new(),
            cr_return_amt_inc_tax: PrimitiveArrayRecycler::new(),
            cr_fee: PrimitiveArrayRecycler::new(),
            cr_return_ship_cost: PrimitiveArrayRecycler::new(),
            cr_refunded_cash: PrimitiveArrayRecycler::new(),
            cr_reversed_charge: PrimitiveArrayRecycler::new(),
            cr_store_credit: PrimitiveArrayRecycler::new(),
            cr_net_loss: PrimitiveArrayRecycler::new(),
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
        Self::schema_ref()
    }
}

impl Iterator for CatalogReturnsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scratch.clear();
        self.scratch
            .extend(self.inner.by_ref().take(self.batch_size));
        let rows = &self.scratch;
        if rows.is_empty() {
            return None;
        }

        let schema = self.schema();
        let Self {
            cr_returned_date_sk,
            cr_returned_time_sk,
            cr_item_sk,
            cr_refunded_customer_sk,
            cr_refunded_cdemo_sk,
            cr_refunded_hdemo_sk,
            cr_refunded_addr_sk,
            cr_returning_customer_sk,
            cr_returning_cdemo_sk,
            cr_returning_hdemo_sk,
            cr_returning_addr_sk,
            cr_call_center_sk,
            cr_catalog_page_sk,
            cr_ship_mode_sk,
            cr_warehouse_sk,
            cr_reason_sk,
            cr_order_number,
            cr_return_quantity,
            cr_return_amount,
            cr_return_tax,
            cr_return_amt_inc_tax,
            cr_fee,
            cr_return_ship_cost,
            cr_refunded_cash,
            cr_reversed_charge,
            cr_store_credit,
            cr_net_loss,
            ..
        } = self;
        let nbm = CatalogReturnsRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CatalogReturnsRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_cr_pricing()))
            };

        let cr_returned_date_sk = sk32(
            cr_returned_date_sk,
            0,
            CatalogReturnsRow::get_cr_returned_date_sk,
        );
        let cr_returned_time_sk = sk32(
            cr_returned_time_sk,
            1,
            CatalogReturnsRow::get_cr_returned_time_sk,
        );
        let cr_item_sk = sk32(cr_item_sk, 2, CatalogReturnsRow::get_cr_item_sk);
        let cr_refunded_customer_sk = sk32(
            cr_refunded_customer_sk,
            3,
            CatalogReturnsRow::get_cr_refunded_customer_sk,
        );
        let cr_refunded_cdemo_sk = sk32(
            cr_refunded_cdemo_sk,
            4,
            CatalogReturnsRow::get_cr_refunded_cdemo_sk,
        );
        let cr_refunded_hdemo_sk = sk32(
            cr_refunded_hdemo_sk,
            5,
            CatalogReturnsRow::get_cr_refunded_hdemo_sk,
        );
        let cr_refunded_addr_sk = sk32(
            cr_refunded_addr_sk,
            6,
            CatalogReturnsRow::get_cr_refunded_addr_sk,
        );
        let cr_returning_customer_sk = sk32(
            cr_returning_customer_sk,
            7,
            CatalogReturnsRow::get_cr_returning_customer_sk,
        );
        let cr_returning_cdemo_sk = sk32(
            cr_returning_cdemo_sk,
            8,
            CatalogReturnsRow::get_cr_returning_cdemo_sk,
        );
        let cr_returning_hdemo_sk = sk32(
            cr_returning_hdemo_sk,
            9,
            CatalogReturnsRow::get_cr_returning_hdemo_sk,
        );
        let cr_returning_addr_sk = sk32(
            cr_returning_addr_sk,
            10,
            CatalogReturnsRow::get_cr_returning_addr_sk,
        );
        let cr_call_center_sk = sk32(
            cr_call_center_sk,
            11,
            CatalogReturnsRow::get_cr_call_center_sk,
        );
        let cr_catalog_page_sk = sk32(
            cr_catalog_page_sk,
            12,
            CatalogReturnsRow::get_cr_catalog_page_sk,
        );
        let cr_ship_mode_sk = sk32(cr_ship_mode_sk, 13, CatalogReturnsRow::get_cr_ship_mode_sk);
        let cr_warehouse_sk = sk32(cr_warehouse_sk, 14, CatalogReturnsRow::get_cr_warehouse_sk);
        let cr_reason_sk = sk32(cr_reason_sk, 15, CatalogReturnsRow::get_cr_reason_sk);
        let cr_order_number =
            cr_order_number.int64(rows, 16, nbm, CatalogReturnsRow::get_cr_order_number);
        let cr_return_quantity =
            cr_return_quantity.int32(rows, 17, nbm, |r| r.get_cr_pricing().get_quantity());
        let cr_return_amount = dec(cr_return_amount, 18, Pricing::get_net_paid);
        let cr_return_tax = dec(cr_return_tax, 19, Pricing::get_ext_tax);
        let cr_return_amt_inc_tax = dec(
            cr_return_amt_inc_tax,
            20,
            Pricing::get_net_paid_including_tax,
        );
        let cr_fee = dec(cr_fee, 21, Pricing::get_fee);
        let cr_return_ship_cost = dec(cr_return_ship_cost, 22, Pricing::get_ext_ship_cost);
        let cr_refunded_cash = dec(cr_refunded_cash, 23, Pricing::get_refunded_cash);
        let cr_reversed_charge = dec(cr_reversed_charge, 24, Pricing::get_reversed_charge);
        let cr_store_credit = dec(cr_store_credit, 25, Pricing::get_store_credit);
        let cr_net_loss = dec(cr_net_loss, 26, Pricing::get_net_loss);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(cr_returned_date_sk),
                Arc::new(cr_returned_time_sk),
                Arc::new(cr_item_sk),
                Arc::new(cr_refunded_customer_sk),
                Arc::new(cr_refunded_cdemo_sk),
                Arc::new(cr_refunded_hdemo_sk),
                Arc::new(cr_refunded_addr_sk),
                Arc::new(cr_returning_customer_sk),
                Arc::new(cr_returning_cdemo_sk),
                Arc::new(cr_returning_hdemo_sk),
                Arc::new(cr_returning_addr_sk),
                Arc::new(cr_call_center_sk),
                Arc::new(cr_catalog_page_sk),
                Arc::new(cr_ship_mode_sk),
                Arc::new(cr_warehouse_sk),
                Arc::new(cr_reason_sk),
                Arc::new(cr_order_number),
                Arc::new(cr_return_quantity),
                Arc::new(cr_return_amount),
                Arc::new(cr_return_tax),
                Arc::new(cr_return_amt_inc_tax),
                Arc::new(cr_fee),
                Arc::new(cr_return_ship_cost),
                Arc::new(cr_refunded_cash),
                Arc::new(cr_reversed_charge),
                Arc::new(cr_store_credit),
                Arc::new(cr_net_loss),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cr_returned_date_sk", DataType::Int32, true),
        Field::new("cr_returned_time_sk", DataType::Int32, true),
        Field::new("cr_item_sk", DataType::Int32, false),
        Field::new("cr_refunded_customer_sk", DataType::Int32, true),
        Field::new("cr_refunded_cdemo_sk", DataType::Int32, true),
        Field::new("cr_refunded_hdemo_sk", DataType::Int32, true),
        Field::new("cr_refunded_addr_sk", DataType::Int32, true),
        Field::new("cr_returning_customer_sk", DataType::Int32, true),
        Field::new("cr_returning_cdemo_sk", DataType::Int32, true),
        Field::new("cr_returning_hdemo_sk", DataType::Int32, true),
        Field::new("cr_returning_addr_sk", DataType::Int32, true),
        Field::new("cr_call_center_sk", DataType::Int32, true),
        Field::new("cr_catalog_page_sk", DataType::Int32, true),
        Field::new("cr_ship_mode_sk", DataType::Int32, true),
        Field::new("cr_warehouse_sk", DataType::Int32, true),
        Field::new("cr_reason_sk", DataType::Int32, true),
        Field::new("cr_order_number", DataType::Int64, false),
        Field::new("cr_return_quantity", DataType::Int32, true),
        Field::new("cr_return_amount", DataType::Decimal128(7, 2), true),
        Field::new("cr_return_tax", DataType::Decimal128(7, 2), true),
        Field::new("cr_return_amt_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("cr_fee", DataType::Decimal128(7, 2), true),
        Field::new("cr_return_ship_cost", DataType::Decimal128(7, 2), true),
        Field::new("cr_refunded_cash", DataType::Decimal128(7, 2), true),
        Field::new("cr_reversed_charge", DataType::Decimal128(7, 2), true),
        Field::new("cr_store_credit", DataType::Decimal128(7, 2), true),
        Field::new("cr_net_loss", DataType::Decimal128(7, 2), true),
    ]))
}
