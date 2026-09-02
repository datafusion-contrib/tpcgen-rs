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
use crate::row::{AbstractRowGenerator, GeneratedRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use crate::types::generate_pricing_for_returns_table;

/// Percentage of sales that get returned (same as store returns)
pub const RETURN_PERCENT: i32 = 10;

/// Percentage of returns where the ship customer returns (vs bill customer)
const GIFT_PERCENTAGE: i32 = 10;

pub struct CatalogReturnsRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

impl CatalogReturnsRowGenerator {
    pub fn new() -> Self {
        CatalogReturnsRowGenerator {
            abstract_generator: AbstractRowGenerator::new(Table::CatalogReturns),
        }
    }

    /// Generate a return row from a sales row
    /// This is called by CatalogSalesRowGenerator when a sale is returned
    pub fn generate_row(
        &mut self,
        session: &Session,
        sales_row: &CatalogSalesRow,
    ) -> Result<GeneratedRow> {
        use CatalogReturnsGeneratorColumn::*;

        let scaling = session.get_scaling();

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
        )
        .into())
    }
}

impl Default for CatalogReturnsRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RowGenerator for CatalogReturnsRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        _row_number: u64,
        _session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        // The catalog_returns table is a child of the catalog_sales table because you can only
        // return things that have already been purchased. This method should only get called
        // if we are generating the catalog_returns table in isolation.
        // Otherwise catalog_returns is generated during the generation of the catalog_sales table
        // via the generate_row method above.
        //
        // For now, we panic if called directly - the proper way is to generate through
        // catalog_sales which calls our generate_row method.
        panic!("CatalogReturnsRowGenerator::generate_row_and_child_rows should not be called directly. Use CatalogSalesRowGenerator to generate both sales and returns.");
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
