use crate::conversions::{address_columns, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, WarehouseRowGenerator};

pub struct WarehouseArrow {
    inner: RowIter<WarehouseRowGenerator>,
    batch_size: usize,
}

impl WarehouseArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Warehouse);
        Self {
            inner: RowIter::new(WarehouseRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for WarehouseArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for WarehouseArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Warehouse(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut w_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut w_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut w_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut w_sq_ft: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut addr_rows: Vec<(tpcdsgen::types::Address, i64, u32)> =
            Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            w_sk.push(sk_opt(nbm, 0, r.get_w_warehouse_sk()));
            w_id.push(opt(nbm, 1, r.get_w_warehouse_id().to_owned()));
            w_name.push(opt(nbm, 2, r.get_w_warehouse_name().to_owned()));
            w_sq_ft.push(opt(nbm, 3, r.get_w_warehouse_sq_ft()));
            addr_rows.push((r.get_w_address().clone(), nbm, 4));
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

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(w_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    w_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    w_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(w_sq_ft)),
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
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("w_warehouse_sk", DataType::Int64, true),
        Field::new("w_warehouse_id", DataType::Utf8View, true),
        Field::new("w_warehouse_name", DataType::Utf8View, true),
        Field::new("w_warehouse_sq_ft", DataType::Int32, true),
        Field::new("w_street_number", DataType::Int32, true),
        Field::new("w_street_name", DataType::Utf8View, true),
        Field::new("w_street_type", DataType::Utf8View, true),
        Field::new("w_suite_number", DataType::Utf8View, true),
        Field::new("w_city", DataType::Utf8View, true),
        Field::new("w_county", DataType::Utf8View, true),
        Field::new("w_state", DataType::Utf8View, true),
        Field::new("w_zip", DataType::Utf8View, true),
        Field::new("w_country", DataType::Utf8View, true),
        Field::new("w_gmt_offset", DataType::Int32, true),
    ]))
}
