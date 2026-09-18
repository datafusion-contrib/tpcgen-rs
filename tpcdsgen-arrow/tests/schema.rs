//! Verifies canonical TPC-DS column names, ordering, data types, and nullability.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatchReader;
use tpcdsgen::config::{Scaling, Session, Table};
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

fn table_schemas(session: &Session) -> Vec<(Table, SchemaRef)> {
    vec![
        (
            Table::DbgenVersion,
            DbgenVersionArrow::new(session.clone()).schema(),
        ),
        (
            Table::CustomerAddress,
            CustomerAddressArrow::new(session.clone()).schema(),
        ),
        (
            Table::CustomerDemographics,
            CustomerDemographicsArrow::new(session.clone()).schema(),
        ),
        (Table::DateDim, DateDimArrow::new(session.clone()).schema()),
        (
            Table::Warehouse,
            WarehouseArrow::new(session.clone()).schema(),
        ),
        (
            Table::ShipMode,
            ShipModeArrow::new(session.clone()).schema(),
        ),
        (Table::TimeDim, TimeDimArrow::new(session.clone()).schema()),
        (Table::Reason, ReasonArrow::new(session.clone()).schema()),
        (
            Table::IncomeBand,
            IncomeBandArrow::new(session.clone()).schema(),
        ),
        (Table::Item, ItemArrow::new(session.clone()).schema()),
        (Table::Store, StoreArrow::new(session.clone()).schema()),
        (
            Table::CallCenter,
            CallCenterArrow::new(session.clone()).schema(),
        ),
        (
            Table::Customer,
            CustomerArrow::new(session.clone()).schema(),
        ),
        (Table::WebSite, WebSiteArrow::new(session.clone()).schema()),
        (
            Table::StoreReturns,
            StoreReturnsArrow::new(session.clone()).schema(),
        ),
        (
            Table::HouseholdDemographics,
            HouseholdDemographicsArrow::new(session.clone()).schema(),
        ),
        (Table::WebPage, WebPageArrow::new(session.clone()).schema()),
        (
            Table::Promotion,
            PromotionArrow::new(session.clone()).schema(),
        ),
        (
            Table::CatalogPage,
            CatalogPageArrow::new(session.clone()).schema(),
        ),
        (
            Table::Inventory,
            InventoryArrow::new(session.clone()).schema(),
        ),
        (
            Table::CatalogReturns,
            CatalogReturnsArrow::new(session.clone()).schema(),
        ),
        (
            Table::WebReturns,
            WebReturnsArrow::new(session.clone()).schema(),
        ),
        (
            Table::WebSales,
            WebSalesArrow::new(session.clone()).schema(),
        ),
        (
            Table::CatalogSales,
            CatalogSalesArrow::new(session.clone()).schema(),
        ),
        (
            Table::StoreSales,
            StoreSalesArrow::new(session.clone()).schema(),
        ),
    ]
}

#[test]
fn schemas_match_expected_columns_and_canonical_types() {
    let session = Session::default();

    for (table, schema) in table_schemas(&session) {
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
