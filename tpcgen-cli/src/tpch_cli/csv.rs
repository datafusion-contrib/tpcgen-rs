//! Implementations of [`Source`] for generating data in TBL format
use crate::generate::Source;
use std::io::Write;
use tpchgen::csv::{
    CustomerCsv, LineItemCsv, NationCsv, OrderCsv, PartCsv, PartSuppCsv, RegionCsv, SupplierCsv,
};
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSuppGenerator, RegionGenerator, SupplierGenerator,
};

/// Define a Source that writes the table in CSV format
macro_rules! define_csv_source {
    ($SOURCE_NAME:ident, $GENERATOR_TYPE:ty, $FORMATTER:ty) => {
        pub struct $SOURCE_NAME {
            inner: $GENERATOR_TYPE,
            delimiter: char,
        }

        impl $SOURCE_NAME {
            pub fn new(inner: $GENERATOR_TYPE, delimiter: char) -> Self {
                Self { inner, delimiter }
            }
        }

        impl Source for $SOURCE_NAME {
            fn header(&self, buffer: Vec<u8>) -> Vec<u8> {
                let mut buffer = buffer;
                writeln!(
                    &mut buffer,
                    "{}",
                    <$FORMATTER>::header_with_delimiter(self.delimiter)
                )
                .expect("writing to memory is infallible");
                buffer
            }

            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                for item in self.inner.iter() {
                    let formatter = <$FORMATTER>::with_delimiter(item, self.delimiter);
                    writeln!(&mut buffer, "{formatter}").expect("writing to memory is infallible");
                }
                buffer
            }
        }
    };
}

// Define .csv sources for all tables
define_csv_source!(NationCsvSource, NationGenerator<'static>, NationCsv);
define_csv_source!(RegionCsvSource, RegionGenerator<'static>, RegionCsv);
define_csv_source!(PartCsvSource, PartGenerator<'static>, PartCsv);
define_csv_source!(SupplierCsvSource, SupplierGenerator<'static>, SupplierCsv);
define_csv_source!(PartSuppCsvSource, PartSuppGenerator<'static>, PartSuppCsv);
define_csv_source!(CustomerCsvSource, CustomerGenerator<'static>, CustomerCsv);
define_csv_source!(OrderCsvSource, OrderGenerator<'static>, OrderCsv);
define_csv_source!(LineItemCsvSource, LineItemGenerator<'static>, LineItemCsv);
