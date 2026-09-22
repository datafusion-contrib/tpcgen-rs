//! Verifies the correctness of the Arrow TPCH generator by parsing the canonical TBL format
//! and comparing with the generated Arrow RecordBatches

// See the `tpchgen_arrow` crate documentation for the `arrow_59` / `arrow_60`
// feature flags; the selected versions are aliased back to `arrow` and
// `arrow_csv` here.
#[cfg(feature = "arrow_59")]
extern crate arrow_59 as arrow;
#[cfg(feature = "arrow_60")]
extern crate arrow_60 as arrow;
#[cfg(feature = "arrow_59")]
extern crate arrow_csv_59 as arrow_csv;
#[cfg(feature = "arrow_60")]
extern crate arrow_csv_60 as arrow_csv;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatchReader;
use std::io::Write;
use std::sync::Arc;
use tpchgen::csv::{
    CustomerCsv, LineItemCsv, NationCsv, OrderCsv, PartCsv, PartSuppCsv, RegionCsv, SupplierCsv,
};
use tpchgen::generators::{
    Customer, CustomerGenerator, LineItem, LineItemGenerator, Nation, NationGenerator, Order,
    OrderGenerator, Part, PartGenerator, PartSupp, PartSuppGenerator, Region, RegionGenerator,
    Supplier, SupplierGenerator,
};
use tpchgen_arrow::{
    CustomerArrow, LineItemArrow, NationArrow, OrderArrow, PartArrow, PartSuppArrow, RegionArrow,
    SupplierArrow,
};

/// Macro that defines tests for tbl for a given type
macro_rules! test_row_type {
    ($FUNCNAME:ident, $GENERATOR:ty, $ARROWITER:ty, $FORMATTYPE:expr) => {
        #[test]
        fn $FUNCNAME() {
            let scale_factor = 0.1;
            let batch_size = 1000;
            let part = 1;
            let part_count = 1;
            let generator = <$GENERATOR>::new(scale_factor, part, part_count);
            $FORMATTYPE.test(
                generator.clone().iter(),
                <$ARROWITER>::new(generator).with_batch_size(batch_size),
            );
        }
    };
}

test_row_type!(customer_tbl, CustomerGenerator, CustomerArrow, Test::tbl());
test_row_type!(customer_csv, CustomerGenerator, CustomerArrow, Test::csv());
test_row_type!(lineitem_tbl, LineItemGenerator, LineItemArrow, Test::tbl());
test_row_type!(lineitem_csv, LineItemGenerator, LineItemArrow, Test::csv());
test_row_type!(nation_tbl, NationGenerator, NationArrow, Test::tbl());
test_row_type!(nation_csv, NationGenerator, NationArrow, Test::csv());
test_row_type!(order_tbl, OrderGenerator, OrderArrow, Test::tbl());
test_row_type!(order_csv, OrderGenerator, OrderArrow, Test::csv());
test_row_type!(part_tbl, PartGenerator, PartArrow, Test::tbl());
test_row_type!(part_csv, PartGenerator, PartArrow, Test::csv());
test_row_type!(partsupp_tbl, PartSuppGenerator, PartSuppArrow, Test::tbl());
test_row_type!(partsupp_csv, PartSuppGenerator, PartSuppArrow, Test::csv());
test_row_type!(region_tbl, RegionGenerator, RegionArrow, Test::tbl());
test_row_type!(region_csv, RegionGenerator, RegionArrow, Test::csv());
test_row_type!(supplier_tbl, SupplierGenerator, SupplierArrow, Test::tbl());
test_row_type!(supplier_csv, SupplierGenerator, SupplierArrow, Test::csv());

/// Common trait for writing rows in TBL and CSV format
trait RowType {
    /// write a row in TBL format, WITHOUT newline
    fn write_tbl_row(self, text_data: &mut Vec<u8>);
    /// write the header in CSV format
    fn write_csv_header(text_data: &mut Vec<u8>);
    /// write a row in CSV format, WITH newline
    fn write_csv_row(self, text_data: &mut Vec<u8>);
}

/// Macro that implements the RowType trait for a given type
macro_rules! impl_row_type {
    ($type:ty, $csv_type:ty) => {
        impl RowType for $type {
            fn write_tbl_row(self, text_data: &mut Vec<u8>) {
                write!(text_data, "{}", self).unwrap();
            }
            fn write_csv_header(text_data: &mut Vec<u8>) {
                writeln!(text_data, "{}", <$csv_type>::header()).unwrap();
            }
            fn write_csv_row(self, text_data: &mut Vec<u8>) {
                writeln!(text_data, "{}", <$csv_type>::new(self)).unwrap();
            }
        }
    };
}

impl_row_type!(Customer<'_>, CustomerCsv);
impl_row_type!(LineItem<'_>, LineItemCsv);
impl_row_type!(Nation<'_>, NationCsv);
impl_row_type!(Order<'_>, OrderCsv);
impl_row_type!(Part<'_>, PartCsv);
impl_row_type!(PartSupp<'_>, PartSuppCsv);
impl_row_type!(Region<'_>, RegionCsv);
impl_row_type!(Supplier, SupplierCsv);

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum Test {
    /// Generate and parse data as TBL format ('|' delimited)
    TBL,
    /// Generate and parse data as CSV format
    CSV,
}

impl Test {
    /// Create a test for TBL format
    fn tbl() -> Self {
        Self::TBL
    }

    /// Create a test for CSV format
    fn csv() -> Self {
        Self::CSV
    }

    /// Generates data using the row iterator and the arrow iterator using the
    /// specified format and compares the results of parsing with the Arrow CSV
    /// parser with the directly generated the batches
    fn test<R, RI, RBI>(&self, mut row_iter: RI, mut arrow_iter: RBI)
    where
        R: RowType,
        RI: Iterator<Item = R>,
        RBI: RecordBatchReader,
    {
        // For each batch generated by the arrow iterator
        while let Some(arrow_batch) = arrow_iter.next() {
            let arrow_batch = arrow_batch.expect("arrow generation should not fail");
            // Generate the same number of rows using the row iterator
            // in the specified format
            let batch_size = arrow_batch.num_rows();
            let mut text_data = Vec::new();
            self.write_header::<R>(&mut text_data);
            row_iter.by_ref().take(batch_size).for_each(|item| {
                self.write_row(item, &mut text_data);
            });
            // reparse the generated data and compare with the arrow batch
            let schema = arrow_iter.schema();
            let reparsed_batch = self.parse(&text_data, &schema, batch_size);
            assert_eq!(reparsed_batch, arrow_batch);
        }
    }

    /// Write the header for the row type
    fn write_header<R: RowType>(&self, text_data: &mut Vec<u8>) {
        match self {
            Test::TBL => {}
            Test::CSV => {
                R::write_csv_header(text_data);
            }
        }
    }

    /// Write a row into the provided buffer
    fn write_row<R: RowType>(&self, row: R, text_data: &mut Vec<u8>) {
        match self {
            Test::TBL => {
                row.write_tbl_row(text_data);
                // Note: TBL lines end with '|' which the arrow csv parser treats as a
                // delimiter for a new column so replace the last '|' with a newline
                let end_offset = text_data.len() - 1;
                text_data[end_offset] = b'\n';
            }
            Test::CSV => {
                row.write_csv_row(text_data);
            }
        }
    }

    /// Parse the provided data into an Arrow RecordBatch
    fn parse(&self, data: &[u8], schema: &SchemaRef, batch_size: usize) -> RecordBatch {
        let builder =
            arrow_csv::reader::ReaderBuilder::new(Arc::clone(schema)).with_batch_size(batch_size);

        let builder = match self {
            Test::TBL => builder.with_header(false).with_delimiter(b'|'),
            Test::CSV => builder.with_header(true),
        };

        let mut parser = builder.build(data).unwrap();

        let batch = parser
            .next()
            .expect("should have a batch")
            .expect("should have no errors parsing");
        assert!(parser.next().is_none(), "should have only one batch");
        batch
    }
}
