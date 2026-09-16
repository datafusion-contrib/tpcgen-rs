use crate::conversions::{date_to_date32, opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Date32Array, RecordBatch, Time32SecondArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DbgenVersionRowGenerator, GeneratedRow};

pub struct DbgenVersionArrow {
    inner: RowIter<DbgenVersionRowGenerator>,
    batch_size: usize,
}

impl DbgenVersionArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DbgenVersion);
        Self {
            inner: RowIter::new(DbgenVersionRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for DbgenVersionArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for DbgenVersionArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::DbgenVersion(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut version: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut create_date: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut create_time: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut cmdline: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            version.push(opt(nbm, 0, r.get_dv_version().to_owned()));
            create_date.push(opt(nbm, 1, date_to_date32(r.get_dv_create_date())));
            create_time.push(opt(nbm, 2, r.get_dv_create_time()));
            cmdline.push(opt(nbm, 3, r.get_dv_cmdline_args().to_owned()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(string_view_array_from_opt_iter(
                    version.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Date32Array::from(create_date)),
                Arc::new(Time32SecondArray::from(create_time)),
                Arc::new(string_view_array_from_opt_iter(
                    cmdline.iter().map(|s| s.as_deref()),
                )),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("dv_version", DataType::Utf8View, true),
        Field::new("dv_create_date", DataType::Date32, true),
        Field::new("dv_create_time", DataType::Time32(TimeUnit::Second), true),
        Field::new("dv_cmdline_args", DataType::Utf8View, true),
    ]))
}
