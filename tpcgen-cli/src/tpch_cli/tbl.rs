//! Implementations of [`Source`] for generating data in TBL format

use crate::generate::Source;
use std::io::Write;
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSuppGenerator, RegionGenerator, SupplierGenerator,
};

/// Define a Source that writes the table in TBL format
macro_rules! define_tbl_source {
    ($SOURCE_NAME:ident, $GENERATOR_TYPE:ty) => {
        pub struct $SOURCE_NAME {
            inner: $GENERATOR_TYPE,
            /// Whether each row keeps the trailing `|` that follows its last field
            trailing_delimiter: bool,
        }

        impl $SOURCE_NAME {
            pub fn new(inner: $GENERATOR_TYPE, trailing_delimiter: bool) -> Self {
                Self {
                    inner,
                    trailing_delimiter,
                }
            }
        }

        impl Source for $SOURCE_NAME {
            fn header(&self, buffer: Vec<u8>) -> Vec<u8> {
                // TBL source does not have a header
                buffer
            }

            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                for item in self.inner.iter() {
                    // The default Display impl writes TBL format, which ends
                    // each row with `|`
                    write!(&mut buffer, "{item}").expect("writing to memory is infallible");
                    if !self.trailing_delimiter {
                        // Remove exactly one final delimiter
                        if buffer.last() == Some(&b'|') {
                            buffer.pop();
                        }
                    }
                    buffer.push(b'\n');
                }
                buffer
            }
        }
    };
}

// Define .tbl sources for all tables
define_tbl_source!(NationTblSource, NationGenerator<'static>);
define_tbl_source!(RegionTblSource, RegionGenerator<'static>);
define_tbl_source!(PartTblSource, PartGenerator<'static>);
define_tbl_source!(SupplierTblSource, SupplierGenerator<'static>);
define_tbl_source!(PartSuppTblSource, PartSuppGenerator<'static>);
define_tbl_source!(CustomerTblSource, CustomerGenerator<'static>);
define_tbl_source!(OrderTblSource, OrderGenerator<'static>);
define_tbl_source!(LineItemTblSource, LineItemGenerator<'static>);
