use crate::conversions::{
    address_columns, decimal_to_i128, julian_to_date32, opt, sk_opt,
    string_view_array_from_opt_iter,
};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Date32Array, Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, StoreRowGenerator};

pub struct StoreArrow {
    inner: RowIter<StoreRowGenerator>,
    batch_size: usize,
}

impl StoreArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Store);
        Self {
            inner: RowIter::new(StoreRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for StoreArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for StoreArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::Store(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut s_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut s_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_rec_start: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut s_rec_end: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut s_closed_date: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut s_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_employees: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut s_floor_space: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut s_hours: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_market_id: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut s_geography_class: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_market_desc: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_market_manager: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_division_id: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut s_division_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut s_company_id: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut s_company_name: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut addr_rows: Vec<(tpcdsgen::types::Address, i64, u32)> =
            Vec::with_capacity(rows.len());
        let mut s_tax_pct: Vec<Option<i128>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            s_sk.push(sk_opt(nbm, 0, r.get_store_sk()));
            s_id.push(opt(nbm, 1, r.get_store_id().to_owned()));
            s_rec_start.push(julian_to_date32(r.get_rec_start_date_id()));
            s_rec_end.push(julian_to_date32(r.get_rec_end_date_id()));
            s_closed_date.push(sk_opt(nbm, 4, r.get_closed_date_id()));
            s_name.push(opt(nbm, 5, r.get_store_name().to_owned()));
            s_employees.push(opt(nbm, 6, r.get_employees()));
            s_floor_space.push(opt(nbm, 7, r.get_floor_space()));
            s_hours.push(opt(nbm, 8, r.get_hours().to_owned()));
            s_manager.push(opt(nbm, 9, r.get_store_manager().to_owned()));
            s_market_id.push(opt(nbm, 10, r.get_market_id()));
            s_geography_class.push(opt(nbm, 11, r.get_geography_class().to_owned()));
            s_market_desc.push(opt(nbm, 12, r.get_market_desc().to_owned()));
            s_market_manager.push(opt(nbm, 13, r.get_market_manager().to_owned()));
            s_division_id.push(opt(nbm, 14, r.get_division_id()));
            s_division_name.push(opt(nbm, 15, r.get_division_name().to_owned()));
            s_company_id.push(opt(nbm, 16, r.get_company_id()));
            s_company_name.push(opt(nbm, 17, r.get_company_name().to_owned()));
            addr_rows.push((r.get_address().clone(), nbm, 18));
            s_tax_pct.push(opt(nbm, 28, decimal_to_i128(r.get_d_tax_percentage())));
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

        let tax_arr = Decimal128Array::from(s_tax_pct)
            .with_precision_and_scale(38, 2)
            .unwrap();

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(s_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    s_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Date32Array::from(s_rec_start)),
                Arc::new(Date32Array::from(s_rec_end)),
                Arc::new(Int64Array::from(s_closed_date)),
                Arc::new(string_view_array_from_opt_iter(
                    s_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(s_employees)),
                Arc::new(Int32Array::from(s_floor_space)),
                Arc::new(string_view_array_from_opt_iter(
                    s_hours.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    s_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(s_market_id)),
                Arc::new(string_view_array_from_opt_iter(
                    s_geography_class.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    s_market_desc.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    s_market_manager.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(s_division_id)),
                Arc::new(string_view_array_from_opt_iter(
                    s_division_name.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(s_company_id)),
                Arc::new(string_view_array_from_opt_iter(
                    s_company_name.iter().map(|s| s.as_deref()),
                )),
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
                Arc::new(tax_arr),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("s_store_sk", DataType::Int64, true),
        Field::new("s_store_id", DataType::Utf8View, true),
        Field::new("s_rec_start_date", DataType::Date32, true),
        Field::new("s_rec_end_date", DataType::Date32, true),
        Field::new("s_closed_date_sk", DataType::Int64, true),
        Field::new("s_store_name", DataType::Utf8View, true),
        Field::new("s_number_employees", DataType::Int32, true),
        Field::new("s_floor_space", DataType::Int32, true),
        Field::new("s_hours", DataType::Utf8View, true),
        Field::new("s_manager", DataType::Utf8View, true),
        Field::new("s_market_id", DataType::Int32, true),
        Field::new("s_geography_class", DataType::Utf8View, true),
        Field::new("s_market_desc", DataType::Utf8View, true),
        Field::new("s_market_manager", DataType::Utf8View, true),
        Field::new("s_division_id", DataType::Int64, true),
        Field::new("s_division_name", DataType::Utf8View, true),
        Field::new("s_company_id", DataType::Int64, true),
        Field::new("s_company_name", DataType::Utf8View, true),
        Field::new("s_street_number", DataType::Int32, true),
        Field::new("s_street_name", DataType::Utf8View, true),
        Field::new("s_street_type", DataType::Utf8View, true),
        Field::new("s_suite_number", DataType::Utf8View, true),
        Field::new("s_city", DataType::Utf8View, true),
        Field::new("s_county", DataType::Utf8View, true),
        Field::new("s_state", DataType::Utf8View, true),
        Field::new("s_zip", DataType::Utf8View, true),
        Field::new("s_country", DataType::Utf8View, true),
        Field::new("s_gmt_offset", DataType::Int32, true),
        Field::new("s_tax_precentage", DataType::Decimal128(38, 2), true),
    ]))
}
