use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{StoreReturnsRow, StoreReturnsRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`StoreReturnsRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct StoreReturnsArrow {
    inner: StoreReturnsRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<StoreReturnsRow>,
    sr_returned_date_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_returned_time_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_item_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_store_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_reason_sk: PrimitiveArrayRecycler<Int32Type>,
    sr_ticket_number: PrimitiveArrayRecycler<Int64Type>,
    sr_return_quantity: PrimitiveArrayRecycler<Int32Type>,
    sr_return_amt: PrimitiveArrayRecycler<Decimal128Type>,
    sr_return_tax: PrimitiveArrayRecycler<Decimal128Type>,
    sr_return_amt_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    sr_fee: PrimitiveArrayRecycler<Decimal128Type>,
    sr_return_ship_cost: PrimitiveArrayRecycler<Decimal128Type>,
    sr_refunded_cash: PrimitiveArrayRecycler<Decimal128Type>,
    sr_reversed_charge: PrimitiveArrayRecycler<Decimal128Type>,
    sr_store_credit: PrimitiveArrayRecycler<Decimal128Type>,
    sr_net_loss: PrimitiveArrayRecycler<Decimal128Type>,
}

impl StoreReturnsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::StoreReturns);
        Self {
            inner: StoreReturnsRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            sr_returned_date_sk: PrimitiveArrayRecycler::new(),
            sr_returned_time_sk: PrimitiveArrayRecycler::new(),
            sr_item_sk: PrimitiveArrayRecycler::new(),
            sr_customer_sk: PrimitiveArrayRecycler::new(),
            sr_cdemo_sk: PrimitiveArrayRecycler::new(),
            sr_hdemo_sk: PrimitiveArrayRecycler::new(),
            sr_addr_sk: PrimitiveArrayRecycler::new(),
            sr_store_sk: PrimitiveArrayRecycler::new(),
            sr_reason_sk: PrimitiveArrayRecycler::new(),
            sr_ticket_number: PrimitiveArrayRecycler::new(),
            sr_return_quantity: PrimitiveArrayRecycler::new(),
            sr_return_amt: PrimitiveArrayRecycler::new(),
            sr_return_tax: PrimitiveArrayRecycler::new(),
            sr_return_amt_inc_tax: PrimitiveArrayRecycler::new(),
            sr_fee: PrimitiveArrayRecycler::new(),
            sr_return_ship_cost: PrimitiveArrayRecycler::new(),
            sr_refunded_cash: PrimitiveArrayRecycler::new(),
            sr_reversed_charge: PrimitiveArrayRecycler::new(),
            sr_store_credit: PrimitiveArrayRecycler::new(),
            sr_net_loss: PrimitiveArrayRecycler::new(),
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
        Self::schema_ref()
    }
}

impl Iterator for StoreReturnsArrow {
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
            sr_returned_date_sk,
            sr_returned_time_sk,
            sr_item_sk,
            sr_customer_sk,
            sr_cdemo_sk,
            sr_hdemo_sk,
            sr_addr_sk,
            sr_store_sk,
            sr_reason_sk,
            sr_ticket_number,
            sr_return_quantity,
            sr_return_amt,
            sr_return_tax,
            sr_return_amt_inc_tax,
            sr_fee,
            sr_return_ship_cost,
            sr_refunded_cash,
            sr_reversed_charge,
            sr_store_credit,
            sr_net_loss,
            ..
        } = self;
        let nbm = StoreReturnsRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&StoreReturnsRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_sr_pricing()))
            };

        let sr_returned_date_sk = sk32(
            sr_returned_date_sk,
            0,
            StoreReturnsRow::get_sr_returned_date_sk,
        );
        let sr_returned_time_sk = sk32(
            sr_returned_time_sk,
            1,
            StoreReturnsRow::get_sr_returned_time_sk,
        );
        let sr_item_sk = sk32(sr_item_sk, 2, StoreReturnsRow::get_sr_item_sk);
        let sr_customer_sk = sk32(sr_customer_sk, 3, StoreReturnsRow::get_sr_customer_sk);
        let sr_cdemo_sk = sk32(sr_cdemo_sk, 4, StoreReturnsRow::get_sr_cdemo_sk);
        let sr_hdemo_sk = sk32(sr_hdemo_sk, 5, StoreReturnsRow::get_sr_hdemo_sk);
        let sr_addr_sk = sk32(sr_addr_sk, 6, StoreReturnsRow::get_sr_addr_sk);
        let sr_store_sk = sk32(sr_store_sk, 7, StoreReturnsRow::get_sr_store_sk);
        let sr_reason_sk = sk32(sr_reason_sk, 8, StoreReturnsRow::get_sr_reason_sk);
        let sr_ticket_number =
            sr_ticket_number.int64_sk(rows, 9, nbm, StoreReturnsRow::get_sr_ticket_number);
        let sr_return_quantity =
            sr_return_quantity.int32(rows, 10, nbm, |r| r.get_sr_pricing().get_quantity());
        let sr_return_amt = dec(sr_return_amt, 11, Pricing::get_net_paid);
        let sr_return_tax = dec(sr_return_tax, 12, Pricing::get_ext_tax);
        let sr_return_amt_inc_tax = dec(
            sr_return_amt_inc_tax,
            13,
            Pricing::get_net_paid_including_tax,
        );
        let sr_fee = dec(sr_fee, 14, Pricing::get_fee);
        let sr_return_ship_cost = dec(sr_return_ship_cost, 15, Pricing::get_ext_ship_cost);
        let sr_refunded_cash = dec(sr_refunded_cash, 16, Pricing::get_refunded_cash);
        let sr_reversed_charge = dec(sr_reversed_charge, 17, Pricing::get_reversed_charge);
        let sr_store_credit = dec(sr_store_credit, 18, Pricing::get_store_credit);
        let sr_net_loss = dec(sr_net_loss, 19, Pricing::get_net_loss);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(sr_returned_date_sk),
                Arc::new(sr_returned_time_sk),
                Arc::new(sr_item_sk),
                Arc::new(sr_customer_sk),
                Arc::new(sr_cdemo_sk),
                Arc::new(sr_hdemo_sk),
                Arc::new(sr_addr_sk),
                Arc::new(sr_store_sk),
                Arc::new(sr_reason_sk),
                Arc::new(sr_ticket_number),
                Arc::new(sr_return_quantity),
                Arc::new(sr_return_amt),
                Arc::new(sr_return_tax),
                Arc::new(sr_return_amt_inc_tax),
                Arc::new(sr_fee),
                Arc::new(sr_return_ship_cost),
                Arc::new(sr_refunded_cash),
                Arc::new(sr_reversed_charge),
                Arc::new(sr_store_credit),
                Arc::new(sr_net_loss),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sr_returned_date_sk", DataType::Int32, true),
        Field::new("sr_return_time_sk", DataType::Int32, true),
        Field::new("sr_item_sk", DataType::Int32, false),
        Field::new("sr_customer_sk", DataType::Int32, true),
        Field::new("sr_cdemo_sk", DataType::Int32, true),
        Field::new("sr_hdemo_sk", DataType::Int32, true),
        Field::new("sr_addr_sk", DataType::Int32, true),
        Field::new("sr_store_sk", DataType::Int32, true),
        Field::new("sr_reason_sk", DataType::Int32, true),
        Field::new("sr_ticket_number", DataType::Int64, false),
        Field::new("sr_return_quantity", DataType::Int32, true),
        Field::new("sr_return_amt", DataType::Decimal128(7, 2), true),
        Field::new("sr_return_tax", DataType::Decimal128(7, 2), true),
        Field::new("sr_return_amt_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("sr_fee", DataType::Decimal128(7, 2), true),
        Field::new("sr_return_ship_cost", DataType::Decimal128(7, 2), true),
        Field::new("sr_refunded_cash", DataType::Decimal128(7, 2), true),
        Field::new("sr_reversed_charge", DataType::Decimal128(7, 2), true),
        Field::new("sr_store_credit", DataType::Decimal128(7, 2), true),
        Field::new("sr_net_loss", DataType::Decimal128(7, 2), true),
    ]))
}
