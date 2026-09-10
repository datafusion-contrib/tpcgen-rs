use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerDemographicsRowGenerator, GeneratedRow};

pub struct CustomerDemographicsArrow {
    inner: RowIter<CustomerDemographicsRowGenerator>,
    batch_size: usize,
}

impl CustomerDemographicsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session
            .get_scaling()
            .get_row_count(Table::CustomerDemographics);
        Self {
            inner: RowIter::new(CustomerDemographicsRowGenerator::new(), session, row_count),
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

impl RecordBatchReader for CustomerDemographicsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CustomerDemographicsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::CustomerDemographics(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut demo_sk: Vec<Option<i64>> = Vec::with_capacity(rows.len());
        let mut gender: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut marital: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut education: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut purchase: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut credit: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut dep_count: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut dep_emp: Vec<Option<i32>> = Vec::with_capacity(rows.len());
        let mut dep_college: Vec<Option<i32>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            demo_sk.push(sk_opt(nbm, 0, r.get_cd_demo_sk()));
            gender.push(opt(nbm, 1, r.get_cd_gender().to_owned()));
            marital.push(opt(nbm, 2, r.get_cd_marital_status().to_owned()));
            education.push(opt(nbm, 3, r.get_cd_education_status().to_owned()));
            purchase.push(opt(nbm, 4, r.get_cd_purchase_estimate()));
            credit.push(opt(nbm, 5, r.get_cd_credit_rating().to_owned()));
            dep_count.push(opt(nbm, 6, r.get_cd_dep_count()));
            dep_emp.push(opt(nbm, 7, r.get_cd_dep_employed_count()));
            dep_college.push(opt(nbm, 8, r.get_cd_dep_college_count()));
        }

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(Int64Array::from(demo_sk)),
                Arc::new(string_view_array_from_opt_iter(
                    gender.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    marital.iter().map(|s| s.as_deref()),
                )),
                Arc::new(string_view_array_from_opt_iter(
                    education.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(purchase)),
                Arc::new(string_view_array_from_opt_iter(
                    credit.iter().map(|s| s.as_deref()),
                )),
                Arc::new(Int32Array::from(dep_count)),
                Arc::new(Int32Array::from(dep_emp)),
                Arc::new(Int32Array::from(dep_college)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cd_demo_sk", DataType::Int64, false),
        Field::new("cd_gender", DataType::Utf8View, false),
        Field::new("cd_marital_status", DataType::Utf8View, false),
        Field::new("cd_education_status", DataType::Utf8View, false),
        Field::new("cd_purchase_estimate", DataType::Int32, false),
        Field::new("cd_credit_rating", DataType::Utf8View, false),
        Field::new("cd_dep_count", DataType::Int32, false),
        Field::new("cd_dep_employed_count", DataType::Int32, false),
        Field::new("cd_dep_college_count", DataType::Int32, false),
    ]))
}
