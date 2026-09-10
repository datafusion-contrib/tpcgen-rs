//! Verifies canonical TPC-DS column names, ordering, data types, and nullability.

use arrow::datatypes::{DataType, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatchReader;
use std::collections::BTreeSet;
use tpcdsgen::config::{Scaling, Session, Table};
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

/// `(table name, Arrow schema, canonical column names)` for every TPC-DS table.
fn canonical_schemas(session: &Session) -> Vec<(&'static str, SchemaRef, &'static [&'static str])> {
    vec![
        (
            "call_center",
            CallCenterArrow::new(session.clone()).schema(),
            &[
                "cc_call_center_sk",
                "cc_call_center_id",
                "cc_rec_start_date",
                "cc_rec_end_date",
                "cc_closed_date_sk",
                "cc_open_date_sk",
                "cc_name",
                "cc_class",
                "cc_employees",
                "cc_sq_ft",
                "cc_hours",
                "cc_manager",
                "cc_mkt_id",
                "cc_mkt_class",
                "cc_mkt_desc",
                "cc_market_manager",
                "cc_division",
                "cc_division_name",
                "cc_company",
                "cc_company_name",
                "cc_street_number",
                "cc_street_name",
                "cc_street_type",
                "cc_suite_number",
                "cc_city",
                "cc_county",
                "cc_state",
                "cc_zip",
                "cc_country",
                "cc_gmt_offset",
                "cc_tax_percentage",
            ],
        ),
        (
            "catalog_page",
            CatalogPageArrow::new(session.clone()).schema(),
            &[
                "cp_catalog_page_sk",
                "cp_catalog_page_id",
                "cp_start_date_sk",
                "cp_end_date_sk",
                "cp_department",
                "cp_catalog_number",
                "cp_catalog_page_number",
                "cp_description",
                "cp_type",
            ],
        ),
        (
            "catalog_returns",
            CatalogReturnsArrow::new(session.clone()).schema(),
            &[
                "cr_returned_date_sk",
                "cr_returned_time_sk",
                "cr_item_sk",
                "cr_refunded_customer_sk",
                "cr_refunded_cdemo_sk",
                "cr_refunded_hdemo_sk",
                "cr_refunded_addr_sk",
                "cr_returning_customer_sk",
                "cr_returning_cdemo_sk",
                "cr_returning_hdemo_sk",
                "cr_returning_addr_sk",
                "cr_call_center_sk",
                "cr_catalog_page_sk",
                "cr_ship_mode_sk",
                "cr_warehouse_sk",
                "cr_reason_sk",
                "cr_order_number",
                "cr_return_quantity",
                "cr_return_amount",
                "cr_return_tax",
                "cr_return_amt_inc_tax",
                "cr_fee",
                "cr_return_ship_cost",
                "cr_refunded_cash",
                "cr_reversed_charge",
                "cr_store_credit",
                "cr_net_loss",
            ],
        ),
        (
            "catalog_sales",
            CatalogSalesArrow::new(session.clone()).schema(),
            &[
                "cs_sold_date_sk",
                "cs_sold_time_sk",
                "cs_ship_date_sk",
                "cs_bill_customer_sk",
                "cs_bill_cdemo_sk",
                "cs_bill_hdemo_sk",
                "cs_bill_addr_sk",
                "cs_ship_customer_sk",
                "cs_ship_cdemo_sk",
                "cs_ship_hdemo_sk",
                "cs_ship_addr_sk",
                "cs_call_center_sk",
                "cs_catalog_page_sk",
                "cs_ship_mode_sk",
                "cs_warehouse_sk",
                "cs_item_sk",
                "cs_promo_sk",
                "cs_order_number",
                "cs_quantity",
                "cs_wholesale_cost",
                "cs_list_price",
                "cs_sales_price",
                "cs_ext_discount_amt",
                "cs_ext_sales_price",
                "cs_ext_wholesale_cost",
                "cs_ext_list_price",
                "cs_ext_tax",
                "cs_coupon_amt",
                "cs_ext_ship_cost",
                "cs_net_paid",
                "cs_net_paid_inc_tax",
                "cs_net_paid_inc_ship",
                "cs_net_paid_inc_ship_tax",
                "cs_net_profit",
            ],
        ),
        (
            "customer",
            CustomerArrow::new(session.clone()).schema(),
            &[
                "c_customer_sk",
                "c_customer_id",
                "c_current_cdemo_sk",
                "c_current_hdemo_sk",
                "c_current_addr_sk",
                "c_first_shipto_date_sk",
                "c_first_sales_date_sk",
                "c_salutation",
                "c_first_name",
                "c_last_name",
                "c_preferred_cust_flag",
                "c_birth_day",
                "c_birth_month",
                "c_birth_year",
                "c_birth_country",
                "c_login",
                "c_email_address",
                "c_last_review_date_sk",
            ],
        ),
        (
            "customer_address",
            CustomerAddressArrow::new(session.clone()).schema(),
            &[
                "ca_address_sk",
                "ca_address_id",
                "ca_street_number",
                "ca_street_name",
                "ca_street_type",
                "ca_suite_number",
                "ca_city",
                "ca_county",
                "ca_state",
                "ca_zip",
                "ca_country",
                "ca_gmt_offset",
                "ca_location_type",
            ],
        ),
        (
            "customer_demographics",
            CustomerDemographicsArrow::new(session.clone()).schema(),
            &[
                "cd_demo_sk",
                "cd_gender",
                "cd_marital_status",
                "cd_education_status",
                "cd_purchase_estimate",
                "cd_credit_rating",
                "cd_dep_count",
                "cd_dep_employed_count",
                "cd_dep_college_count",
            ],
        ),
        (
            "date_dim",
            DateDimArrow::new(session.clone()).schema(),
            &[
                "d_date_sk",
                "d_date_id",
                "d_date",
                "d_month_seq",
                "d_week_seq",
                "d_quarter_seq",
                "d_year",
                "d_dow",
                "d_moy",
                "d_dom",
                "d_qoy",
                "d_fy_year",
                "d_fy_quarter_seq",
                "d_fy_week_seq",
                "d_day_name",
                "d_quarter_name",
                "d_holiday",
                "d_weekend",
                "d_following_holiday",
                "d_first_dom",
                "d_last_dom",
                "d_same_day_ly",
                "d_same_day_lq",
                "d_current_day",
                "d_current_week",
                "d_current_month",
                "d_current_quarter",
                "d_current_year",
            ],
        ),
        (
            "dbgen_version",
            DbgenVersionArrow::new(session.clone()).schema(),
            &[
                "dv_version",
                "dv_create_date",
                "dv_create_time",
                "dv_cmdline_args",
            ],
        ),
        (
            "household_demographics",
            HouseholdDemographicsArrow::new(session.clone()).schema(),
            &[
                "hd_demo_sk",
                "hd_income_band_sk",
                "hd_buy_potential",
                "hd_dep_count",
                "hd_vehicle_count",
            ],
        ),
        (
            "income_band",
            IncomeBandArrow::new(session.clone()).schema(),
            &["ib_income_band_sk", "ib_lower_bound", "ib_upper_bound"],
        ),
        (
            "inventory",
            InventoryArrow::new(session.clone()).schema(),
            &[
                "inv_date_sk",
                "inv_item_sk",
                "inv_warehouse_sk",
                "inv_quantity_on_hand",
            ],
        ),
        (
            "item",
            ItemArrow::new(session.clone()).schema(),
            &[
                "i_item_sk",
                "i_item_id",
                "i_rec_start_date",
                "i_rec_end_date",
                "i_item_desc",
                "i_current_price",
                "i_wholesale_cost",
                "i_brand_id",
                "i_brand",
                "i_class_id",
                "i_class",
                "i_category_id",
                "i_category",
                "i_manufact_id",
                "i_manufact",
                "i_size",
                "i_formulation",
                "i_color",
                "i_units",
                "i_container",
                "i_manager_id",
                "i_product_name",
            ],
        ),
        (
            "promotion",
            PromotionArrow::new(session.clone()).schema(),
            &[
                "p_promo_sk",
                "p_promo_id",
                "p_start_date_sk",
                "p_end_date_sk",
                "p_item_sk",
                "p_cost",
                "p_response_target",
                "p_promo_name",
                "p_channel_dmail",
                "p_channel_email",
                "p_channel_catalog",
                "p_channel_tv",
                "p_channel_radio",
                "p_channel_press",
                "p_channel_event",
                "p_channel_demo",
                "p_channel_details",
                "p_purpose",
                "p_discount_active",
            ],
        ),
        (
            "reason",
            ReasonArrow::new(session.clone()).schema(),
            &["r_reason_sk", "r_reason_id", "r_reason_desc"],
        ),
        (
            "ship_mode",
            ShipModeArrow::new(session.clone()).schema(),
            &[
                "sm_ship_mode_sk",
                "sm_ship_mode_id",
                "sm_type",
                "sm_code",
                "sm_carrier",
                "sm_contract",
            ],
        ),
        (
            "store",
            StoreArrow::new(session.clone()).schema(),
            &[
                "s_store_sk",
                "s_store_id",
                "s_rec_start_date",
                "s_rec_end_date",
                "s_closed_date_sk",
                "s_store_name",
                "s_number_employees",
                "s_floor_space",
                "s_hours",
                "s_manager",
                "s_market_id",
                "s_geography_class",
                "s_market_desc",
                "s_market_manager",
                "s_division_id",
                "s_division_name",
                "s_company_id",
                "s_company_name",
                "s_street_number",
                "s_street_name",
                "s_street_type",
                "s_suite_number",
                "s_city",
                "s_county",
                "s_state",
                "s_zip",
                "s_country",
                "s_gmt_offset",
                "s_tax_precentage",
            ],
        ),
        (
            "store_returns",
            StoreReturnsArrow::new(session.clone()).schema(),
            &[
                "sr_returned_date_sk",
                "sr_return_time_sk",
                "sr_item_sk",
                "sr_customer_sk",
                "sr_cdemo_sk",
                "sr_hdemo_sk",
                "sr_addr_sk",
                "sr_store_sk",
                "sr_reason_sk",
                "sr_ticket_number",
                "sr_return_quantity",
                "sr_return_amt",
                "sr_return_tax",
                "sr_return_amt_inc_tax",
                "sr_fee",
                "sr_return_ship_cost",
                "sr_refunded_cash",
                "sr_reversed_charge",
                "sr_store_credit",
                "sr_net_loss",
            ],
        ),
        (
            "store_sales",
            StoreSalesArrow::new(session.clone()).schema(),
            &[
                "ss_sold_date_sk",
                "ss_sold_time_sk",
                "ss_item_sk",
                "ss_customer_sk",
                "ss_cdemo_sk",
                "ss_hdemo_sk",
                "ss_addr_sk",
                "ss_store_sk",
                "ss_promo_sk",
                "ss_ticket_number",
                "ss_quantity",
                "ss_wholesale_cost",
                "ss_list_price",
                "ss_sales_price",
                "ss_ext_discount_amt",
                "ss_ext_sales_price",
                "ss_ext_wholesale_cost",
                "ss_ext_list_price",
                "ss_ext_tax",
                "ss_coupon_amt",
                "ss_net_paid",
                "ss_net_paid_inc_tax",
                "ss_net_profit",
            ],
        ),
        (
            "time_dim",
            TimeDimArrow::new(session.clone()).schema(),
            &[
                "t_time_sk",
                "t_time_id",
                "t_time",
                "t_hour",
                "t_minute",
                "t_second",
                "t_am_pm",
                "t_shift",
                "t_sub_shift",
                "t_meal_time",
            ],
        ),
        (
            "warehouse",
            WarehouseArrow::new(session.clone()).schema(),
            &[
                "w_warehouse_sk",
                "w_warehouse_id",
                "w_warehouse_name",
                "w_warehouse_sq_ft",
                "w_street_number",
                "w_street_name",
                "w_street_type",
                "w_suite_number",
                "w_city",
                "w_county",
                "w_state",
                "w_zip",
                "w_country",
                "w_gmt_offset",
            ],
        ),
        (
            "web_page",
            WebPageArrow::new(session.clone()).schema(),
            &[
                "wp_web_page_sk",
                "wp_web_page_id",
                "wp_rec_start_date",
                "wp_rec_end_date",
                "wp_creation_date_sk",
                "wp_access_date_sk",
                "wp_autogen_flag",
                "wp_customer_sk",
                "wp_url",
                "wp_type",
                "wp_char_count",
                "wp_link_count",
                "wp_image_count",
                "wp_max_ad_count",
            ],
        ),
        (
            "web_returns",
            WebReturnsArrow::new(session.clone()).schema(),
            &[
                "wr_returned_date_sk",
                "wr_returned_time_sk",
                "wr_item_sk",
                "wr_refunded_customer_sk",
                "wr_refunded_cdemo_sk",
                "wr_refunded_hdemo_sk",
                "wr_refunded_addr_sk",
                "wr_returning_customer_sk",
                "wr_returning_cdemo_sk",
                "wr_returning_hdemo_sk",
                "wr_returning_addr_sk",
                "wr_web_page_sk",
                "wr_reason_sk",
                "wr_order_number",
                "wr_return_quantity",
                "wr_return_amt",
                "wr_return_tax",
                "wr_return_amt_inc_tax",
                "wr_fee",
                "wr_return_ship_cost",
                "wr_refunded_cash",
                "wr_reversed_charge",
                "wr_account_credit",
                "wr_net_loss",
            ],
        ),
        (
            "web_sales",
            WebSalesArrow::new(session.clone()).schema(),
            &[
                "ws_sold_date_sk",
                "ws_sold_time_sk",
                "ws_ship_date_sk",
                "ws_item_sk",
                "ws_bill_customer_sk",
                "ws_bill_cdemo_sk",
                "ws_bill_hdemo_sk",
                "ws_bill_addr_sk",
                "ws_ship_customer_sk",
                "ws_ship_cdemo_sk",
                "ws_ship_hdemo_sk",
                "ws_ship_addr_sk",
                "ws_web_page_sk",
                "ws_web_site_sk",
                "ws_ship_mode_sk",
                "ws_warehouse_sk",
                "ws_promo_sk",
                "ws_order_number",
                "ws_quantity",
                "ws_wholesale_cost",
                "ws_list_price",
                "ws_sales_price",
                "ws_ext_discount_amt",
                "ws_ext_sales_price",
                "ws_ext_wholesale_cost",
                "ws_ext_list_price",
                "ws_ext_tax",
                "ws_coupon_amt",
                "ws_ext_ship_cost",
                "ws_net_paid",
                "ws_net_paid_inc_tax",
                "ws_net_paid_inc_ship",
                "ws_net_paid_inc_ship_tax",
                "ws_net_profit",
            ],
        ),
        (
            "web_site",
            WebSiteArrow::new(session.clone()).schema(),
            &[
                "web_site_sk",
                "web_site_id",
                "web_rec_start_date",
                "web_rec_end_date",
                "web_name",
                "web_open_date_sk",
                "web_close_date_sk",
                "web_class",
                "web_manager",
                "web_mkt_id",
                "web_mkt_class",
                "web_mkt_desc",
                "web_market_manager",
                "web_company_id",
                "web_company_name",
                "web_street_number",
                "web_street_name",
                "web_street_type",
                "web_suite_number",
                "web_city",
                "web_county",
                "web_state",
                "web_zip",
                "web_country",
                "web_gmt_offset",
                "web_tax_percentage",
            ],
        ),
    ]
}

#[test]
fn schemas_match_expected_columns_and_canonical_types() {
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

#[test]
fn integer_widths_match_lakebench_v4() {
    let session = Session::default();
    let mut integer_fields = 0;
    let mut bigint_fields = Vec::new();

    for (table, schema) in table_schemas(&session) {
        for field in schema.fields() {
            match field.data_type() {
                DataType::Int32 => {
                    integer_fields += 1;
                }
                DataType::Int64 => {
                    bigint_fields.push(format!("{}.{}", table.get_name(), field.name()));
                }
                _ => {}
            }
        }
    }

    bigint_fields.sort_unstable();
    assert_eq!(integer_fields, 183);
    assert_eq!(
        bigint_fields,
        [
            "catalog_returns.cr_order_number",
            "catalog_sales.cs_order_number",
            "store_returns.sr_ticket_number",
            "store_sales.ss_ticket_number",
            "web_returns.wr_order_number",
            "web_sales.ws_order_number",
        ]
    );
}

#[test]
fn sf100000_integer_domains_fit_i32() {
    let scaling = Scaling::new(100000.0);
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
            scaling.get_row_count(table) <= i64::from(i32::MAX),
            "{} exceeds the Arrow Int32 key domain at SF100000",
            table.get_name()
        );
    }

    for table in [Table::StoreSales, Table::CatalogSales, Table::WebSales] {
        assert!(
            scaling.get_row_count(table) > i64::from(i32::MAX),
            "{} order identifiers require Arrow Int64 at SF100000",
            table.get_name()
        );
    }
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

#[test]
fn tpcds_schemas_use_canonical_column_names() {
    let session = Session::default();
    for (table, schema, expected) in canonical_schemas(&session) {
        let actual: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert_eq!(actual, expected, "column names for table {table}");
    }
}
