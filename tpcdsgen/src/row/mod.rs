pub mod abstract_row_generator;
pub mod generated_row;
pub mod table_row;
mod tables;

pub use abstract_row_generator::AbstractRowGenerator;
pub use generated_row::GeneratedRow;

/// One line item of a sales order, as stepped through by a sales generator
/// and the returns generator that replays it.
pub(crate) struct LineItem {
    pub(crate) item_sk: i64,
    /// Whether the item is returned, so has a row in the returns table.
    pub(crate) is_returned: bool,
}

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

#[cfg(test)]
mod tests {
    use crate::config::{Session, SessionBuilder, Table};
    use crate::row::{
        CallCenterRowGenerator, ItemRowGenerator, ReasonRowGenerator, StoreRowGenerator,
        WebPageRowGenerator, WebSiteRowGenerator,
    };

    fn session(scale_factor: f64) -> Session {
        SessionBuilder::new()
            .with_scale_factor(scale_factor)
            .build()
            .expect("session")
    }

    /// Collect the DAT text of the rows `$generator` emits for `$table` over
    /// each of `$ranges`, concatenated in order.
    macro_rules! rows_for {
        ($generator:ty, $table:expr, $session:expr, $ranges:expr) => {{
            let session: &Session = $session;
            let row_count = session.get_scaling().get_row_count($table);
            let mut out: Vec<String> = Vec::new();
            for &(start, end) in $ranges {
                let mut rows = <$generator>::new(session.clone(), row_count);
                rows.set_source_row_range(start, end);
                out.extend(rows.map(|row| row.to_string()));
            }
            out
        }};
    }

    /// Splitting a table into source row ranges must produce exactly the same
    /// rows as generating it in one pass.
    #[test]
    fn source_row_ranges_concatenate_to_the_unranged_output() {
        let session = session(1.0);
        let whole = rows_for!(ReasonRowGenerator, Table::Reason, &session, &[(1, 35)]);
        let chunked = rows_for!(
            ReasonRowGenerator,
            Table::Reason,
            &session,
            &[(1, 1), (2, 10), (11, 34), (35, 35)]
        );

        assert_eq!(whole.len(), 35);
        assert_eq!(whole, chunked);
    }

    /// An empty range produces nothing
    #[test]
    fn an_empty_range_produces_no_rows() {
        let session = session(1.0);
        let rows = rows_for!(ReasonRowGenerator, Table::Reason, &session, &[(1, 0)]);
        assert!(rows.is_empty());
    }

    /// Assert that generating `$table` one source row at a time reproduces the
    /// unranged output. Every row is a range start, so this covers each
    /// position of the six-row revision cycle.
    macro_rules! assert_scd_single_row_ranges_match {
        ($generator:ty, $table:expr) => {{
            let session = session(1.0);
            // Two full revision cycles are enough; call_center only has six rows.
            let row_count = session.get_scaling().get_row_count($table).min(12);
            let singles: Vec<(u64, u64)> = (1..=row_count).map(|row| (row, row)).collect();

            let whole = rows_for!($generator, $table, &session, &[(1, row_count)]);
            assert_eq!(whole.len(), row_count as usize, "{}", $table);
            assert_eq!(
                whole,
                rows_for!($generator, $table, &session, &singles),
                "{}",
                $table
            );
        }};
    }

    /// A range of an SCD table can start on a revision that copies values from
    /// the row before it, which the range never generates.
    #[test]
    fn scd_source_row_ranges_concatenate_to_the_unranged_output() {
        assert_scd_single_row_ranges_match!(ItemRowGenerator, Table::Item);
        assert_scd_single_row_ranges_match!(StoreRowGenerator, Table::Store);
        assert_scd_single_row_ranges_match!(WebPageRowGenerator, Table::WebPage);
        assert_scd_single_row_ranges_match!(WebSiteRowGenerator, Table::WebSite);
        assert_scd_single_row_ranges_match!(CallCenterRowGenerator, Table::CallCenter);
    }

    /// Reusing one generator across seeks must not carry revision state from
    /// the old position, including when seeking backwards or to the same row.
    #[test]
    fn seeking_an_scd_generator_rebuilds_its_history() {
        let session = session(1.0);
        let row_count = 12;
        let whole = rows_for!(ItemRowGenerator, Table::Item, &session, &[(1, row_count)]);

        let mut rows = ItemRowGenerator::new(session.clone(), row_count);
        for start in [row_count, 6, 3, 6, 1] {
            rows.skip_rows_until_starting_row_number(start);
            let row = rows.next().expect("row").to_string();
            assert_eq!(row, whole[start as usize - 1], "seek {start}");
        }
    }
}
