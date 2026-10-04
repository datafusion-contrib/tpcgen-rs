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

//! Catalog returns row generator

use crate::config::Session;
use crate::error::Result;
use crate::generator::CatalogReturnsGeneratorColumn;
use crate::join_key_utils::generate_join_key;
use crate::nulls::create_null_bit_map;
use crate::random::RandomValueGenerator;
use crate::row::catalog_returns_row::CatalogReturnsRow;
use crate::row::catalog_sales_row::CatalogSalesRow;
use crate::row::catalog_sales_row_generator::CatalogSalesRowGenerator;
use crate::row::AbstractRowGenerator;
use crate::table::Table;
use crate::types::generate_pricing_for_returns_table;

/// Percentage of sales that get returned (same as store returns)
pub const RETURN_PERCENT: i32 = 10;

/// Percentage of returns where the ship customer returns (vs bill customer)
const GIFT_PERCENTAGE: i32 = 10;

/// Generates `catalog_returns` rows: the returned line items of `catalog_sales`.
///
/// Replays the `catalog_sales` line items through a [`CatalogSalesRowGenerator`],
/// generating the sales row only for the ~10% of line items that are
/// returned.
pub struct CatalogReturnsRowGenerator {
    sales: CatalogSalesRowGenerator,
    abstract_generator: AbstractRowGenerator,
}

impl CatalogReturnsRowGenerator {
    /// Generate the returns of `catalog_sales` source rows `1..=row_count`.
    pub fn new(session: Session, row_count: u64) -> Self {
        CatalogReturnsRowGenerator {
            sales: CatalogSalesRowGenerator::new(session, row_count),
            abstract_generator: AbstractRowGenerator::new(Table::CatalogReturns, row_count),
        }
    }

    /// Start generating at source row `starting_row_number` (1-based), fast
    /// forwarding the random number streams to that row.
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.sales
            .skip_rows_until_starting_row_number(starting_row_number);
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Restrict generation to source rows
    /// `starting_row_number..=ending_row_number` (1-based, inclusive).
    ///
    /// The ending row number is clamped to the table's row count.
    pub fn set_source_row_range(&mut self, starting_row_number: u64, ending_row_number: u64) {
        self.sales
            .set_source_row_range(starting_row_number, ending_row_number);
        self.abstract_generator
            .set_source_row_range(starting_row_number, ending_row_number);
    }

    /// Generate the return row for `sales_row`.
    fn generate_row(&mut self, sales_row: &CatalogSalesRow) -> Result<CatalogReturnsRow> {
        use CatalogReturnsGeneratorColumn::*;

        let scaling = self.sales.session().get_scaling();

        // Generate null bit map
        let stream = self.abstract_generator.get_random_number_stream(&CrNulls);
        let null_bit_map = create_null_bit_map(Table::CatalogReturns, stream);

        // Some fields are conditionally taken from the sale
        // By default, use bill customer info (which gets refunded)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturningCustomerSk);
        let mut cr_returning_customer_sk = generate_join_key(
            &CrReturningCustomerSk,
            stream,
            crate::config::Table::Customer,
            2,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturningCdemoSk);
        let mut cr_returning_cdemo_sk = generate_join_key(
            &CrReturningCdemoSk,
            stream,
            crate::config::Table::CustomerDemographics,
            2,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturningHdemoSk);
        let cr_returning_hdemo_sk = generate_join_key(
            &CrReturningHdemoSk,
            stream,
            crate::config::Table::HouseholdDemographics,
            2,
            scaling,
        )?;

        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturningAddrSk);
        let mut cr_returning_addr_sk = generate_join_key(
            &CrReturningAddrSk,
            stream,
            crate::config::Table::CustomerAddress,
            2,
            scaling,
        )?;

        // If the order was a gift (10%), the ship customer is doing the return
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturningCustomerSk);
        let random_int = RandomValueGenerator::generate_uniform_random_int(0, 99, stream);
        if random_int < GIFT_PERCENTAGE {
            cr_returning_customer_sk = sales_row.get_cs_ship_customer_sk();
            cr_returning_cdemo_sk = sales_row.get_cs_ship_cdemo_sk();
            // skip cr_returning_hdemo_sk, since it doesn't exist on the sales record
            cr_returning_addr_sk = sales_row.get_cs_ship_addr_sk();
        }

        // Generate return quantity (1 to original sale quantity)
        let sales_pricing = sales_row.get_cs_pricing();
        let quantity = if sales_pricing.get_quantity() == -1 {
            sales_pricing.get_quantity()
        } else {
            let stream = self.abstract_generator.get_random_number_stream(&CrPricing);
            RandomValueGenerator::generate_uniform_random_int(
                1,
                sales_pricing.get_quantity(),
                stream,
            )
        };

        // Generate return pricing
        let stream = self.abstract_generator.get_random_number_stream(&CrPricing);
        let cr_pricing = generate_pricing_for_returns_table(stream, quantity, sales_pricing);

        // Generate returned date (based on ship date + lag)
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturnedDateSk);
        let cr_returned_date_sk = generate_join_key(
            &CrReturnedDateSk,
            stream,
            crate::config::Table::DateDim,
            sales_row.get_cs_ship_date_sk(),
            scaling,
        )?;

        // Generate returned time
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReturnedTimeSk);
        let cr_returned_time_sk = generate_join_key(
            &CrReturnedTimeSk,
            stream,
            crate::config::Table::TimeDim,
            1,
            scaling,
        )?;

        // Generate ship mode
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrShipModeSk);
        let cr_ship_mode_sk = generate_join_key(
            &CrShipModeSk,
            stream,
            crate::config::Table::ShipMode,
            1,
            scaling,
        )?;

        // Generate warehouse
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrWarehouseSk);
        let cr_warehouse_sk = generate_join_key(
            &CrWarehouseSk,
            stream,
            crate::config::Table::Warehouse,
            1,
            scaling,
        )?;

        // Generate reason
        let stream = self
            .abstract_generator
            .get_random_number_stream(&CrReasonSk);
        let cr_reason_sk = generate_join_key(
            &CrReasonSk,
            stream,
            crate::config::Table::Reason,
            1,
            scaling,
        )?;

        Ok(CatalogReturnsRow::new(
            null_bit_map,
            cr_returned_date_sk,
            cr_returned_time_sk,
            sales_row.get_cs_sold_item_sk(), // cr_item_sk from sales
            sales_row.get_cs_bill_customer_sk(), // cr_refunded_customer_sk from sales bill
            sales_row.get_cs_bill_cdemo_sk(), // cr_refunded_cdemo_sk from sales bill
            sales_row.get_cs_bill_hdemo_sk(), // cr_refunded_hdemo_sk from sales bill
            sales_row.get_cs_bill_addr_sk(), // cr_refunded_addr_sk from sales bill
            cr_returning_customer_sk,
            cr_returning_cdemo_sk,
            cr_returning_hdemo_sk,
            cr_returning_addr_sk,
            sales_row.get_cs_call_center_sk(), // cr_call_center_sk from sales
            sales_row.get_cs_catalog_page_sk(), // cr_catalog_page_sk from sales
            cr_ship_mode_sk,
            cr_warehouse_sk,
            cr_reason_sk,
            sales_row.get_cs_order_number(), // cr_order_number from sales
            cr_pricing,
        ))
    }
}

impl Iterator for CatalogReturnsRowGenerator {
    type Item = CatalogReturnsRow;

    fn next(&mut self) -> Option<CatalogReturnsRow> {
        loop {
            let item = self.sales.next_line_item().expect("row gen")?;
            let row = if item.is_returned {
                let sales_row = self
                    .sales
                    .generate_sales_row(item.item_sk)
                    .expect("row gen");
                Some(self.generate_row(&sales_row).expect("row gen"))
            } else {
                self.sales.skip_item_sales_draws();
                None
            };
            if self.sales.finish_line_item() {
                self.abstract_generator.consume_remaining_seeds_for_row();
            }
            if row.is_some() {
                return row;
            }
        }
    }
}
