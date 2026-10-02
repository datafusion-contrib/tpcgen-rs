//! Verifies correctness of the tpcdsgen-arrow generators by reparsing the
//! textual output formats (pipe-delimited `.dat` and CSV) and comparing against
//! the directly generated Arrow RecordBatches.
//!
//! This also serves as a transitive test for csv correctness: we compare the
//! DAT output to the original C and Trino generators, and verify it is the same
//! as arrow. Thus, if CSV is the same as arrow, it too is the same as the
//! original generators.
//!
//! Strategy:
//! - drive the tpcdsgen row generators to produce rows for each table
//! - write rows via their `fmt::Display` and [`CsvRow`] impls just like the CLI does
//! - re-parse the output with the Arrow CSV reader using the same schema
//! - assert that the reparsed and direct Arrow RecordBatches are equal

use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatchReader;
use std::fmt::Display;
use std::io::Write as _;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::csv::*;
use tpcdsgen::row::*;
use tpcdsgen_arrow::arrow;
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow, PromotionArrow,
    ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow, TimeDimArrow,
    WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

/// Session options for tests (scale factor 1).
static SESSION: LazyLock<Session> = LazyLock::new(Session::default);
const DAT_SEPARATOR: char = '|';
const CSV_SEPARATOR: char = ',';

/// Number of rows to test for `table`.
fn test_row_count(table: Table) -> u64 {
    // Test up to 10k rows, rather than the entire table, to keep testing time
    // reasonable for large fact tables.
    const MAX_REPARSE_SOURCE_ROWS: u64 = 10_000;

    SESSION
        .get_scaling()
        .get_row_count(table)
        .min(MAX_REPARSE_SOURCE_ROWS)
}

/// The textual output formats that the tpcds crate can produce, each of which
/// must reparse exactly to the generated Arrow data.
#[derive(Debug, Clone, Copy)]
enum Format {
    /// Pipe delimited `.dat` format, with a trailing separator and no header:
    /// ```text
    /// 1|foo|
    /// 2|bar|
    /// ```
    Dat,
    /// Comma delimited CSV, with a header line and quoting as needed:
    /// ```text
    /// id,name
    /// 1,foo
    /// 2,"bar,baz"
    /// ```
    Csv,
}

impl Format {
    /// Writes the header line, if the format has one.
    fn write_header(&self, csv_header: &str, data: &mut Vec<u8>) {
        match self {
            Format::Dat => {}
            Format::Csv => writeln!(data, "{csv_header}").unwrap(),
        }
    }

    /// Writes `row` as a single line, including the trailing newline: its
    /// `Display` for DAT, or `csv_line(row)` for CSV.
    fn write_row<R: Display>(&self, row: &R, csv_line: impl Fn(&R) -> String, data: &mut Vec<u8>) {
        match self {
            Format::Dat => {
                write!(data, "{row}").unwrap();
                // Note: .dat lines end with '|' which the Arrow CSV parser treats as a
                // delimiter for a new column, so replace the trailing '|' with a newline.
                let end_offset = data.len() - 1;
                data[end_offset] = b'\n';
            }
            Format::Csv => writeln!(data, "{}", csv_line(row)).unwrap(),
        }
    }

    /// Re-parses data with the Arrow CSV reader.
    fn parse<'a>(
        &self,
        data: &'a [u8],
        schema: &'a SchemaRef,
    ) -> impl Iterator<Item = RecordBatch> + 'a {
        let null_re = regex::Regex::new("^$").unwrap();
        let builder =
            arrow::csv::reader::ReaderBuilder::new(Arc::clone(schema)).with_null_regex(null_re);
        let builder = match self {
            Format::Dat => builder
                .with_delimiter(DAT_SEPARATOR as u8)
                .with_header(false),
            Format::Csv => builder
                .with_delimiter(CSV_SEPARATOR as u8)
                .with_header(true)
                .with_header_validation(true),
        };
        builder
            .build(data)
            .unwrap()
            .map(|batch| batch.expect("parse text data into RecordBatch"))
    }
}

/// Yields Arrow RecordBatches by writing `rows` in `format`, and parsing the
/// result back to Arrow.
///
/// `csv_header` and `csv_line` give the CSV header and a row's CSV line; a
/// row's DAT line is its `Display`.
fn reparsed_rows<R: Display>(
    mut rows: impl Iterator<Item = R>,
    format: Format,
    schema: &SchemaRef,
    csv_header: String,
    csv_line: impl Fn(&R) -> String,
) -> impl Iterator<Item = RecordBatch> {
    let schema = Arc::clone(schema);

    const REPARSE_BUFFER_TARGET_BYTES: usize = 256 * 1024;
    std::iter::from_fn(move || {
        let mut data = Vec::new();
        format.write_header(&csv_header, &mut data);
        let header_len = data.len();

        while data.len() < REPARSE_BUFFER_TARGET_BYTES {
            let Some(row) = rows.next() else { break };
            format.write_row(&row, &csv_line, &mut data);
        }

        if data.len() == header_len {
            None
        } else {
            let batches = format.parse(&data, &schema).collect::<Vec<_>>();
            Some(batches)
        }
    })
    .flatten()
}

/// Asserts that two streams of Arrow RecordBatches are logically equal up to a
/// specified row limit.
///
/// It ignores any differences in how the rows are distributed across batches
/// by realigning the batches before comparison.
fn assert_record_batch_streams<L, R>(left: L, right: R, row_limit: usize)
where
    L: RecordBatchReader,
    R: Iterator<Item = RecordBatch>,
{
    // Use FixedSizeBatches to align batch boundaries for comparison.
    let left = left.map(|batch| batch.expect("arrow generation should not fail"));
    let mut left = FixedSizeBatches::new(left, row_limit);
    let mut right = FixedSizeBatches::new(right, row_limit);

    // Compare the two streams, batch by batch.
    let mut compared_rows = 0;
    left.by_ref()
        .zip(right.by_ref())
        .for_each(|(left_batch, right_batch)| {
            compared_rows += left_batch.num_rows();
            assert_eq!(left_batch, right_batch);
        });
    assert_eq!(compared_rows, row_limit);
    assert!(left.next().is_none(), "left stream produced extra batches");
    assert!(
        right.next().is_none(),
        "right stream produced extra batches"
    );
}

// ---------------------------------------------------------------------------
// One test per table.
// ---------------------------------------------------------------------------

macro_rules! table_test {
    // $name: module name
    // $gen: the table's row generator type, an `Iterator` over its rows
    //       created with `new(session, source_row_count)`.
    // $csv: the [`CsvRow`] wrapper for the table's rows.
    // $arrow_gen: constructor for the matching Arrow RecordBatch generator.
    // $table: TPC-DS table whose row count is the number of source rows.
    ($name:ident, $gen:ident, $csv:ident, $arrow_gen:expr, $table:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn from_start_dat() {
                from_start(Format::Dat);
            }

            #[test]
            fn from_start_csv() {
                from_start(Format::Csv);
            }

            #[test]
            fn skip_dat() {
                skip(Format::Dat);
            }

            #[test]
            fn skip_csv() {
                skip(Format::Csv);
            }

            /// Parse from the start of the table
            fn from_start(format: Format) {
                let source_row_count = SESSION.get_scaling().get_row_count($table);
                let row_limit = test_row_count($table) as usize;
                let arrow_gen = $arrow_gen(SESSION.clone());
                let schema = arrow_gen.schema();
                let rows = <$gen>::new(SESSION.clone(), source_row_count);
                let reparsed = reparsed_rows(
                    rows,
                    format,
                    &schema,
                    <$csv>::header_with_delimiter(CSV_SEPARATOR),
                    |row| <$csv>::with_delimiter(row, CSV_SEPARATOR).to_string(),
                );

                assert_record_batch_streams(arrow_gen, reparsed, row_limit);
            }

            /// Parse after skipping some rows.
            fn skip(format: Format) {
                let source_row_count = SESSION.get_scaling().get_row_count($table);
                let starting_row_number = source_row_count.min(100);
                let remaining_source_rows = source_row_count - starting_row_number + 1;
                let row_limit =
                    test_row_count($table).min(remaining_source_rows).min(1024) as usize;

                let mut rows = <$gen>::new(SESSION.clone(), source_row_count);
                rows.skip_rows_until_starting_row_number(starting_row_number);

                let mut arrow_gen = $arrow_gen(SESSION.clone());
                arrow_gen.skip_rows_until_starting_row_number(starting_row_number);

                let schema = arrow_gen.schema();
                let reparsed = reparsed_rows(
                    rows,
                    format,
                    &schema,
                    <$csv>::header_with_delimiter(CSV_SEPARATOR),
                    |row| <$csv>::with_delimiter(row, CSV_SEPARATOR).to_string(),
                );

                assert_record_batch_streams(arrow_gen, reparsed, row_limit);
            }
        }
    };
}

table_test!(
    income_band,
    IncomeBandRowGenerator,
    IncomeBandCsv,
    IncomeBandArrow::new,
    Table::IncomeBand
);
table_test!(
    reason,
    ReasonRowGenerator,
    ReasonCsv,
    ReasonArrow::new,
    Table::Reason
);
table_test!(
    ship_mode,
    ShipModeRowGenerator,
    ShipModeCsv,
    ShipModeArrow::new,
    Table::ShipMode
);
table_test!(
    inventory,
    InventoryRowGenerator,
    InventoryCsv,
    InventoryArrow::new,
    Table::Inventory
);
table_test!(
    household_demographics,
    HouseholdDemographicsRowGenerator,
    HouseholdDemographicsCsv,
    HouseholdDemographicsArrow::new,
    Table::HouseholdDemographics
);
table_test!(
    customer_demographics,
    CustomerDemographicsRowGenerator,
    CustomerDemographicsCsv,
    CustomerDemographicsArrow::new,
    Table::CustomerDemographics
);
table_test!(
    customer_address,
    CustomerAddressRowGenerator,
    CustomerAddressCsv,
    CustomerAddressArrow::new,
    Table::CustomerAddress
);
table_test!(
    customer,
    CustomerRowGenerator,
    CustomerCsv,
    CustomerArrow::new,
    Table::Customer
);
table_test!(
    catalog_page,
    CatalogPageRowGenerator,
    CatalogPageCsv,
    CatalogPageArrow::new,
    Table::CatalogPage
);
table_test!(
    time_dim,
    TimeDimRowGenerator,
    TimeDimCsv,
    TimeDimArrow::new,
    Table::TimeDim
);
table_test!(
    date_dim,
    DateDimRowGenerator,
    DateDimCsv,
    DateDimArrow::new,
    Table::DateDim
);
table_test!(
    warehouse,
    WarehouseRowGenerator,
    WarehouseCsv,
    WarehouseArrow::new,
    Table::Warehouse
);
table_test!(item, ItemRowGenerator, ItemCsv, ItemArrow::new, Table::Item);
table_test!(
    promotion,
    PromotionRowGenerator,
    PromotionCsv,
    PromotionArrow::new,
    Table::Promotion
);
table_test!(
    store,
    StoreRowGenerator,
    StoreCsv,
    StoreArrow::new,
    Table::Store
);
table_test!(
    web_page,
    WebPageRowGenerator,
    WebPageCsv,
    WebPageArrow::new,
    Table::WebPage
);
table_test!(
    web_site,
    WebSiteRowGenerator,
    WebSiteCsv,
    WebSiteArrow::new,
    Table::WebSite
);
table_test!(
    call_center,
    CallCenterRowGenerator,
    CallCenterCsv,
    CallCenterArrow::new,
    Table::CallCenter
);

table_test!(
    catalog_sales,
    CatalogSalesRowGenerator,
    CatalogSalesCsv,
    CatalogSalesArrow::new,
    Table::CatalogSales
);
table_test!(
    catalog_returns,
    CatalogReturnsRowGenerator,
    CatalogReturnsCsv,
    CatalogReturnsArrow::new,
    Table::CatalogSales
);
table_test!(
    store_sales,
    StoreSalesRowGenerator,
    StoreSalesCsv,
    StoreSalesArrow::new,
    Table::StoreSales
);
table_test!(
    store_returns,
    StoreReturnsRowGenerator,
    StoreReturnsCsv,
    StoreReturnsArrow::new,
    Table::StoreSales
);
table_test!(
    web_sales,
    WebSalesRowGenerator,
    WebSalesCsv,
    WebSalesArrow::new,
    Table::WebSales
);
table_test!(
    web_returns,
    WebReturnsRowGenerator,
    WebReturnsCsv,
    WebReturnsArrow::new,
    Table::WebSales
);

/// Adapts an iterator of RecordBatches to emit batches with a fixed row count.
///
/// This iterator is designed to assist comparing two iterators of RecordBatches
/// where the batch sizes can be different between the two iterators.
///
/// It concatenates small batches and slices large batches so each yielded batch
/// has `batch_size` rows, except the final batch, which may be smaller.
///
/// It stops after `row_limit` rows.
struct FixedSizeBatches<I> {
    /// The source of the RecordBatches.
    inner: I,
    /// The output batch size, except for the last batch.
    batch_size: usize,
    /// How many rows remain until the limit.
    remaining_rows: usize,
    /// Partially output batch, if any.
    pending: Option<RecordBatch>,
}

impl<I> FixedSizeBatches<I> {
    fn new(inner: I, row_limit: usize) -> Self {
        Self {
            inner,
            batch_size: 1024,
            remaining_rows: row_limit,
            pending: None,
        }
    }
}

impl<I> Iterator for FixedSizeBatches<I>
where
    I: Iterator<Item = RecordBatch>,
{
    type Item = RecordBatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_rows == 0 {
            return None;
        }

        let target_rows = self.batch_size.min(self.remaining_rows);
        let mut batches = Vec::new();
        let mut rows = 0;

        while rows < target_rows {
            let batch = match self.pending.take().or_else(|| self.inner.next()) {
                Some(batch) => batch,
                None => break,
            };

            let remaining = target_rows - rows;
            if batch.num_rows() <= remaining {
                rows += batch.num_rows();
                batches.push(batch);
            } else {
                batches.push(batch.slice(0, remaining));
                self.pending = Some(batch.slice(remaining, batch.num_rows() - remaining));
                rows = target_rows;
            }
        }

        if rows == 0 {
            None
        } else {
            self.remaining_rows -= rows;
            let schema = batches[0].schema();
            Some(concat_batches(&schema, &batches).expect("concatenate batches"))
        }
    }
}
