//! Verifies canonical TPC-DS column names, ordering, data types, and nullability.
//!
//! CSV headers are covered transitively: `reparse.rs` re-parses CSV output
//! with header validation enabled against these same Arrow schemas.

use arrow::datatypes::SchemaRef;
use tpcdsgen::config::{Scaling, Table};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

#[path = "schema/expected.rs"]
mod expected;
use expected::expected_schema;

fn table_schemas() -> Vec<(Table, SchemaRef)> {
    vec![
        (Table::DbgenVersion, DbgenVersionArrow::schema_ref()),
        (Table::CustomerAddress, CustomerAddressArrow::schema_ref()),
        (
            Table::CustomerDemographics,
            CustomerDemographicsArrow::schema_ref(),
        ),
        (Table::DateDim, DateDimArrow::schema_ref()),
        (Table::Warehouse, WarehouseArrow::schema_ref()),
        (Table::ShipMode, ShipModeArrow::schema_ref()),
        (Table::TimeDim, TimeDimArrow::schema_ref()),
        (Table::Reason, ReasonArrow::schema_ref()),
        (Table::IncomeBand, IncomeBandArrow::schema_ref()),
        (Table::Item, ItemArrow::schema_ref()),
        (Table::Store, StoreArrow::schema_ref()),
        (Table::CallCenter, CallCenterArrow::schema_ref()),
        (Table::Customer, CustomerArrow::schema_ref()),
        (Table::WebSite, WebSiteArrow::schema_ref()),
        (Table::StoreReturns, StoreReturnsArrow::schema_ref()),
        (
            Table::HouseholdDemographics,
            HouseholdDemographicsArrow::schema_ref(),
        ),
        (Table::WebPage, WebPageArrow::schema_ref()),
        (Table::Promotion, PromotionArrow::schema_ref()),
        (Table::CatalogPage, CatalogPageArrow::schema_ref()),
        (Table::Inventory, InventoryArrow::schema_ref()),
        (Table::CatalogReturns, CatalogReturnsArrow::schema_ref()),
        (Table::WebReturns, WebReturnsArrow::schema_ref()),
        (Table::WebSales, WebSalesArrow::schema_ref()),
        (Table::CatalogSales, CatalogSalesArrow::schema_ref()),
        (Table::StoreSales, StoreSalesArrow::schema_ref()),
    ]
}

#[test]
fn schemas_match_expected_columns_and_canonical_types() {
    for (table, schema) in table_schemas() {
        let table_name = table.get_name();
        assert_eq!(schema.as_ref(), &expected_schema(table), "{table_name}");
    }
}

#[test]
fn sf100000_integer_domains_fit_i32() {
    let scaling = Scaling::new(100000.0);
    let max_i32 = u64::try_from(i32::MAX).expect("i32::MAX fits in u64");
    let integer_key_tables = [
        Table::CallCenter,
        Table::CatalogPage,
        Table::Customer,
        Table::CustomerAddress,
        Table::CustomerDemographics,
        Table::DateDim,
        Table::HouseholdDemographics,
        Table::IncomeBand,
        Table::Item,
        Table::Promotion,
        Table::Reason,
        Table::ShipMode,
        Table::Store,
        Table::TimeDim,
        Table::Warehouse,
        Table::WebPage,
        Table::WebSite,
    ];

    for table in integer_key_tables {
        assert!(
            scaling.get_row_count(table) <= max_i32,
            "{} exceeds the Arrow Int32 key domain at SF100000",
            table.get_name()
        );
    }

    for table in [Table::StoreSales, Table::CatalogSales, Table::WebSales] {
        assert!(
            scaling.get_row_count(table) > max_i32,
            "{} order identifiers require Arrow Int64 at SF100000",
            table.get_name()
        );
    }
}
