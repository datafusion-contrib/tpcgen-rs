use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{WebReturnsRow, WebReturnsRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`WebReturnsRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct WebReturnsArrow {
    inner: WebReturnsRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<WebReturnsRow>,
    wr_returned_date_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_returned_time_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_item_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_refunded_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_refunded_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_refunded_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_refunded_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_returning_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_returning_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_returning_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_returning_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_web_page_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_reason_sk: PrimitiveArrayRecycler<Int32Type>,
    wr_order_number: PrimitiveArrayRecycler<Int64Type>,
    wr_return_quantity: PrimitiveArrayRecycler<Int32Type>,
    wr_return_amt: PrimitiveArrayRecycler<Decimal128Type>,
    wr_return_tax: PrimitiveArrayRecycler<Decimal128Type>,
    wr_return_amt_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    wr_fee: PrimitiveArrayRecycler<Decimal128Type>,
    wr_return_ship_cost: PrimitiveArrayRecycler<Decimal128Type>,
    wr_refunded_cash: PrimitiveArrayRecycler<Decimal128Type>,
    wr_reversed_charge: PrimitiveArrayRecycler<Decimal128Type>,
    wr_account_credit: PrimitiveArrayRecycler<Decimal128Type>,
    wr_net_loss: PrimitiveArrayRecycler<Decimal128Type>,
}

impl WebReturnsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebReturns);
        Self {
            inner: WebReturnsRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            wr_returned_date_sk: PrimitiveArrayRecycler::new(),
            wr_returned_time_sk: PrimitiveArrayRecycler::new(),
            wr_item_sk: PrimitiveArrayRecycler::new(),
            wr_refunded_customer_sk: PrimitiveArrayRecycler::new(),
            wr_refunded_cdemo_sk: PrimitiveArrayRecycler::new(),
            wr_refunded_hdemo_sk: PrimitiveArrayRecycler::new(),
            wr_refunded_addr_sk: PrimitiveArrayRecycler::new(),
            wr_returning_customer_sk: PrimitiveArrayRecycler::new(),
            wr_returning_cdemo_sk: PrimitiveArrayRecycler::new(),
            wr_returning_hdemo_sk: PrimitiveArrayRecycler::new(),
            wr_returning_addr_sk: PrimitiveArrayRecycler::new(),
            wr_web_page_sk: PrimitiveArrayRecycler::new(),
            wr_reason_sk: PrimitiveArrayRecycler::new(),
            wr_order_number: PrimitiveArrayRecycler::new(),
            wr_return_quantity: PrimitiveArrayRecycler::new(),
            wr_return_amt: PrimitiveArrayRecycler::new(),
            wr_return_tax: PrimitiveArrayRecycler::new(),
            wr_return_amt_inc_tax: PrimitiveArrayRecycler::new(),
            wr_fee: PrimitiveArrayRecycler::new(),
            wr_return_ship_cost: PrimitiveArrayRecycler::new(),
            wr_refunded_cash: PrimitiveArrayRecycler::new(),
            wr_reversed_charge: PrimitiveArrayRecycler::new(),
            wr_account_credit: PrimitiveArrayRecycler::new(),
            wr_net_loss: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for WebReturnsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WebReturnsArrow {
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
            wr_returned_date_sk,
            wr_returned_time_sk,
            wr_item_sk,
            wr_refunded_customer_sk,
            wr_refunded_cdemo_sk,
            wr_refunded_hdemo_sk,
            wr_refunded_addr_sk,
            wr_returning_customer_sk,
            wr_returning_cdemo_sk,
            wr_returning_hdemo_sk,
            wr_returning_addr_sk,
            wr_web_page_sk,
            wr_reason_sk,
            wr_order_number,
            wr_return_quantity,
            wr_return_amt,
            wr_return_tax,
            wr_return_amt_inc_tax,
            wr_fee,
            wr_return_ship_cost,
            wr_refunded_cash,
            wr_reversed_charge,
            wr_account_credit,
            wr_net_loss,
            ..
        } = self;
        let nbm = WebReturnsRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebReturnsRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_wr_pricing()))
            };

        let wr_returned_date_sk = sk32(
            wr_returned_date_sk,
            0,
            WebReturnsRow::get_wr_returned_date_sk,
        );
        let wr_returned_time_sk = sk32(
            wr_returned_time_sk,
            1,
            WebReturnsRow::get_wr_returned_time_sk,
        );
        let wr_item_sk = sk32(wr_item_sk, 2, WebReturnsRow::get_wr_item_sk);
        let wr_refunded_customer_sk = sk32(
            wr_refunded_customer_sk,
            3,
            WebReturnsRow::get_wr_refunded_customer_sk,
        );
        let wr_refunded_cdemo_sk = sk32(
            wr_refunded_cdemo_sk,
            4,
            WebReturnsRow::get_wr_refunded_cdemo_sk,
        );
        let wr_refunded_hdemo_sk = sk32(
            wr_refunded_hdemo_sk,
            5,
            WebReturnsRow::get_wr_refunded_hdemo_sk,
        );
        let wr_refunded_addr_sk = sk32(
            wr_refunded_addr_sk,
            6,
            WebReturnsRow::get_wr_refunded_addr_sk,
        );
        let wr_returning_customer_sk = sk32(
            wr_returning_customer_sk,
            7,
            WebReturnsRow::get_wr_returning_customer_sk,
        );
        let wr_returning_cdemo_sk = sk32(
            wr_returning_cdemo_sk,
            8,
            WebReturnsRow::get_wr_returning_cdemo_sk,
        );
        let wr_returning_hdemo_sk = sk32(
            wr_returning_hdemo_sk,
            9,
            WebReturnsRow::get_wr_returning_hdemo_sk,
        );
        let wr_returning_addr_sk = sk32(
            wr_returning_addr_sk,
            10,
            WebReturnsRow::get_wr_returning_addr_sk,
        );
        let wr_web_page_sk = sk32(wr_web_page_sk, 11, WebReturnsRow::get_wr_web_page_sk);
        let wr_reason_sk = sk32(wr_reason_sk, 12, WebReturnsRow::get_wr_reason_sk);
        let wr_order_number =
            wr_order_number.int64(rows, 13, nbm, WebReturnsRow::get_wr_order_number);
        let wr_return_quantity =
            wr_return_quantity.int32(rows, 14, nbm, |r| r.get_wr_pricing().get_quantity());
        let wr_return_amt = dec(wr_return_amt, 15, Pricing::get_net_paid);
        let wr_return_tax = dec(wr_return_tax, 16, Pricing::get_ext_tax);
        let wr_return_amt_inc_tax = dec(
            wr_return_amt_inc_tax,
            17,
            Pricing::get_net_paid_including_tax,
        );
        let wr_fee = dec(wr_fee, 18, Pricing::get_fee);
        let wr_return_ship_cost = dec(wr_return_ship_cost, 19, Pricing::get_ext_ship_cost);
        let wr_refunded_cash = dec(wr_refunded_cash, 20, Pricing::get_refunded_cash);
        let wr_reversed_charge = dec(wr_reversed_charge, 21, Pricing::get_reversed_charge);
        let wr_account_credit = dec(wr_account_credit, 22, Pricing::get_store_credit);
        let wr_net_loss = dec(wr_net_loss, 23, Pricing::get_net_loss);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(wr_returned_date_sk),
                Arc::new(wr_returned_time_sk),
                Arc::new(wr_item_sk),
                Arc::new(wr_refunded_customer_sk),
                Arc::new(wr_refunded_cdemo_sk),
                Arc::new(wr_refunded_hdemo_sk),
                Arc::new(wr_refunded_addr_sk),
                Arc::new(wr_returning_customer_sk),
                Arc::new(wr_returning_cdemo_sk),
                Arc::new(wr_returning_hdemo_sk),
                Arc::new(wr_returning_addr_sk),
                Arc::new(wr_web_page_sk),
                Arc::new(wr_reason_sk),
                Arc::new(wr_order_number),
                Arc::new(wr_return_quantity),
                Arc::new(wr_return_amt),
                Arc::new(wr_return_tax),
                Arc::new(wr_return_amt_inc_tax),
                Arc::new(wr_fee),
                Arc::new(wr_return_ship_cost),
                Arc::new(wr_refunded_cash),
                Arc::new(wr_reversed_charge),
                Arc::new(wr_account_credit),
                Arc::new(wr_net_loss),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("wr_returned_date_sk", DataType::Int32, true),
        Field::new("wr_returned_time_sk", DataType::Int32, true),
        Field::new("wr_item_sk", DataType::Int32, false),
        Field::new("wr_refunded_customer_sk", DataType::Int32, true),
        Field::new("wr_refunded_cdemo_sk", DataType::Int32, true),
        Field::new("wr_refunded_hdemo_sk", DataType::Int32, true),
        Field::new("wr_refunded_addr_sk", DataType::Int32, true),
        Field::new("wr_returning_customer_sk", DataType::Int32, true),
        Field::new("wr_returning_cdemo_sk", DataType::Int32, true),
        Field::new("wr_returning_hdemo_sk", DataType::Int32, true),
        Field::new("wr_returning_addr_sk", DataType::Int32, true),
        Field::new("wr_web_page_sk", DataType::Int32, true),
        Field::new("wr_reason_sk", DataType::Int32, true),
        Field::new("wr_order_number", DataType::Int64, false),
        Field::new("wr_return_quantity", DataType::Int32, true),
        Field::new("wr_return_amt", DataType::Decimal128(7, 2), true),
        Field::new("wr_return_tax", DataType::Decimal128(7, 2), true),
        Field::new("wr_return_amt_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("wr_fee", DataType::Decimal128(7, 2), true),
        Field::new("wr_return_ship_cost", DataType::Decimal128(7, 2), true),
        Field::new("wr_refunded_cash", DataType::Decimal128(7, 2), true),
        Field::new("wr_reversed_charge", DataType::Decimal128(7, 2), true),
        Field::new("wr_account_credit", DataType::Decimal128(7, 2), true),
        Field::new("wr_net_loss", DataType::Decimal128(7, 2), true),
    ]))
}
