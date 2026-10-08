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
use std::ops::RangeInclusive;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, SessionBuilder, Table};
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

/// Sessions to test every table with.
///
/// Generating a row range costs the same at any scale factor, so large scale
/// factors are cheap.
static SESSIONS: LazyLock<[Session; 6]> = LazyLock::new(|| {
    [1.0, 10.0, 100.0, 1000.0, 10_000.0, 100_000.0].map(|scale_factor| {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .expect("valid session")
    })
});
const DAT_SEPARATOR: char = '|';
const CSV_SEPARATOR: char = ',';

/// The source row ranges (1-based, inclusive) to test for `table`.
///
/// Tables with at most `FULL_TABLE_SOURCE_ROWS` source rows are tested in
/// full. Larger tables are only spot-checked: `WINDOW_SOURCE_ROWS` rows from
/// the start, from the middle and from the end.
fn test_row_ranges(session: &Session, table: Table) -> Vec<RangeInclusive<u64>> {
    const FULL_TABLE_SOURCE_ROWS: u64 = 10_000;
    const WINDOW_SOURCE_ROWS: u64 = 500;

    let rows = session.get_scaling().get_row_count(table);
    if rows <= FULL_TABLE_SOURCE_ROWS {
        return vec![RangeInclusive::new(1, rows)];
    }
    [1, rows / 2, rows - WINDOW_SOURCE_ROWS + 1]
        .map(|start| start..=start + WINDOW_SOURCE_ROWS - 1)
        .to_vec()
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

/// Asserts that two streams of Arrow RecordBatches are logically equal and
/// not empty. `context` describes the case in failure messages.
///
/// It ignores any differences in how the rows are distributed across batches
/// by realigning the batches before comparison.
fn assert_record_batch_streams<L, R>(left: L, right: R, context: &str)
where
    L: RecordBatchReader,
    R: Iterator<Item = RecordBatch>,
{
    // Use FixedSizeBatches to align batch boundaries for comparison.
    let left = left.map(|batch| batch.expect("arrow generation should not fail"));
    let mut left = FixedSizeBatches::new(left);
    let mut right = FixedSizeBatches::new(right);

    // Compare the two streams, batch by batch.
    let mut compared_rows = 0;
    left.by_ref()
        .zip(right.by_ref())
        .for_each(|(left_batch, right_batch)| {
            compared_rows += left_batch.num_rows();
            assert_eq!(left_batch, right_batch, "{context}");
        });
    assert!(compared_rows > 0, "{context}: no rows compared");
    assert!(
        left.next().is_none(),
        "{context}: left stream produced extra batches"
    );
    assert!(
        right.next().is_none(),
        "{context}: right stream produced extra batches"
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
            fn dat() {
                check(Format::Dat);
            }

            #[test]
            fn csv() {
                check(Format::Csv);
            }

            /// Reparse each test row range at each test session.
            fn check(format: Format) {
                for session in SESSIONS.iter() {
                    let source_row_count = session.get_scaling().get_row_count($table);
                    for range in test_row_ranges(session, $table) {
                        let (start, end) = (*range.start(), *range.end());
                        let mut rows = <$gen>::new(session.clone(), source_row_count);
                        rows.set_source_row_range(start, end);
                        let arrow_gen =
                            $arrow_gen(session.clone()).with_source_row_range(start, end);

                        let schema = arrow_gen.schema();
                        let reparsed = reparsed_rows(
                            rows,
                            format,
                            &schema,
                            <$csv>::header_with_delimiter(CSV_SEPARATOR),
                            |row| <$csv>::with_delimiter(row, CSV_SEPARATOR).to_string(),
                        );

                        let context = format!(
                            "{format:?} at SF{}, source rows {range:?}",
                            session.get_scaling().get_scale(),
                        );
                        assert_record_batch_streams(arrow_gen, reparsed, &context);
                    }
                }
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
struct FixedSizeBatches<I> {
    /// The source of the RecordBatches.
    inner: I,
    /// The output batch size, except for the last batch.
    batch_size: usize,
    /// Partially output batch, if any.
    pending: Option<RecordBatch>,
}

impl<I> FixedSizeBatches<I> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            batch_size: 1024,
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
        let target_rows = self.batch_size;
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
            let schema = batches[0].schema();
            Some(concat_batches(&schema, &batches).expect("concatenate batches"))
        }
    }
}
