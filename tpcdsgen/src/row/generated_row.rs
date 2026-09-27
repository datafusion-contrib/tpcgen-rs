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

//! Generated row enum for static dispatch and zero-allocation row storage.
//!
//! This enum eliminates the need for a boxed trait object in the hot path,
//! avoiding heap allocations and enabling static dispatch.
//!
//! See ISSUE-004 for details.

use crate::config::Table;
use crate::row::{
    CallCenterRow, CatalogPageRow, CatalogReturnsRow, CatalogSalesRow, CustomerAddressRow,
    CustomerDemographicsRow, CustomerRow, DateDimRow, DbgenVersionRow, HouseholdDemographicsRow,
    IncomeBandRow, InventoryRow, ItemRow, PromotionRow, ReasonRow, ShipModeRow, StoreReturnsRow,
    StoreRow, StoreSalesRow, TimeDimRow, WarehouseRow, WebPageRow, WebReturnsRow, WebSalesRow,
    WebSiteRow,
};
use std::fmt;

/// Enum holding all possible generated row types.
///
/// This enables static dispatch and eliminates heap allocations that would
/// be required with a boxed trait object.
#[derive(Clone)]
pub enum GeneratedRow {
    CallCenter(CallCenterRow),
    CatalogPage(CatalogPageRow),
    CatalogReturns(CatalogReturnsRow),
    CatalogSales(CatalogSalesRow),
    Customer(CustomerRow),
    CustomerAddress(CustomerAddressRow),
    CustomerDemographics(CustomerDemographicsRow),
    DateDim(DateDimRow),
    DbgenVersion(DbgenVersionRow),
    HouseholdDemographics(HouseholdDemographicsRow),
    IncomeBand(IncomeBandRow),
    Inventory(InventoryRow),
    Item(ItemRow),
    Promotion(PromotionRow),
    Reason(ReasonRow),
    ShipMode(ShipModeRow),
    Store(StoreRow),
    StoreReturns(StoreReturnsRow),
    StoreSales(StoreSalesRow),
    TimeDim(TimeDimRow),
    Warehouse(WarehouseRow),
    WebPage(WebPageRow),
    WebReturns(WebReturnsRow),
    WebSales(WebSalesRow),
    WebSite(WebSiteRow),
}

impl GeneratedRow {
    /// The table this row belongs to.
    ///
    /// The sales generators emit rows for two tables (sales and returns); This
    /// function can be used to filter for rows from only one table.
    pub fn table(&self) -> Table {
        match self {
            GeneratedRow::CallCenter(_) => Table::CallCenter,
            GeneratedRow::CatalogPage(_) => Table::CatalogPage,
            GeneratedRow::CatalogReturns(_) => Table::CatalogReturns,
            GeneratedRow::CatalogSales(_) => Table::CatalogSales,
            GeneratedRow::Customer(_) => Table::Customer,
            GeneratedRow::CustomerAddress(_) => Table::CustomerAddress,
            GeneratedRow::CustomerDemographics(_) => Table::CustomerDemographics,
            GeneratedRow::DateDim(_) => Table::DateDim,
            GeneratedRow::DbgenVersion(_) => Table::DbgenVersion,
            GeneratedRow::HouseholdDemographics(_) => Table::HouseholdDemographics,
            GeneratedRow::IncomeBand(_) => Table::IncomeBand,
            GeneratedRow::Inventory(_) => Table::Inventory,
            GeneratedRow::Item(_) => Table::Item,
            GeneratedRow::Promotion(_) => Table::Promotion,
            GeneratedRow::Reason(_) => Table::Reason,
            GeneratedRow::ShipMode(_) => Table::ShipMode,
            GeneratedRow::Store(_) => Table::Store,
            GeneratedRow::StoreReturns(_) => Table::StoreReturns,
            GeneratedRow::StoreSales(_) => Table::StoreSales,
            GeneratedRow::TimeDim(_) => Table::TimeDim,
            GeneratedRow::Warehouse(_) => Table::Warehouse,
            GeneratedRow::WebPage(_) => Table::WebPage,
            GeneratedRow::WebReturns(_) => Table::WebReturns,
            GeneratedRow::WebSales(_) => Table::WebSales,
            GeneratedRow::WebSite(_) => Table::WebSite,
        }
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator (no newline), delegating to the variant's `Display` impl.
impl fmt::Display for GeneratedRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneratedRow::CallCenter(row) => row.fmt(f),
            GeneratedRow::CatalogPage(row) => row.fmt(f),
            GeneratedRow::CatalogReturns(row) => row.fmt(f),
            GeneratedRow::CatalogSales(row) => row.fmt(f),
            GeneratedRow::Customer(row) => row.fmt(f),
            GeneratedRow::CustomerAddress(row) => row.fmt(f),
            GeneratedRow::CustomerDemographics(row) => row.fmt(f),
            GeneratedRow::DateDim(row) => row.fmt(f),
            GeneratedRow::DbgenVersion(row) => row.fmt(f),
            GeneratedRow::HouseholdDemographics(row) => row.fmt(f),
            GeneratedRow::IncomeBand(row) => row.fmt(f),
            GeneratedRow::Inventory(row) => row.fmt(f),
            GeneratedRow::Item(row) => row.fmt(f),
            GeneratedRow::Promotion(row) => row.fmt(f),
            GeneratedRow::Reason(row) => row.fmt(f),
            GeneratedRow::ShipMode(row) => row.fmt(f),
            GeneratedRow::Store(row) => row.fmt(f),
            GeneratedRow::StoreReturns(row) => row.fmt(f),
            GeneratedRow::StoreSales(row) => row.fmt(f),
            GeneratedRow::TimeDim(row) => row.fmt(f),
            GeneratedRow::Warehouse(row) => row.fmt(f),
            GeneratedRow::WebPage(row) => row.fmt(f),
            GeneratedRow::WebReturns(row) => row.fmt(f),
            GeneratedRow::WebSales(row) => row.fmt(f),
            GeneratedRow::WebSite(row) => row.fmt(f),
        }
    }
}

// Convenience From implementations for easy conversion
impl From<CallCenterRow> for GeneratedRow {
    fn from(row: CallCenterRow) -> Self {
        GeneratedRow::CallCenter(row)
    }
}

impl From<CatalogPageRow> for GeneratedRow {
    fn from(row: CatalogPageRow) -> Self {
        GeneratedRow::CatalogPage(row)
    }
}

impl From<CatalogReturnsRow> for GeneratedRow {
    fn from(row: CatalogReturnsRow) -> Self {
        GeneratedRow::CatalogReturns(row)
    }
}

impl From<CatalogSalesRow> for GeneratedRow {
    fn from(row: CatalogSalesRow) -> Self {
        GeneratedRow::CatalogSales(row)
    }
}

impl From<CustomerRow> for GeneratedRow {
    fn from(row: CustomerRow) -> Self {
        GeneratedRow::Customer(row)
    }
}

impl From<CustomerAddressRow> for GeneratedRow {
    fn from(row: CustomerAddressRow) -> Self {
        GeneratedRow::CustomerAddress(row)
    }
}

impl From<CustomerDemographicsRow> for GeneratedRow {
    fn from(row: CustomerDemographicsRow) -> Self {
        GeneratedRow::CustomerDemographics(row)
    }
}

impl From<DateDimRow> for GeneratedRow {
    fn from(row: DateDimRow) -> Self {
        GeneratedRow::DateDim(row)
    }
}

impl From<DbgenVersionRow> for GeneratedRow {
    fn from(row: DbgenVersionRow) -> Self {
        GeneratedRow::DbgenVersion(row)
    }
}

impl From<HouseholdDemographicsRow> for GeneratedRow {
    fn from(row: HouseholdDemographicsRow) -> Self {
        GeneratedRow::HouseholdDemographics(row)
    }
}

impl From<IncomeBandRow> for GeneratedRow {
    fn from(row: IncomeBandRow) -> Self {
        GeneratedRow::IncomeBand(row)
    }
}

impl From<InventoryRow> for GeneratedRow {
    fn from(row: InventoryRow) -> Self {
        GeneratedRow::Inventory(row)
    }
}

impl From<ItemRow> for GeneratedRow {
    fn from(row: ItemRow) -> Self {
        GeneratedRow::Item(row)
    }
}

impl From<PromotionRow> for GeneratedRow {
    fn from(row: PromotionRow) -> Self {
        GeneratedRow::Promotion(row)
    }
}

impl From<ReasonRow> for GeneratedRow {
    fn from(row: ReasonRow) -> Self {
        GeneratedRow::Reason(row)
    }
}

impl From<ShipModeRow> for GeneratedRow {
    fn from(row: ShipModeRow) -> Self {
        GeneratedRow::ShipMode(row)
    }
}

impl From<StoreRow> for GeneratedRow {
    fn from(row: StoreRow) -> Self {
        GeneratedRow::Store(row)
    }
}

impl From<StoreReturnsRow> for GeneratedRow {
    fn from(row: StoreReturnsRow) -> Self {
        GeneratedRow::StoreReturns(row)
    }
}

impl From<StoreSalesRow> for GeneratedRow {
    fn from(row: StoreSalesRow) -> Self {
        GeneratedRow::StoreSales(row)
    }
}

impl From<TimeDimRow> for GeneratedRow {
    fn from(row: TimeDimRow) -> Self {
        GeneratedRow::TimeDim(row)
    }
}

impl From<WarehouseRow> for GeneratedRow {
    fn from(row: WarehouseRow) -> Self {
        GeneratedRow::Warehouse(row)
    }
}

impl From<WebPageRow> for GeneratedRow {
    fn from(row: WebPageRow) -> Self {
        GeneratedRow::WebPage(row)
    }
}

impl From<WebReturnsRow> for GeneratedRow {
    fn from(row: WebReturnsRow) -> Self {
        GeneratedRow::WebReturns(row)
    }
}

impl From<WebSalesRow> for GeneratedRow {
    fn from(row: WebSalesRow) -> Self {
        GeneratedRow::WebSales(row)
    }
}

impl From<WebSiteRow> for GeneratedRow {
    fn from(row: WebSiteRow) -> Self {
        GeneratedRow::WebSite(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_row_from_call_center() {
        let row = CallCenterRow::builder().build();
        let generated: GeneratedRow = row.into();
        assert!(matches!(generated, GeneratedRow::CallCenter(_)));
    }

    #[test]
    fn test_generated_row_display_matches_row_display() {
        let row = CallCenterRow::builder().build();
        let generated: GeneratedRow = row.clone().into();

        // The enum's Display delegates to the variant's Display.
        assert_eq!(row.to_string(), generated.to_string());
    }

    #[test]
    fn table_reports_the_variant_s_table() {
        let generated: GeneratedRow = CallCenterRow::builder().build().into();
        assert_eq!(generated.table(), Table::CallCenter);
    }
}
