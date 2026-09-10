use crate::conversions::{is_null, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch, StringViewBuilder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerAddressRowGenerator, GeneratedRow};

pub struct CustomerAddressArrow {
    inner: RowIter<CustomerAddressRowGenerator>,
    batch_size: usize,
}

impl CustomerAddressArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CustomerAddress);
        Self {
            inner: RowIter::new(CustomerAddressRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CustomerAddressArrow {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&SCHEMA)
    }
}

impl Iterator for CustomerAddressArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::CustomerAddress(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut ca_addr_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut ca_addr_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut street_number: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut street_name_b = StringViewBuilder::new();
        let mut street_type_b = StringViewBuilder::new();
        let mut suite_number_b = StringViewBuilder::new();
        let mut city_b = StringViewBuilder::new();
        let mut county_b = StringViewBuilder::new();
        let mut state_b = StringViewBuilder::new();
        let mut zip_b = StringViewBuilder::new();
        let mut country_b = StringViewBuilder::new();
        let mut gmt_offset: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut location_type: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            let a = r.get_ca_address();
            ca_addr_sk.push(sk_opt(nbm, 0, r.get_ca_addr_sk()));
            ca_addr_id.push(opt(nbm, 1, r.get_ca_addr_id().to_owned()));
            street_number.push(if is_null(nbm, 2) {
                None
            } else {
                Some(a.get_street_number())
            });
            if is_null(nbm, 3) {
                street_name_b.append_null();
            } else {
                street_name_b.append_value(a.get_street_name());
            }
            if is_null(nbm, 4) {
                street_type_b.append_null();
            } else {
                street_type_b.append_value(a.get_street_type());
            }
            if is_null(nbm, 5) {
                suite_number_b.append_null();
            } else {
                suite_number_b.append_value(a.get_suite_number());
            }
            if is_null(nbm, 6) {
                city_b.append_null();
            } else {
                city_b.append_value(a.get_city());
            }
            match a.get_county() {
                Some(c) if !is_null(nbm, 7) => county_b.append_value(c),
                _ => county_b.append_null(),
            }
            if is_null(nbm, 8) {
                state_b.append_null();
            } else {
                state_b.append_value(a.get_state());
            }
            if is_null(nbm, 9) {
                zip_b.append_null();
            } else {
                zip_b.append_value(format!("{:05}", a.get_zip()));
            }
            if is_null(nbm, 10) {
                country_b.append_null();
            } else {
                country_b.append_value(a.get_country());
            }
            gmt_offset.push(if is_null(nbm, 11) {
                None
            } else {
                Some(a.get_gmt_offset())
            });
            location_type.push(opt(nbm, 12, r.get_ca_location_type().to_owned()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(ca_addr_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    ca_addr_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(street_number)),
                Arc::new(street_name_b.finish()),
                Arc::new(street_type_b.finish()),
                Arc::new(suite_number_b.finish()),
                Arc::new(city_b.finish()),
                Arc::new(county_b.finish()),
                Arc::new(state_b.finish()),
                Arc::new(zip_b.finish()),
                Arc::new(country_b.finish()),
                Arc::new(Int32Array::from(gmt_offset)),
                Arc::new(string_view_array_from_opt_iter(
                    location_type.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ca_address_sk", DataType::Int64, true),
        Field::new("ca_address_id", DataType::Utf8View, true),
        Field::new("ca_street_number", DataType::Int32, true),
        Field::new("ca_street_name", DataType::Utf8View, true),
        Field::new("ca_street_type", DataType::Utf8View, true),
        Field::new("ca_suite_number", DataType::Utf8View, true),
        Field::new("ca_city", DataType::Utf8View, true),
        Field::new("ca_county", DataType::Utf8View, true),
        Field::new("ca_state", DataType::Utf8View, true),
        Field::new("ca_zip", DataType::Utf8View, true),
        Field::new("ca_country", DataType::Utf8View, true),
        Field::new("ca_gmt_offset", DataType::Int32, true),
        Field::new("ca_location_type", DataType::Utf8View, true),
    ]))
}
