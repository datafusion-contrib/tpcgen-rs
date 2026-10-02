//! TPC-DS CSV output.
//!
//! Rows are formatted via the [`tpcdsgen::csv::CsvRow`] wrappers (the same
//! model as the TPC-H CSV output): one header line, then one line per row
//! with the same field values as the DAT output, joined by the delimiter
//! with no trailing separator. Free-text columns that can contain the
//! delimiter are double-quoted.
//!
//! Two deliberate differences from the DAT output, documented in more detail
//! on `tpcdsgen::csv`:
//!
//! * Output is UTF-8 in both compat modes, where the DAT output is ISO-8859-1
//!   in `CompatMode::Trino`. The values match as characters, not as bytes.
//! * Quoting is a fixed per-column property rather than quote-when-needed, so
//!   `--delimiter` is only safe for delimiters that no unquoted column
//!   contains (`,`, `|`, tab, `;`).

use crate::generate::Source;
use crate::output_location::OutputLocation;
use std::io::Write;
use tpcdsgen::csv::*;
use tpcdsgen::row::*;

/// CSV output generator.
#[derive(Debug, Clone)]
pub(super) struct Csv {
    pub(super) base_location: OutputLocation,
    pub(super) delimiter: char,
    pub(super) chunk_size_bytes: i64,
}

impl Csv {
    pub(super) fn new(
        base_location: OutputLocation,
        delimiter: char,
        chunk_size_bytes: i64,
    ) -> Self {
        Self {
            base_location,
            delimiter,
            chunk_size_bytes,
        }
    }
}

/// Define a [`Source`] that writes `$ROWS` as CSV lines via the `$CSV`
/// [`CsvRow`] wrapper
macro_rules! define_csv_source {
    ($SOURCE_NAME:ident, $ROWS:ty, $CSV:ident) => {
        pub(super) struct $SOURCE_NAME {
            rows: $ROWS,
            delimiter: char,
        }

        impl $SOURCE_NAME {
            pub(super) fn new(rows: $ROWS, delimiter: char) -> Self {
                Self { rows, delimiter }
            }
        }

        impl Source for $SOURCE_NAME {
            fn header(&self, mut buffer: Vec<u8>) -> Vec<u8> {
                writeln!(buffer, "{}", $CSV::header_with_delimiter(self.delimiter))
                    .expect("writing to memory cannot fail");
                buffer
            }

            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                for row in self.rows {
                    writeln!(buffer, "{}", $CSV::with_delimiter(&row, self.delimiter))
                        .expect("writing to memory cannot fail");
                }
                buffer
            }
        }
    };
}

// Define .csv sources for all tables
define_csv_source!(
    CallCenterCsvSource,
    SingleRowIter<CallCenterRowGenerator>,
    CallCenterCsv
);
define_csv_source!(
    CatalogPageCsvSource,
    SingleRowIter<CatalogPageRowGenerator>,
    CatalogPageCsv
);
define_csv_source!(
    CatalogReturnsCsvSource,
    CatalogReturnsRowGenerator,
    CatalogReturnsCsv
);
define_csv_source!(
    CatalogSalesCsvSource,
    CatalogSalesRowGenerator,
    CatalogSalesCsv
);
define_csv_source!(
    CustomerCsvSource,
    SingleRowIter<CustomerRowGenerator>,
    CustomerCsv
);
define_csv_source!(
    CustomerAddressCsvSource,
    SingleRowIter<CustomerAddressRowGenerator>,
    CustomerAddressCsv
);
define_csv_source!(
    CustomerDemographicsCsvSource,
    SingleRowIter<CustomerDemographicsRowGenerator>,
    CustomerDemographicsCsv
);
define_csv_source!(
    DateDimCsvSource,
    SingleRowIter<DateDimRowGenerator>,
    DateDimCsv
);
define_csv_source!(
    DbgenVersionCsvSource,
    SingleRowIter<DbgenVersionRowGenerator>,
    DbgenVersionCsv
);
define_csv_source!(
    HouseholdDemographicsCsvSource,
    SingleRowIter<HouseholdDemographicsRowGenerator>,
    HouseholdDemographicsCsv
);
define_csv_source!(
    IncomeBandCsvSource,
    SingleRowIter<IncomeBandRowGenerator>,
    IncomeBandCsv
);
define_csv_source!(
    InventoryCsvSource,
    SingleRowIter<InventoryRowGenerator>,
    InventoryCsv
);
define_csv_source!(ItemCsvSource, SingleRowIter<ItemRowGenerator>, ItemCsv);
define_csv_source!(
    PromotionCsvSource,
    SingleRowIter<PromotionRowGenerator>,
    PromotionCsv
);
define_csv_source!(
    ReasonCsvSource,
    SingleRowIter<ReasonRowGenerator>,
    ReasonCsv
);
define_csv_source!(
    ShipModeCsvSource,
    SingleRowIter<ShipModeRowGenerator>,
    ShipModeCsv
);
define_csv_source!(StoreCsvSource, SingleRowIter<StoreRowGenerator>, StoreCsv);
define_csv_source!(
    StoreReturnsCsvSource,
    StoreReturnsRowGenerator,
    StoreReturnsCsv
);
define_csv_source!(StoreSalesCsvSource, StoreSalesRowGenerator, StoreSalesCsv);
define_csv_source!(
    TimeDimCsvSource,
    SingleRowIter<TimeDimRowGenerator>,
    TimeDimCsv
);
define_csv_source!(
    WarehouseCsvSource,
    SingleRowIter<WarehouseRowGenerator>,
    WarehouseCsv
);
define_csv_source!(
    WebPageCsvSource,
    SingleRowIter<WebPageRowGenerator>,
    WebPageCsv
);
define_csv_source!(WebReturnsCsvSource, WebReturnsRowGenerator, WebReturnsCsv);
define_csv_source!(WebSalesCsvSource, WebSalesRowGenerator, WebSalesCsv);
define_csv_source!(
    WebSiteCsvSource,
    SingleRowIter<WebSiteRowGenerator>,
    WebSiteCsv
);
