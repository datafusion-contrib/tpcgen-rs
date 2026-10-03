/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! TPC-DS DAT output.
//!
//! Generates TPC-DS benchmark data with byte-for-byte compatibility with the Java reference.

use crate::generate::Source;
use crate::output_location::OutputLocation;
use tpcdsgen::config::CompatMode;
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output generator.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    /// Where to write the output
    pub(super) base_location: OutputLocation,
    /// Which reference implementation to match.
    pub(super) compat_mode: CompatMode,
    /// Target size of each generated buffer
    pub(super) chunk_size_bytes: i64,
}

impl Dat {
    pub(super) fn new(
        base_location: OutputLocation,
        compat_mode: CompatMode,
        chunk_size_bytes: i64,
    ) -> Result<Self> {
        Ok(Self {
            base_location,
            compat_mode,
            chunk_size_bytes,
        })
    }
}

/// Define a [`Source`] that writes `$ROWS` in DAT format
macro_rules! define_dat_source {
    ($SOURCE_NAME:ident, $ROWS:ty) => {
        pub(super) struct $SOURCE_NAME {
            rows: $ROWS,
            compat_mode: CompatMode,
        }

        impl $SOURCE_NAME {
            pub(super) fn new(rows: $ROWS, compat_mode: CompatMode) -> Self {
                Self { rows, compat_mode }
            }
        }

        impl Source for $SOURCE_NAME {
            /// DAT output has no header.
            fn header(&self, buffer: Vec<u8>) -> Vec<u8> {
                buffer
            }

            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                let mut writer = DatWriter::new(&mut buffer, self.compat_mode);
                for row in self.rows {
                    // Writing to memory cannot fail, and every generated value is
                    // representable in the output encoding (the distributions the
                    // values come from are themselves ISO-8859-1).
                    writer
                        .write_display_row(&row)
                        .expect("DAT rows are always writable to memory");
                }
                writer
                    .flush()
                    .expect("DAT rows are always writable to memory");
                drop(writer);

                buffer
            }
        }
    };
}

// Define .dat sources for all tables
define_dat_source!(CallCenterDatSource, CallCenterRowGenerator);
define_dat_source!(CatalogPageDatSource, CatalogPageRowGenerator);
define_dat_source!(CatalogReturnsDatSource, CatalogReturnsRowGenerator);
define_dat_source!(CatalogSalesDatSource, CatalogSalesRowGenerator);
define_dat_source!(CustomerDatSource, CustomerRowGenerator);
define_dat_source!(CustomerAddressDatSource, CustomerAddressRowGenerator);
define_dat_source!(
    CustomerDemographicsDatSource,
    CustomerDemographicsRowGenerator
);
define_dat_source!(DateDimDatSource, DateDimRowGenerator);
define_dat_source!(DbgenVersionDatSource, DbgenVersionRowGenerator);
define_dat_source!(
    HouseholdDemographicsDatSource,
    HouseholdDemographicsRowGenerator
);
define_dat_source!(IncomeBandDatSource, IncomeBandRowGenerator);
define_dat_source!(InventoryDatSource, InventoryRowGenerator);
define_dat_source!(ItemDatSource, ItemRowGenerator);
define_dat_source!(PromotionDatSource, PromotionRowGenerator);
define_dat_source!(ReasonDatSource, ReasonRowGenerator);
define_dat_source!(ShipModeDatSource, ShipModeRowGenerator);
define_dat_source!(StoreDatSource, StoreRowGenerator);
define_dat_source!(StoreReturnsDatSource, StoreReturnsRowGenerator);
define_dat_source!(StoreSalesDatSource, StoreSalesRowGenerator);
define_dat_source!(TimeDimDatSource, TimeDimRowGenerator);
define_dat_source!(WarehouseDatSource, WarehouseRowGenerator);
define_dat_source!(WebPageDatSource, WebPageRowGenerator);
define_dat_source!(WebReturnsDatSource, WebReturnsRowGenerator);
define_dat_source!(WebSalesDatSource, WebSalesRowGenerator);
define_dat_source!(WebSiteDatSource, WebSiteRowGenerator);
