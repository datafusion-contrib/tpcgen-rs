use crate::conversions::{opt, sk_opt};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, InventoryRowGenerator};

pub struct InventoryArrow {
    inner: RowIter<InventoryRowGenerator>,
    batch_size: usize,
}

impl InventoryArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        Self {
            inner: RowIter::new(InventoryRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for InventoryArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for InventoryArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Inventory(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut inv_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut inv_item: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut inv_warehouse: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut inv_qty: Vec<Option<i32>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            inv_date.push(sk_opt(nbm, 0, r.get_inv_date_sk()));
            inv_item.push(sk_opt(nbm, 1, r.get_inv_item_sk()));
            inv_warehouse.push(sk_opt(nbm, 2, r.get_inv_warehouse_sk()));
            inv_qty.push(opt(nbm, 3, r.get_inv_quantity_on_hand()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(inv_date)),
                Arc::new(Int64Array::from(inv_item)),
                Arc::new(Int64Array::from(inv_warehouse)),
                Arc::new(Int32Array::from(inv_qty)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("inv_date_sk", DataType::Int64, true),
        Field::new("inv_item_sk", DataType::Int64, true),
        Field::new("inv_warehouse_sk", DataType::Int64, true),
        Field::new("inv_quantity_on_hand", DataType::Int32, true),
    ]))
}
