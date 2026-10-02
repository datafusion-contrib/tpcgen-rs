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
//! - drive the tpcdsgen `SingleRowIter` / `SalesRowIter` to produce rows for each table
//! - write rows via their `fmt::Display` impls just like the CLI does
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
use tpcdsgen::csv::{
    csv_header, CallCenterCsv, CatalogPageCsv, CatalogReturnsCsv, CatalogSalesCsv, CsvRow,
    CustomerAddressCsv, CustomerCsv, CustomerDemographicsCsv, DateDimCsv, HouseholdDemographicsCsv,
    IncomeBandCsv, InventoryCsv, ItemCsv, PromotionCsv, ReasonCsv, ShipModeCsv, StoreCsv,
    StoreReturnsCsv, StoreSalesCsv, TimeDimCsv, WarehouseCsv, WebPageCsv, WebReturnsCsv,
    WebSalesCsv, WebSiteCsv,
};
use tpcdsgen::row::{
    CallCenterRowGenerator, CatalogPageRowGenerator, CatalogSalesRowGenerator,
    CustomerAddressRowGenerator, CustomerDemographicsRowGenerator, CustomerRowGenerator,
    DateDimRowGenerator, HouseholdDemographicsRowGenerator, IncomeBandRowGenerator,
    InventoryRowGenerator, ItemRowGenerator, PromotionRowGenerator, ReasonRowGenerator,
    SalesRowGenerator, SalesRowIter, ShipModeRowGenerator, SingleRowGenerator, SingleRowIter,
    StoreRowGenerator, StoreSalesRowGenerator, TimeDimRowGenerator, WarehouseRowGenerator,
    WebPageRowGenerator, WebSalesRowGenerator, WebSiteRowGenerator,
};
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
    /// Writes the header line for `table`, if the format has one.
    fn write_header(&self, table: Table, data: &mut Vec<u8>) {
        match self {
            Format::Dat => {}
            Format::Csv => {
                let header = csv_header(table, CSV_SEPARATOR).expect("csv header for table");
                writeln!(data, "{header}").unwrap();
            }
        }
    }

    /// Writes `row` as a single line, including the trailing newline.
    /// `write_csv` writes the row's CSV line.
    fn write_row<R: Display>(
        &self,
        row: &R,
        write_csv: &impl Fn(&R, &mut Vec<u8>),
        data: &mut Vec<u8>,
    ) {
        match self {
            Format::Dat => {
                write!(data, "{row}").unwrap();
                // Note: .dat lines end with '|' which the Arrow CSV parser treats as a
                // delimiter for a new column, so replace the trailing '|' with a newline.
                let end_offset = data.len() - 1;
                data[end_offset] = b'\n';
            }
            Format::Csv => write_csv(row, data),
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

/// Rows of single-row `table`, starting at source row `starting_row_number`.
fn single_rows<G: SingleRowGenerator>(
    generator: G,
    table: Table,
    starting_row_number: u64,
) -> SingleRowIter<G> {
    let source_row_count = SESSION.get_scaling().get_row_count(table);
    let mut rows = SingleRowIter::new(generator, SESSION.clone(), source_row_count);
    rows.skip_rows_until_starting_row_number(starting_row_number);
    rows
}

/// Line items of sales (or paired returns) `table`, starting at source row
/// `starting_row_number`.
fn line_items<G: SalesRowGenerator>(
    generator: G,
    table: Table,
    starting_row_number: u64,
) -> SalesRowIter<G> {
    let source_row_count = SESSION.get_scaling().get_row_count(table.source_table());
    let mut rows = SalesRowIter::new(generator, SESSION.clone(), source_row_count);
    rows.skip_rows_until_starting_row_number(starting_row_number);
    rows
}

/// Sales rows of sales `table`, starting at source row `starting_row_number`.
fn sales_rows<G: SalesRowGenerator>(
    generator: G,
    table: Table,
    starting_row_number: u64,
) -> impl Iterator<Item = G::Sales> {
    line_items(generator, table, starting_row_number).filter_map(|rows| rows.sales)
}

/// Returns rows of returns `table`, starting at source row `starting_row_number`.
fn returns_rows<G: SalesRowGenerator>(
    generator: G,
    table: Table,
    starting_row_number: u64,
) -> impl Iterator<Item = G::Returns> {
    line_items(generator, table, starting_row_number).filter_map(|rows| rows.returns)
}

/// Yields Arrow RecordBatches by writing `rows` in `format` and parsing the
/// result back to Arrow. `write_csv` writes one row as a CSV line.
fn reparsed_batches<R: Display>(
    mut rows: impl Iterator<Item = R>,
    write_csv: impl Fn(&R, &mut Vec<u8>),
    format: Format,
    table: Table,
    schema: &SchemaRef,
) -> impl Iterator<Item = RecordBatch> {
    let schema = Arc::clone(schema);

    const REPARSE_BUFFER_TARGET_BYTES: usize = 256 * 1024;
    std::iter::from_fn(move || {
        let mut data = Vec::new();
        format.write_header(table, &mut data);
        let header_len = data.len();

        while data.len() < REPARSE_BUFFER_TARGET_BYTES {
            let Some(row) = rows.next() else { break };
            format.write_row(&row, &write_csv, &mut data);
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
    // $rows: closure from a starting source row number to an iterator of the
    //        table's rows, built with `single_rows`, `sales_rows` or `returns_rows`.
    // $arrow_gen: constructor for the matching Arrow RecordBatch generator.
    // $table: TPC-DS table under test.
    // $csv: the table's `CsvRow` wrapper.
    ($name:ident, $rows:expr, $arrow_gen:expr, $table:expr, $csv:ident) => {
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
                let row_limit = test_row_count($table.source_table()) as usize;
                let arrow_gen = $arrow_gen(SESSION.clone());
                let schema = arrow_gen.schema();
                let reparsed = reparsed_batches(
                    $rows(1),
                    |row, data| writeln!(data, "{}", $csv::new(row)).unwrap(),
                    format,
                    $table,
                    &schema,
                );

                assert_record_batch_streams(arrow_gen, reparsed, row_limit);
            }

            /// Parse after skipping some rows.
            fn skip(format: Format) {
                let source_table = $table.source_table();
                let source_row_count = SESSION.get_scaling().get_row_count(source_table);
                let starting_row_number = source_row_count.min(100);
                let remaining_source_rows = source_row_count - starting_row_number + 1;
                let row_limit = test_row_count(source_table)
                    .min(remaining_source_rows)
                    .min(1024) as usize;

                let mut arrow_gen = $arrow_gen(SESSION.clone());
                arrow_gen.skip_rows_until_starting_row_number(starting_row_number);

                let schema = arrow_gen.schema();
                let reparsed = reparsed_batches(
                    $rows(starting_row_number),
                    |row, data| writeln!(data, "{}", $csv::new(row)).unwrap(),
                    format,
                    $table,
                    &schema,
                );

                assert_record_batch_streams(arrow_gen, reparsed, row_limit);
            }
        }
    };
}

table_test!(
    income_band,
    |start| single_rows(IncomeBandRowGenerator::new(), Table::IncomeBand, start),
    IncomeBandArrow::new,
    Table::IncomeBand,
    IncomeBandCsv
);
table_test!(
    reason,
    |start| single_rows(ReasonRowGenerator::new(), Table::Reason, start),
    ReasonArrow::new,
    Table::Reason,
    ReasonCsv
);
table_test!(
    ship_mode,
    |start| single_rows(ShipModeRowGenerator::new(), Table::ShipMode, start),
    ShipModeArrow::new,
    Table::ShipMode,
    ShipModeCsv
);
table_test!(
    inventory,
    |start| single_rows(InventoryRowGenerator::new(), Table::Inventory, start),
    InventoryArrow::new,
    Table::Inventory,
    InventoryCsv
);
table_test!(
    household_demographics,
    |start| single_rows(
        HouseholdDemographicsRowGenerator::new(),
        Table::HouseholdDemographics,
        start
    ),
    HouseholdDemographicsArrow::new,
    Table::HouseholdDemographics,
    HouseholdDemographicsCsv
);
table_test!(
    customer_demographics,
    |start| single_rows(
        CustomerDemographicsRowGenerator::new(),
        Table::CustomerDemographics,
        start
    ),
    CustomerDemographicsArrow::new,
    Table::CustomerDemographics,
    CustomerDemographicsCsv
);
table_test!(
    customer_address,
    |start| single_rows(
        CustomerAddressRowGenerator::new(),
        Table::CustomerAddress,
        start
    ),
    CustomerAddressArrow::new,
    Table::CustomerAddress,
    CustomerAddressCsv
);
table_test!(
    customer,
    |start| single_rows(CustomerRowGenerator::new(), Table::Customer, start),
    CustomerArrow::new,
    Table::Customer,
    CustomerCsv
);
table_test!(
    catalog_page,
    |start| single_rows(CatalogPageRowGenerator::new(), Table::CatalogPage, start),
    CatalogPageArrow::new,
    Table::CatalogPage,
    CatalogPageCsv
);
table_test!(
    time_dim,
    |start| single_rows(TimeDimRowGenerator::new(), Table::TimeDim, start),
    TimeDimArrow::new,
    Table::TimeDim,
    TimeDimCsv
);
table_test!(
    date_dim,
    |start| single_rows(DateDimRowGenerator::new(), Table::DateDim, start),
    DateDimArrow::new,
    Table::DateDim,
    DateDimCsv
);
table_test!(
    warehouse,
    |start| single_rows(WarehouseRowGenerator::new(), Table::Warehouse, start),
    WarehouseArrow::new,
    Table::Warehouse,
    WarehouseCsv
);
table_test!(
    item,
    |start| single_rows(ItemRowGenerator::new(), Table::Item, start),
    ItemArrow::new,
    Table::Item,
    ItemCsv
);
table_test!(
    promotion,
    |start| single_rows(PromotionRowGenerator::new(), Table::Promotion, start),
    PromotionArrow::new,
    Table::Promotion,
    PromotionCsv
);
table_test!(
    store,
    |start| single_rows(StoreRowGenerator::new(), Table::Store, start),
    StoreArrow::new,
    Table::Store,
    StoreCsv
);
table_test!(
    web_page,
    |start| single_rows(WebPageRowGenerator::new(), Table::WebPage, start),
    WebPageArrow::new,
    Table::WebPage,
    WebPageCsv
);
table_test!(
    web_site,
    |start| single_rows(WebSiteRowGenerator::new(), Table::WebSite, start),
    WebSiteArrow::new,
    Table::WebSite,
    WebSiteCsv
);
table_test!(
    call_center,
    |start| single_rows(CallCenterRowGenerator::new(), Table::CallCenter, start),
    CallCenterArrow::new,
    Table::CallCenter,
    CallCenterCsv
);

table_test!(
    catalog_sales,
    |start| sales_rows(
        CatalogSalesRowGenerator::sales(),
        Table::CatalogSales,
        start
    ),
    CatalogSalesArrow::new,
    Table::CatalogSales,
    CatalogSalesCsv
);
table_test!(
    catalog_returns,
    |start| returns_rows(
        CatalogSalesRowGenerator::returns(),
        Table::CatalogReturns,
        start
    ),
    CatalogReturnsArrow::new,
    Table::CatalogReturns,
    CatalogReturnsCsv
);
table_test!(
    store_sales,
    |start| sales_rows(StoreSalesRowGenerator::sales(), Table::StoreSales, start),
    StoreSalesArrow::new,
    Table::StoreSales,
    StoreSalesCsv
);
table_test!(
    store_returns,
    |start| returns_rows(
        StoreSalesRowGenerator::returns(),
        Table::StoreReturns,
        start
    ),
    StoreReturnsArrow::new,
    Table::StoreReturns,
    StoreReturnsCsv
);
table_test!(
    web_sales,
    |start| sales_rows(WebSalesRowGenerator::sales(), Table::WebSales, start),
    WebSalesArrow::new,
    Table::WebSales,
    WebSalesCsv
);
table_test!(
    web_returns,
    |start| returns_rows(WebSalesRowGenerator::returns(), Table::WebReturns, start),
    WebReturnsArrow::new,
    Table::WebReturns,
    WebReturnsCsv
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
