use arrow::datatypes::{DataType, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatchReader;
use std::collections::BTreeSet;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::csv::csv_header;
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

fn schema_fingerprint(schema: &SchemaRef) -> u64 {
    let descriptor = schema
        .fields()
        .iter()
        .map(|field| {
            let data_type = match field.data_type() {
                DataType::Int32 | DataType::Int64 => "integer".to_string(),
                DataType::Utf8View => "string".to_string(),
                DataType::Date32 => "date".to_string(),
                DataType::Time32(TimeUnit::Second) => "time".to_string(),
                DataType::Decimal128(precision, scale) => {
                    format!("decimal({precision},{scale})")
                }
                data_type => panic!("unsupported TPC-DS Arrow type: {data_type}"),
            };
            let nullability = if field.is_nullable() {
                "nullable"
            } else {
                "required"
            };
            format!("{}:{data_type}:{nullability}", field.name())
        })
        .collect::<Vec<_>>()
        .join(",");

    descriptor.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

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
fn schemas_match_canonical_c_kit_columns_and_decimals() {
    let expected = [
        ("dbgen_version", 4, 0x6997614743317b61),
        ("customer_address", 13, 0x3ee32ab1c4b84f75),
        ("customer_demographics", 9, 0x081d435d116f96eb),
        ("date_dim", 28, 0x0ccaa61a6b82beb9),
        ("warehouse", 14, 0xfddd442664ff5df5),
        ("ship_mode", 6, 0x8b98c2661c942a22),
        ("time_dim", 10, 0x95f53373375ebfb8),
        ("reason", 3, 0x34ce10a37a6e0b10),
        ("income_band", 3, 0xbf4692a260910723),
        ("item", 22, 0x402b6fe0f746f141),
        ("store", 29, 0xe3bc1d5f6de25aa7),
        ("call_center", 31, 0x5382dbc7050ac76f),
        ("customer", 18, 0x8a6c5a1ace467874),
        ("web_site", 26, 0xa13f4fe04fc201c8),
        ("store_returns", 20, 0x16fb0323450fd25d),
        ("household_demographics", 5, 0xfe50aa17e8b90df2),
        ("web_page", 14, 0x68f749360f62cfc5),
        ("promotion", 19, 0x014f46fbc2444ab2),
        ("catalog_page", 9, 0x5ce26c04939b7a7c),
        ("inventory", 4, 0x4b606ddb03b624d1),
        ("catalog_returns", 27, 0xc9eeed5980d11fa3),
        ("web_returns", 24, 0x660a2030b37af161),
        ("web_sales", 34, 0x427a46604c1e75d8),
        ("catalog_sales", 34, 0x31169f631b8f1e43),
        ("store_sales", 23, 0x9d46b5a676f193ba),
    ];
    let session = Session::default();
    let mut visited = BTreeSet::new();
    let mut mismatches = Vec::new();

    for (table, schema) in table_schemas(&session) {
        let table_name = table.get_name();
        assert!(
            visited.insert(table_name),
            "duplicate schema for {table_name}"
        );
        let (_, expected_columns, expected_fingerprint) = expected
            .iter()
            .find(|(name, _, _)| *name == table_name)
            .expect("canonical C-kit table");
        let arrow_header = schema
            .fields()
            .iter()
            .map(|field| field.name().as_str())
            .collect::<Vec<_>>()
            .join(",");

        assert_eq!(schema.fields().len(), *expected_columns, "{table_name}");
        let actual_fingerprint = schema_fingerprint(&schema);
        if actual_fingerprint != *expected_fingerprint {
            mismatches.push((table_name, actual_fingerprint, *expected_fingerprint));
        }
        assert_eq!(
            csv_header(table, ',').expect("CSV header"),
            arrow_header,
            "{table_name}"
        );
    }

    let expected_tables = expected
        .iter()
        .map(|(name, _, _)| *name)
        .collect::<BTreeSet<_>>();
    assert_eq!(visited, expected_tables);
    assert!(mismatches.is_empty(), "{mismatches:#x?}");
}

fn assert_decimal_fields(schema: SchemaRef, expected: &[(&str, u8)]) {
    let actual: Vec<_> = schema
        .fields()
        .iter()
        .filter_map(|field| match field.data_type() {
            DataType::Decimal128(precision, scale) => {
                Some((field.name().as_str(), *precision, *scale))
            }
            _ => None,
        })
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|(name, precision)| (*name, *precision, 2))
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn decimal_schemas_match_canonical_c_kit() {
    let session = Session::default();

    assert_decimal_fields(
        CallCenterArrow::new(session.clone()).schema(),
        &[("cc_gmt_offset", 5), ("cc_tax_percentage", 5)],
    );
    assert_decimal_fields(
        CatalogReturnsArrow::new(session.clone()).schema(),
        &[
            ("cr_return_amount", 7),
            ("cr_return_tax", 7),
            ("cr_return_amt_inc_tax", 7),
            ("cr_fee", 7),
            ("cr_return_ship_cost", 7),
            ("cr_refunded_cash", 7),
            ("cr_reversed_charge", 7),
            ("cr_store_credit", 7),
            ("cr_net_loss", 7),
        ],
    );
    assert_decimal_fields(
        CatalogSalesArrow::new(session.clone()).schema(),
        &[
            ("cs_wholesale_cost", 7),
            ("cs_list_price", 7),
            ("cs_sales_price", 7),
            ("cs_ext_discount_amt", 7),
            ("cs_ext_sales_price", 7),
            ("cs_ext_wholesale_cost", 7),
            ("cs_ext_list_price", 7),
            ("cs_ext_tax", 7),
            ("cs_coupon_amt", 7),
            ("cs_ext_ship_cost", 7),
            ("cs_net_paid", 7),
            ("cs_net_paid_inc_tax", 7),
            ("cs_net_paid_inc_ship", 7),
            ("cs_net_paid_inc_ship_tax", 7),
            ("cs_net_profit", 7),
        ],
    );
    assert_decimal_fields(
        CustomerAddressArrow::new(session.clone()).schema(),
        &[("ca_gmt_offset", 5)],
    );
    assert_decimal_fields(
        ItemArrow::new(session.clone()).schema(),
        &[("i_current_price", 7), ("i_wholesale_cost", 7)],
    );
    assert_decimal_fields(
        PromotionArrow::new(session.clone()).schema(),
        &[("p_cost", 15)],
    );
    assert_decimal_fields(
        StoreArrow::new(session.clone()).schema(),
        &[("s_gmt_offset", 5), ("s_tax_precentage", 5)],
    );
    assert_decimal_fields(
        StoreReturnsArrow::new(session.clone()).schema(),
        &[
            ("sr_return_amt", 7),
            ("sr_return_tax", 7),
            ("sr_return_amt_inc_tax", 7),
            ("sr_fee", 7),
            ("sr_return_ship_cost", 7),
            ("sr_refunded_cash", 7),
            ("sr_reversed_charge", 7),
            ("sr_store_credit", 7),
            ("sr_net_loss", 7),
        ],
    );
    assert_decimal_fields(
        StoreSalesArrow::new(session.clone()).schema(),
        &[
            ("ss_wholesale_cost", 7),
            ("ss_list_price", 7),
            ("ss_sales_price", 7),
            ("ss_ext_discount_amt", 7),
            ("ss_ext_sales_price", 7),
            ("ss_ext_wholesale_cost", 7),
            ("ss_ext_list_price", 7),
            ("ss_ext_tax", 7),
            ("ss_coupon_amt", 7),
            ("ss_net_paid", 7),
            ("ss_net_paid_inc_tax", 7),
            ("ss_net_profit", 7),
        ],
    );
    assert_decimal_fields(
        WarehouseArrow::new(session.clone()).schema(),
        &[("w_gmt_offset", 5)],
    );
    assert_decimal_fields(
        WebReturnsArrow::new(session.clone()).schema(),
        &[
            ("wr_return_amt", 7),
            ("wr_return_tax", 7),
            ("wr_return_amt_inc_tax", 7),
            ("wr_fee", 7),
            ("wr_return_ship_cost", 7),
            ("wr_refunded_cash", 7),
            ("wr_reversed_charge", 7),
            ("wr_account_credit", 7),
            ("wr_net_loss", 7),
        ],
    );
    assert_decimal_fields(
        WebSalesArrow::new(session.clone()).schema(),
        &[
            ("ws_wholesale_cost", 7),
            ("ws_list_price", 7),
            ("ws_sales_price", 7),
            ("ws_ext_discount_amt", 7),
            ("ws_ext_sales_price", 7),
            ("ws_ext_wholesale_cost", 7),
            ("ws_ext_list_price", 7),
            ("ws_ext_tax", 7),
            ("ws_coupon_amt", 7),
            ("ws_ext_ship_cost", 7),
            ("ws_net_paid", 7),
            ("ws_net_paid_inc_tax", 7),
            ("ws_net_paid_inc_ship", 7),
            ("ws_net_paid_inc_ship_tax", 7),
            ("ws_net_profit", 7),
        ],
    );
    assert_decimal_fields(
        WebSiteArrow::new(session).schema(),
        &[("web_gmt_offset", 5), ("web_tax_percentage", 5)],
    );
}
