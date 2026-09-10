use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogPageRowGenerator, GeneratedRow};

pub struct CatalogPageArrow {
    inner: RowIter<CatalogPageRowGenerator>,
    batch_size: usize,
}

impl CatalogPageArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogPage);
        Self {
            inner: RowIter::new(CatalogPageRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CatalogPageArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CatalogPageArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::CatalogPage(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut cp_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cp_id: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cp_start: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cp_end: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut cp_dept: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cp_num: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cp_page_num: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cp_desc: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cp_type: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            cp_sk.push(sk_opt(nbm, 0, r.get_cp_catalog_page_sk()));
            cp_id.push(opt(nbm, 1, r.get_cp_catalog_page_id().to_owned()));
            cp_start.push(sk_opt(nbm, 2, r.get_cp_start_date_id()));
            cp_end.push(sk_opt(nbm, 3, r.get_cp_end_date_id()));
            // CpPromoId occupies global bit 4 but is not in the output schema,
            // so output columns shift: CpDepartment=bit5, ..., CpType=bit9.
            cp_dept.push(opt(nbm, 5, r.get_cp_department().to_owned()));
            cp_num.push(opt(nbm, 6, r.get_cp_catalog_number()));
            cp_page_num.push(opt(nbm, 7, r.get_cp_catalog_page_number()));
            cp_desc.push(opt(nbm, 8, r.get_cp_description().to_owned()));
            cp_type.push(opt(nbm, 9, r.get_cp_type().to_owned()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(cp_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    cp_id.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int64Array::from(cp_start)),
                Arc::new(Int64Array::from(cp_end)),
                Arc::new(string_view_array_from_opt_iter(
                    cp_dept.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(cp_num)),
                Arc::new(Int32Array::from(cp_page_num)),
                Arc::new(string_view_array_from_opt_iter(
                    cp_desc.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    cp_type.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cp_catalog_page_sk", DataType::Int64, true),
        Field::new("cp_catalog_page_id", DataType::Utf8View, true),
        Field::new("cp_start_date_sk", DataType::Int64, true),
        Field::new("cp_end_date_sk", DataType::Int64, true),
        Field::new("cp_department", DataType::Utf8View, true),
        Field::new("cp_catalog_number", DataType::Int32, true),
        Field::new("cp_catalog_page_number", DataType::Int32, true),
        Field::new("cp_description", DataType::Utf8View, true),
        Field::new("cp_type", DataType::Utf8View, true),
    ]))
}
