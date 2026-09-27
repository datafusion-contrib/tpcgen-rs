pub mod abstract_row_generator;
pub mod generated_row;
pub mod row_generator;
pub mod row_iter;
pub mod table_row;
mod tables;

pub use abstract_row_generator::AbstractRowGenerator;
pub use generated_row::GeneratedRow;
pub use row_generator::{RowGenerator, RowGeneratorResult};
pub use row_iter::RowIter;

/// Splits a row's DAT line into its column values.
///
/// Test helper for the row tests, which assert on individual columns of the
/// `fmt::Display` (DAT) output.
#[cfg(test)]
pub(crate) fn dat_values(row: &impl std::fmt::Display) -> Vec<String> {
    row.to_string()
        .strip_suffix('|')
        .expect("DAT line ends with a field separator")
        .split('|')
        .map(str::to_string)
        .collect()
}
pub use tables::{
    call_center_row, call_center_row_generator, catalog_page_row, catalog_page_row_generator,
    catalog_returns_row, catalog_returns_row_generator, catalog_sales_row,
    catalog_sales_row_generator, customer_address_row, customer_address_row_generator,
    customer_demographics_row, customer_demographics_row_generator, customer_row,
    customer_row_generator, date_dim_row, date_dim_row_generator, dbgen_version_row,
    dbgen_version_row_generator, household_demographics_row, household_demographics_row_generator,
    income_band_row, income_band_row_generator, inventory_row, inventory_row_generator, item_row,
    item_row_generator, promotion_row, promotion_row_generator, reason_row, reason_row_generator,
    ship_mode_row, ship_mode_row_generator, store_returns_row, store_returns_row_generator,
    store_row, store_row_generator, store_sales_row, store_sales_row_generator, time_dim_row,
    time_dim_row_generator, warehouse_row, warehouse_row_generator, web_page_row,
    web_page_row_generator, web_returns_row, web_returns_row_generator, web_sales_row,
    web_sales_row_generator, web_site_row, web_site_row_generator,
};
pub use tables::{
    call_center_row::CallCenterRow, call_center_row_generator::CallCenterRowGenerator,
    catalog_page_row::CatalogPageRow, catalog_page_row_generator::CatalogPageRowGenerator,
    catalog_returns_row::CatalogReturnsRow,
    catalog_returns_row_generator::CatalogReturnsRowGenerator, catalog_sales_row::CatalogSalesRow,
    catalog_sales_row_generator::CatalogSalesRowGenerator,
    customer_address_row::CustomerAddressRow,
    customer_address_row_generator::CustomerAddressRowGenerator,
    customer_demographics_row::CustomerDemographicsRow,
    customer_demographics_row_generator::CustomerDemographicsRowGenerator,
    customer_row::CustomerRow, customer_row_generator::CustomerRowGenerator,
    date_dim_row::DateDimRow, date_dim_row_generator::DateDimRowGenerator,
    dbgen_version_row::DbgenVersionRow, dbgen_version_row_generator::DbgenVersionRowGenerator,
    household_demographics_row::HouseholdDemographicsRow,
    household_demographics_row_generator::HouseholdDemographicsRowGenerator,
    income_band_row::IncomeBandRow, income_band_row_generator::IncomeBandRowGenerator,
    inventory_row::InventoryRow, inventory_row_generator::InventoryRowGenerator, item_row::ItemRow,
    item_row_generator::ItemRowGenerator, promotion_row::PromotionRow,
    promotion_row_generator::PromotionRowGenerator, reason_row::ReasonRow,
    reason_row_generator::ReasonRowGenerator, ship_mode_row::ShipModeRow,
    ship_mode_row_generator::ShipModeRowGenerator, store_returns_row::StoreReturnsRow,
    store_returns_row_generator::StoreReturnsRowGenerator, store_row::StoreRow,
    store_row_generator::StoreRowGenerator, store_sales_row::StoreSalesRow,
    store_sales_row_generator::StoreSalesRowGenerator, time_dim_row::TimeDimRow,
    time_dim_row_generator::TimeDimRowGenerator, warehouse_row::WarehouseRow,
    warehouse_row_generator::WarehouseRowGenerator, web_page_row::WebPageRow,
    web_page_row_generator::WebPageRowGenerator, web_returns_row::WebReturnsRow,
    web_returns_row_generator::WebReturnsRowGenerator, web_sales_row::WebSalesRow,
    web_sales_row_generator::WebSalesRowGenerator, web_site_row::WebSiteRow,
    web_site_row_generator::WebSiteRowGenerator,
};
